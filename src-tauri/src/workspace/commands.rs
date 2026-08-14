// Ginger Code — Workspace Tauri commands
// Exposes workspace operations to the frontend.

use crate::workspace::{Workspace, WorkspaceService, PaneState};
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceStatus {
    pub open: bool,
    pub workspace: Option<Workspace>,
    pub pane_state: PaneState,
    pub is_git: bool,
}

#[tauri::command]
pub fn workspace_open(
    svc: State<'_, WorkspaceService>,
    path: String,
) -> Result<Workspace, String> {
    svc.open(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn workspace_close(svc: State<'_, WorkspaceService>) -> Result<(), String> {
    svc.close();
    Ok(())
}

#[tauri::command]
pub fn workspace_status(svc: State<'_, WorkspaceService>) -> WorkspaceStatus {
    let workspace = svc.current();
    let open = workspace.is_some();
    let is_git = workspace
        .as_ref()
        .map(|w| std::path::Path::new(&w.root_path).join(".git").exists())
        .unwrap_or(false);
    WorkspaceStatus {
        open,
        workspace,
        pane_state: svc.pane_state(),
        is_git,
    }
}

#[tauri::command]
pub fn workspace_set_pane_state(
    svc: State<'_, WorkspaceService>,
    state: PaneState,
) -> Result<(), String> {
    svc.set_pane_state(state);
    Ok(())
}