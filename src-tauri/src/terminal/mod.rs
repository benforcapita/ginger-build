pub mod commands;
mod process;
use process::{OwnedProcess, SharedMaster};

use parking_lot::Mutex;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde::Serialize;
use std::{collections::{HashMap, VecDeque}, io::{Read, Write}, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicU64, Ordering}}};
use tauri::ipc::Channel;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("terminal: {0}")]
    Pty(String),
    #[error("terminal not found: {0}")]
    NotFound(u64),
    #[error("terminal already exited: {0}")]
    Exited(u64),
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TerminalEvent {
    Output { data: Vec<u8> },
    Exit { code: Option<u32> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TerminalOwner { User, Agent, Editor }

#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub id: u64,
    pub cwd: String,
    pub shell: String,
    pub args: Vec<String>,
    pub owner_type: TerminalOwner,
    pub owner_id: Option<u64>,
    pub exited: bool,
}

#[derive(Default)]
struct OutputState {
    history: VecDeque<u8>,
    channel: Option<Channel<TerminalEvent>>,
    exited: bool,
    exit_code: Option<u32>,
}

impl OutputState {
    fn output(&mut self, bytes: &[u8]) {
        self.history.extend(bytes);
        let excess = self.history.len().saturating_sub(512 * 1024);
        self.history.drain(..excess);
        if let Some(channel) = &self.channel {
            if channel.send(TerminalEvent::Output { data: bytes.to_vec() }).is_err() {
                self.channel = None;
            }
        }
    }

    fn exit(&mut self, code: Option<u32>) {
        self.exited = true;
        self.exit_code = code;
        if let Some(channel) = self.channel.take() {
            let _ = channel.send(TerminalEvent::Exit { code });
        }
    }
}

struct TerminalSession {
    info: TerminalInfo,
    master: SharedMaster,
    input: std::sync::mpsc::SyncSender<Vec<u8>>,
    process: Arc<Mutex<OwnedProcess>>,
    output: Arc<Mutex<OutputState>>,
}

impl TerminalSession {
    fn shutdown(&self) -> Result<(), TerminalError> { self.process.lock().shutdown() }
}

impl Drop for TerminalSession {
    fn drop(&mut self) { let _ = self.shutdown(); }
}

#[derive(Default)]
pub struct TerminalHost {
    next_id: AtomicU64,
    sessions: Mutex<HashMap<u64, TerminalSession>>,
}

impl TerminalHost {
    pub fn create(&self, cwd: &PathBuf, shell: Option<&str>, owner: TerminalOwner, owner_id: Option<u64>) -> Result<u64, TerminalError> {
        let shell = shell.map(str::to_owned).unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into()));
        self.launch(cwd, &shell, &["-l".into()], owner, owner_id)
    }

    pub fn launch(&self, cwd: &Path, program: &str, args: &[String], owner: TerminalOwner, owner_id: Option<u64>) -> Result<u64, TerminalError> {
        if !cwd.is_dir() { return Err(TerminalError::Pty("working directory does not exist".into())); }
        let executable = resolve_program(program, cwd)?;
        let pair = native_pty_system().openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 }).map_err(pty_error)?;
        let mut command = CommandBuilder::new(executable);
        command.args(args);
        command.cwd(cwd);
        command.env("PATH", execution_path());
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        let mut reader = pair.master.try_clone_reader().map_err(pty_error)?;
        let mut writer = pair.master.take_writer().map_err(pty_error)?;
        let child = pair.slave.spawn_command(command).map_err(pty_error)?;
        drop(pair.slave);
        let master = Arc::new(Mutex::new(pair.master));
        let process = Arc::new(Mutex::new(OwnedProcess::new(child, master.clone())));
        let reader_process = process.clone();
        let monitor_process = process.clone();
        let (input, input_rx) = std::sync::mpsc::sync_channel::<Vec<u8>>(32);
        let id = self.next_id.fetch_add(1, Ordering::Relaxed) + 1;
        let output = Arc::new(Mutex::new(OutputState::default()));
        let reader_output = output.clone();
        self.sessions.lock().insert(id, TerminalSession {
            info: TerminalInfo { id, cwd: cwd.display().to_string(), shell: program.to_string(), args: args.to_vec(), owner_type: owner, owner_id, exited: false },
            master, input, process, output,
        });
        std::thread::spawn(move || {
            loop {
                match monitor_process.lock().poll() {
                    Ok(true) => break,
                    Ok(false) => {},
                    Err(error) => { tracing::error!("process monitor: {error}"); break; }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        });
        std::thread::spawn(move || {
            while let Ok(data) = input_rx.recv() {
                if writer.write_all(&data).and_then(|_| writer.flush()).is_err() { break; }
            }
        });
        std::thread::spawn(move || {
            let mut buffer = [0u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => reader_output.lock().output(&buffer[..n]),
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
            let code = loop {
                let process = reader_process.lock();
                if process.complete { break process.code; }
                drop(process);
                std::thread::sleep(std::time::Duration::from_millis(10));
            };
            reader_output.lock().exit(code);
        });
        Ok(id)
    }

    pub fn subscribe(&self, id: u64, channel: Channel<TerminalEvent>) -> Result<(), TerminalError> {
        let sessions = self.sessions.lock();
        let session = sessions.get(&id).ok_or(TerminalError::NotFound(id))?;
        let mut output = session.output.lock();
        // Replay and attachment share the lock: no gap or duplicated output.
        channel.send(TerminalEvent::Output { data: output.history.iter().copied().collect() }).map_err(pty_error)?;
        if output.exited { channel.send(TerminalEvent::Exit { code: output.exit_code }).map_err(pty_error)?; }
        else { output.channel = Some(channel); }
        Ok(())
    }

    pub fn write(&self, id: u64, data: &[u8]) -> Result<(), TerminalError> {
        if data.len() > 64 * 1024 { return Err(TerminalError::Pty("paste is too large; paste at most 64 KiB at a time".into())); }
        let sessions = self.sessions.lock();
        let session = sessions.get(&id).ok_or(TerminalError::NotFound(id))?;
        if session.output.lock().exited { return Err(TerminalError::Exited(id)); }
        session.input.try_send(data.to_vec()).map_err(|error| TerminalError::Pty(format!("terminal input queue unavailable: {error}")))
    }

    pub fn resize(&self, id: u64, rows: u16, cols: u16) -> Result<(), TerminalError> {
        let sessions = self.sessions.lock();
        let session = sessions.get(&id).ok_or(TerminalError::NotFound(id))?;
        let result = session.master.lock().resize(PtySize { rows: rows.max(1), cols: cols.max(1), pixel_width: 0, pixel_height: 0 }).map_err(pty_error);
        result
    }

    pub fn terminate(&self, id: u64) -> Result<(), TerminalError> {
        // Keep ownership if shutdown fails, so the session remains closeable.
        let mut sessions = self.sessions.lock();
        sessions.get(&id).ok_or(TerminalError::NotFound(id))?.shutdown()?;
        sessions.remove(&id);
        Ok(())
    }

    pub fn terminate_all(&self) { self.sessions.lock().clear(); }

    pub fn list(&self) -> Vec<TerminalInfo> {
        let mut list: Vec<_> = self.sessions.lock().values().map(|session| {
            let mut info = session.info.clone();
            info.exited = session.output.lock().exited;
            info
        }).collect();
        list.sort_by_key(|session| session.id);
        list
    }
}

fn execution_path() -> std::ffi::OsString {
    let current = std::env::var_os("PATH").unwrap_or_default();
    let mut paths: Vec<PathBuf> = std::env::split_paths(&current).collect();
    if let Some(home) = dirs::home_dir() {
        paths.extend([home.join(".local/bin"), home.join(".cargo/bin"), home.join(".npm-global/bin")]);
    }
    paths.extend([PathBuf::from("/opt/homebrew/bin"), PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/bin"), PathBuf::from("/bin")]);
    std::env::join_paths(paths).unwrap_or(current)
}

pub fn resolve_program(program: &str, cwd: &Path) -> Result<PathBuf, TerminalError> {
    which::which_in(program, Some(execution_path()), cwd).map_err(|_| TerminalError::Pty(format!("{program} is not installed or not on PATH")))
}

fn pty_error(error: impl std::fmt::Display) -> TerminalError { TerminalError::Pty(error.to_string()) }

#[cfg(test)]
mod tests;
