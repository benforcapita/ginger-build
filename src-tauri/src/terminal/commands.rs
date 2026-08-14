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