use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicU64, Ordering}}};
use parking_lot::Mutex;
use serde::Serialize;
use tauri::{ipc::Channel, State};
use tokio::{io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader}, process::Command, sync::{mpsc, oneshot, Semaphore}};
use crate::{workspace::WorkspaceService, terminal::{execution_path, resolve_program}};
const MAX_MESSAGE: usize = 16 * 1024 * 1024;
fn executable(server: &str) -> Result<(&'static str, &'static [&'static str]), String> {
    match server {
        "typescript" => Ok(("typescript-language-server", &["--stdio"])),
        "python" => Ok(("pyright-langserver", &["--stdio"])),
        "rust" => Ok(("rust-analyzer", &[])),
        "json" => Ok(("vscode-json-language-server", &["--stdio"])),
        _ => Err("Unsupported language server".into()),
    }
}
fn install_root() -> Result<PathBuf, String> { Ok(dirs::home_dir().ok_or("Home directory unavailable")?.join(".ginger/language-servers")) }
fn resolve(server: &str, root: &Path) -> Result<PathBuf, String> {
    let (program, _) = executable(server)?;
    let managed = install_root()?.join("node_modules/.bin").join(program);
    if managed.is_file() { return Ok(managed); }
    if let Ok(path) = resolve_program(program, root) { return Ok(path); }
    if server == "rust" {
        let rustup = resolve_program("rustup", root).map_err(|e| e.to_string())?;
        let output = std::process::Command::new(rustup).args(["which", "rust-analyzer"]).current_dir(root).env("PATH", execution_path()).output().map_err(|e| e.to_string())?;
        if output.status.success() { let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()); if path.is_file() { return Ok(path); } }
    }
    Err(format!("{program} is not installed. Open Language servers from Commands to install it."))
}
#[derive(Clone, Serialize)]
pub struct ServerInfo { id: String, executable: String, available: bool, path: Option<String>, install_program: String, install_args: Vec<String> }
#[tauri::command]
pub fn language_server_status(workspace: State<'_, WorkspaceService>) -> Vec<ServerInfo> {
    let root = workspace.current().map(|w| PathBuf::from(w.root_path)).unwrap_or_else(|| PathBuf::from("/"));
    ["typescript", "python", "rust", "json"].into_iter().map(|id| {
        let path = resolve(id, &root).ok();
        let packages: &[&str] = match id { "typescript" => &["typescript@6.0.3", "typescript-language-server"], "python" => &["pyright"], "json" => &["vscode-langservers-extracted"], _ => &[] };
        let args = if id == "rust" { vec!["component".into(), "add".into(), "rust-analyzer".into(), "rust-src".into()] }
            else { let mut args = vec!["install".into(), "--prefix".into(), install_root().unwrap_or_default().to_string_lossy().into_owned(), "--no-audit".into(), "--no-fund".into()]; args.extend(packages.iter().map(|p| p.to_string())); args };
        ServerInfo { id: id.into(), executable: executable(id).unwrap().0.into(), available: path.is_some(), path: path.map(|p| p.to_string_lossy().into_owned()), install_program: if id == "rust" { "rustup" } else { "npm" }.into(), install_args: args }
    }).collect()
}
#[derive(Clone, Serialize)]
pub struct ServerEvent { id: u64, kind: &'static str, data: String }
struct Running { server: String, delivered: Arc<Semaphore>, input: mpsc::Sender<String>, stop: Option<oneshot::Sender<()>> }
impl Drop for Running { fn drop(&mut self) { if let Some(stop) = self.stop.take() { let _ = stop.send(()); } } }
#[derive(Default)]
pub struct LanguageServers { next: AtomicU64, running: Arc<Mutex<HashMap<u64, Running>>> }
impl LanguageServers { pub fn stop_all(&self) { self.running.lock().clear(); } }

async fn read_message<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Option<String>, String> {
    let mut header = Vec::new();
    loop {
        let byte = match reader.read_u8().await { Ok(b) => b, Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof && header.is_empty() => return Ok(None), Err(e) => return Err(e.to_string()) };
        header.push(byte);
        if header.ends_with(b"\r\n\r\n") { break; }
        if header.len() >= 8192 { return Err("Language server header too large".into()); }
    }
    let header = std::str::from_utf8(&header).map_err(|e| e.to_string())?;
    let mut length = None;
    for line in header.split("\r\n") {
        if let Some((key, value)) = line.split_once(':') {
            if key.eq_ignore_ascii_case("Content-Length") {
                if length.is_some() { return Err("Duplicate Content-Length".into()); }
                length = Some(value.trim().parse::<usize>().map_err(|e| e.to_string())?);
            }
        }
    }
    let length = length.ok_or("Missing Content-Length")?;
    if length > MAX_MESSAGE { return Err("Language server message too large".into()); }
    let mut body = vec![0; length];
    reader.read_exact(&mut body).await.map_err(|e| e.to_string())?;
    String::from_utf8(body).map(Some).map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn language_server_start(host: State<'_, LanguageServers>, workspace: State<'_, WorkspaceService>, server: String, root: String, on_event: Channel<ServerEvent>) -> Result<u64, String> {
    let current = workspace.current().ok_or("Open a folder first")?;
    if current.root_path != root { return Err("Workspace changed".into()); }
    let root = PathBuf::from(root);
    let program = resolve(&server, &root)?;
    let (_, args) = executable(&server)?;
    let mut paths: Vec<PathBuf> = std::env::split_paths(&execution_path()).collect();
    if let Some(parent) = program.parent() { paths.insert(0, parent.to_owned()); }
    let child_path = std::env::join_paths(paths).map_err(|e| e.to_string())?;
    let mut command = Command::new(program);
    command.args(args).current_dir(&root).env("PATH", child_path).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).kill_on_drop(true);
    #[cfg(unix)] { command.process_group(0); }
    let mut running = host.running.lock();
    if running.len() >= 4 { return Err("Maximum four language servers per workspace".into()); }
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let pid = child.id().ok_or("Missing process ID")?;
    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let (input, mut messages) = mpsc::channel::<String>(32);
    let (stop, mut stopped) = oneshot::channel();
    let id = host.next.fetch_add(1, Ordering::Relaxed) + 1;
    let delivered = Arc::new(Semaphore::new(2));
    running.insert(id, Running { server, delivered: delivered.clone(), input, stop: Some(stop) });
    drop(running);
    let all = host.running.clone();
    tauri::async_runtime::spawn(async move {
        let tail = Arc::new(Mutex::new(Vec::<u8>::new()));
        let err_tail = tail.clone();
        let errors = tokio::spawn(async move {
            let mut buf = [0; 1024];
            while let Ok(n) = stderr.read(&mut buf).await { if n == 0 { break; } let mut tail = err_tail.lock(); tail.extend_from_slice(&buf[..n]); if tail.len() > 4096 { let n = tail.len() - 4096; tail.drain(..n); } }
        });
        let output_channel = on_event.clone();
        let mut output = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                let permit = delivered.acquire().await.map_err(|e| e.to_string())?;
                let Some(data) = read_message(&mut reader).await? else { break; };
                output_channel.send(ServerEvent { id, kind: "message", data }).map_err(|e| e.to_string())?;
                permit.forget();
            }
            Ok::<(), String>(())
        });
        let mut writer = tokio::spawn(async move {
            while let Some(message) = messages.recv().await {
                let frame = format!("Content-Length: {}\r\n\r\n{}", message.len(), message);
                stdin.write_all(frame.as_bytes()).await.map_err(|e| e.to_string())?;
                stdin.flush().await.map_err(|e| e.to_string())?;
            }
            Ok::<(), String>(())
        });
        let reason = tokio::select! {
            _ = &mut stopped => "Stopped".to_owned(),
            status = child.wait() => format!("Language server exited: {}", match status { Ok(status) => status.to_string(), Err(error) => error.to_string() }),
            result = &mut output => format!("Server output closed: {result:?}"),
            result = &mut writer => format!("Server input closed: {result:?}"),
        };
        #[cfg(unix)] unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
        let _ = child.kill().await;
        let _ = child.wait().await;
        output.abort(); writer.abort(); errors.abort();
        all.lock().remove(&id);
        let detail = String::from_utf8_lossy(&tail.lock()).to_string();
        let _ = on_event.send(ServerEvent { id, kind: "exit", data: format!("{reason}\n{detail}") });
    });
    Ok(id)
}
#[tauri::command]
pub fn language_server_send(host: State<'_, LanguageServers>, id: u64, message: String) -> Result<(), String> {
    if message.len() > MAX_MESSAGE { return Err("Language server message too large".into()); }
    let all = host.running.lock();
    let process = all.get(&id).ok_or("Language server is not running")?;
    let mut value: serde_json::Value = serde_json::from_str(&message).map_err(|e| e.to_string())?;
    if process.server == "typescript" && value["method"] == "initialize" {
        value["params"]["initializationOptions"]["tsserver"]["fallbackPath"] = serde_json::Value::String(install_root()?.join("node_modules/typescript/lib/tsserver.js").to_string_lossy().into_owned());
    }
    process.input.try_send(value.to_string()).map_err(|e| e.to_string())
}
#[tauri::command]
pub fn language_server_ack(host: State<'_, LanguageServers>, id: u64) {
    if let Some(process) = host.running.lock().get(&id) { if process.delivered.available_permits() < 2 { process.delivered.add_permits(1); } }
}
#[tauri::command]
pub fn language_server_stop(host: State<'_, LanguageServers>, id: u64) { host.running.lock().remove(&id); }
#[tauri::command]
pub fn language_servers_stop_all(host: State<'_, LanguageServers>) { host.stop_all(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn parses_multiple_frames_and_utf8_bytes() {
        let body = "{\"text\":\"世界\"}";
        let input = format!("Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n{}Content-Length: 2\r\n\r\n{{}}", body.len(), body);
        let mut reader = input.as_bytes();
        assert_eq!(read_message(&mut reader).await.unwrap(), Some(body.into()));
        assert_eq!(read_message(&mut reader).await.unwrap(), Some("{}".into()));
        assert_eq!(read_message(&mut reader).await.unwrap(), None);
    }
    #[tokio::test]
    async fn rejects_bad_and_unbounded_frames() {
        for input in ["Content-Length: 999999999\r\n\r\n", "Wrong: 2\r\n\r\n{}", "Content-Length: 2\r\nContent-Length: 3\r\n\r\n{}", "Content-Length: 4\r\n\r\n{}"] {
            assert!(read_message(&mut input.as_bytes()).await.is_err());
        }
    }
    #[test]
    fn only_known_commands_are_launchable() { assert!(executable("sh").is_err()); assert_eq!(executable("python").unwrap().0, "pyright-langserver"); }
}
