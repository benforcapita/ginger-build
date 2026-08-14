/// Ginger Code — Workspace Trust
/// New repositories begin untrusted. Reading/editing and safe scans are allowed,
/// but project-defined executable behavior is blocked until trust is granted.
/// Trust is explicit and reversible.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Untrusted,
    Trusted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTrust {
    pub workspace_id: i64,
    pub root: String,
    pub level: TrustLevel,
    pub granted_at: Option<u64>,
}

pub struct TrustStore {
    workspaces: Mutex<HashMap<i64, WorkspaceTrust>>,
}

impl TrustStore {
    pub fn new() -> Self {
        Self {
            workspaces: Mutex::new(HashMap::new()),
        }
    }

    pub fn grant(&self, workspace_id: i64, root: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.workspaces.lock().unwrap().insert(
            workspace_id,
            WorkspaceTrust {
                workspace_id,
                root: root.to_string(),
                level: TrustLevel::Trusted,
                granted_at: Some(now),
            },
        );
    }

    pub fn revoke(&self, workspace_id: i64) {
        if let Some(ws) = self.workspaces.lock().unwrap().get_mut(&workspace_id) {
            ws.level = TrustLevel::Untrusted;
            ws.granted_at = None;
        }
    }

    pub fn is_trusted(&self, workspace_id: i64) -> bool {
        self.workspaces
            .lock()
            .unwrap()
            .get(&workspace_id)
            .map(|w| w.level == TrustLevel::Trusted)
            .unwrap_or(false)
    }

    pub fn get(&self, workspace_id: i64) -> Option<WorkspaceTrust> {
        self.workspaces.lock().unwrap().get(&workspace_id).cloned()
    }
}

impl Default for TrustStore {
    fn default() -> Self {
        Self::new()
    }
}