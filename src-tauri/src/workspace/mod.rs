pub mod commands;
pub mod search;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        if !p.is_dir() {
            return Err(WorkspaceError::PathNotFound(path.into()));
        }

        let p = p.canonicalize().map_err(|e| WorkspaceError::Inner(e.to_string()))?;
        let display_name = p
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let is_git = p.join(".git").exists();

        let workspace = Workspace {
            id: None,
            root_path: p.to_string_lossy().to_string(),
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

    pub fn resolve_path(&self, relative: &str) -> Result<PathBuf, WorkspaceError> {
        let workspace = self.current().ok_or_else(|| WorkspaceError::Inner("open a folder first".into()))?;
        let relative = std::path::Path::new(relative);
        if relative.is_absolute() || relative.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return Err(WorkspaceError::Inner("path must stay inside the workspace".into()));
        }
        let root = PathBuf::from(workspace.root_path);
        let path = root.join(relative).canonicalize().map_err(|e| WorkspaceError::Inner(e.to_string()))?;
        if !path.starts_with(&root) { return Err(WorkspaceError::Inner("path leaves the workspace".into())); }
        Ok(path)
    }

    pub fn list_directory(&self, relative: &str) -> Result<Vec<FileEntry>, WorkspaceError> {
        let directory = self.resolve_path(relative)?;
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(directory).map_err(|e| WorkspaceError::Inner(e.to_string()))? {
            let entry = entry.map_err(|e| WorkspaceError::Inner(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" { continue; }
            let path = std::path::Path::new(relative).join(&name).to_string_lossy().to_string();
            // Do not expose external symlink targets in the explorer.
            let Ok(resolved) = self.resolve_path(&path) else { continue; };
            entries.push(FileEntry { name, path, is_dir: resolved.is_dir() });
        }
        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())).then(a.name.cmp(&b.name)));
        Ok(entries)
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_files_as_workspace_roots() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("code.rs");
        std::fs::write(&file, "hello").unwrap();
        assert!(WorkspaceService::new().open(file.to_str().unwrap()).is_err());
    }

    #[test]
    fn lists_directories_first_and_blocks_escape() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("README.md"), "hello").unwrap();
        let svc = WorkspaceService::new();
        svc.open(dir.path().to_str().unwrap()).unwrap();
        let entries = svc.list_directory("").unwrap();
        assert_eq!(entries[0].name, "src");
        assert!(entries[0].is_dir);
        assert_eq!(entries[1].path, "README.md");
        assert!(svc.resolve_path("../").is_err());
        assert!(svc.resolve_path("/etc/passwd").is_err());
    }

    #[test]
    #[cfg(unix)]
    fn blocks_symlinks_outside_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape")).unwrap();
        let svc = WorkspaceService::new();
        svc.open(dir.path().to_str().unwrap()).unwrap();
        assert!(svc.resolve_path("escape").is_err());
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
}
