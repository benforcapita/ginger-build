// Ginger Code — Terminal Tauri commands
// Exposes PTY terminal operations to the frontend.

use crate::terminal::{TerminalHost, TerminalInfo, TerminalOwner};
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTerminalArgs {
    pub cwd: String,
    pub shell: Option<String>,
    pub owner_type: Option<String>, // "user" | "agent"
    pub owner_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreateTerminalResult {
    pub id: u64,
}

#[tauri::command]
pub async fn terminal_create(
    host: State<'_, TerminalHost>,
    args: CreateTerminalArgs,
) -> Result<CreateTerminalResult, String> {
    let owner = match args.owner_type.as_deref() {
        Some("agent") => TerminalOwner::Agent,
        _ => TerminalOwner::User,
    };
    let id = host.create(
        &std::path::PathBuf::from(&args.cwd),
        args.shell.as_deref(),
        owner,
        args.owner_id,
    ).map_err(|e| e.to_string())?;
    Ok(CreateTerminalResult { id })
}

#[tauri::command]
pub fn terminal_write(
    host: State<'_, TerminalHost>,
    id: u64,
    data: Vec<u8>,
) -> Result<(), String> {
    host.write(id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_resize(
    host: State<'_, TerminalHost>,
    id: u64,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    host.resize(id, rows, cols).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_terminate(
    host: State<'_, TerminalHost>,
    id: u64,
) -> Result<(), String> {
    host.terminate(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_list(host: State<'_, TerminalHost>) -> Vec<TerminalInfo> {
    host.list()
}
#[tauri::command]
pub fn terminal_subscribe(host: State<'_, TerminalHost>, id: u64, on_event: tauri::ipc::Channel<crate::terminal::TerminalEvent>) -> Result<(), String> {
    host.subscribe(id, on_event).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn terminal_launch(
    host: State<'_, TerminalHost>,
    workspace: State<'_, crate::workspace::WorkspaceService>,
    kind: String,
    program: Option<String>,
    args: Option<Vec<String>>,
    path: Option<String>,
) -> Result<CreateTerminalResult, String> {
    let root = workspace.resolve_path("").map_err(|e| e.to_string())?;
    let id = match kind.as_str() {
        "editor" => {
            let relative = path.ok_or("select a file first")?;
            let file = workspace.resolve_path(&relative).map_err(|e| e.to_string())?;
            if !file.is_file() { return Err("select a regular file".into()); }
            let bundled = dirs::home_dir().unwrap_or_default().join(".ginger/runtime/bin/nvim");
            let executable = if bundled.is_file() { bundled.to_string_lossy().to_string() } else { "nvim".into() };
            host.launch(&root, &executable, &["--".into(), file.to_string_lossy().to_string()], TerminalOwner::Editor, None)
        }
        "agent" => {
            if host.list().iter().filter(|s| s.owner_type == TerminalOwner::Agent && !s.exited).count() >= 3 {
                return Err("close an active harness before starting another (maximum 3)".into());
            }
            let program = program.ok_or("choose a harness executable")?;
            host.launch(&root, &program, &args.unwrap_or_default(), TerminalOwner::Agent, None)
        }
        "shell" => if let Some(program) = program { host.launch(&root, &program, &args.unwrap_or_default(), TerminalOwner::User, None) } else { host.create(&root, None, TerminalOwner::User, None) },
        _ => return Err("unknown session kind".into()),
    }.map_err(|e| e.to_string())?;
    Ok(CreateTerminalResult { id })
}

#[tauri::command]
pub fn terminal_terminate_all(host: State<'_, TerminalHost>) { host.terminate_all(); }

#[derive(Serialize)]
pub struct HarnessInfo { pub name: String, pub program: String, pub available: bool }

#[tauri::command]
pub fn terminal_harnesses() -> Vec<HarnessInfo> {
    [("Claude Code", "claude"), ("Codex", "codex"), ("OpenCode", "opencode")].into_iter().map(|(name, program)| HarnessInfo {
        name: name.into(), program: program.into(), available: crate::terminal::resolve_program(program, std::path::Path::new("/")).is_ok(),
    }).collect()
}
