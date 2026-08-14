// Ginger Code — Workspace Management
// Handles opening folders, tracking active workspace, and session lifecycle.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("workspace error: {0}")]
    Inner(String),
    #[error("path does not exist: {0}")]
    PathNotFound(String),
    #[error("not a git repository: {0}")]
    NotAGitRepo(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: Option<i64>,
    pub root_path: String,
    pub display_name: String,
    pub runtime_version: String,
    pub created_at: String,
    pub last_opened_at: Option<String>,
    pub active_session_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaneState {
    pub explorer_visible: bool,
    pub explorer_width: u32,
    pub agent_dock_visible: bool,
    pub agent_dock_width: u32,
    pub bottom_panel_visible: bool,
    pub bottom_panel_height: u32,
    pub bottom_panel_tab: String,
}

impl Default for PaneState {
    fn default() -> Self {
        Self {
            explorer_visible: true,
            explorer_width: 240,
            agent_dock_visible: false,
            agent_dock_width: 300,
            bottom_panel_visible: false,
            bottom_panel_height: 240,
            bottom_panel_tab: "terminal".into(),
        }
    }
}

pub struct WorkspaceService {
    current: parking_lot::RwLock<Option<Workspace>>,
    pane_state: parking_lot::RwLock<PaneState>,
}

impl WorkspaceService {
    pub fn new() -> Self {
        Self {
            current: parking_lot::RwLock::new(None),
            pane_state: parking_lot::RwLock::new(PaneState::default()),
        }
    }

    /// Open a workspace from a folder path.
    pub fn open(&self, path: &str) -> Result<Workspace, WorkspaceError> {
        let p = PathBuf::from(path);
        if !p.exists() {
            return Err(WorkspaceError::PathNotFound(path.into()));
        }

        let display_name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let is_git = p.join(".git").exists();

        let workspace = Workspace {
            id: None,
            root_path: path.into(),
            display_name,
            runtime_version: "0.1.0".into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_opened_at: None,
            active_session_id: None,
        };

        *self.current.write() = Some(workspace.clone());
        *self.pane_state.write() = PaneState::default();

        tracing::info!(
            "Workspace opened: {} (git: {})",
            workspace.display_name,
            is_git
        );

        Ok(workspace)
    }

    /// Close the current workspace.
    pub fn close(&self) {
        *self.current.write() = None;
        *self.pane_state.write() = PaneState::default();
        tracing::info!("Workspace closed");
    }

    /// Get the current workspace.
    pub fn current(&self) -> Option<Workspace> {
        self.current.read().clone()
    }

    /// Get the current pane state.
    pub fn pane_state(&self) -> PaneState {
        self.pane_state.read().clone()
    }

    /// Update pane state.
    pub fn set_pane_state(&self, state: PaneState) {
        *self.pane_state.write() = state;
    }

    /// Check if a workspace is open.
    pub fn is_open(&self) -> bool {
        self.current.read().is_some()
    }
}

impl Default for WorkspaceService {
    fn default() -> Self { Self::new() }
}