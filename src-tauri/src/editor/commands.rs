// Ginger Code — Editor Tauri commands
// Exposes editor operations to the frontend via Tauri IPC.

use crate::editor::{EditorError, NeovimHost};
use crate::persistence::PersistenceService;
use crate::editor::core;
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EditorStatus {
    pub alive: bool,
    pub runtime_path: String,
    pub safe_mode: bool,
}

#[tauri::command]
pub async fn editor_start(
    host: tauri::State<'_, tokio::sync::Mutex<NeovimHost>>,
    persistence: tauri::State<'_, PersistenceService>,
) -> Result<(), EditorError> {
    // Ensure protected core exists
    core::ensure_protected_core(persistence.data_root())
        .map_err(|e| EditorError::UserConfig(e.to_string()))?;

    let mut host = host.lock().await;
    host.start().await
}

#[tauri::command]
pub async fn editor_stop(
    host: tauri::State<'_, tokio::sync::Mutex<NeovimHost>>,
) -> Result<(), EditorError> {
    let mut host = host.lock().await;
    host.stop().await
}

#[tauri::command]
pub async fn editor_status(
    host: tauri::State<'_, tokio::sync::Mutex<NeovimHost>>,
) -> Result<EditorStatus, EditorError> {
    let host = host.lock().await;
    Ok(EditorStatus {
        alive: host.is_alive(),
        runtime_path: host.runtime_path().display().to_string(),
        safe_mode: false,
    })
}