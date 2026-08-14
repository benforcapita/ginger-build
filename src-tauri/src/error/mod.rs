/// Ginger Code — Rust Error Conventions (LLD 215)
/// Internal errors retain technical causes; IPC exposes stable codes such as
/// GINGER_GIT_WORKTREE_CREATE_FAILED and GINGER_EDITOR_RPC_HANDSHAKE_FAILED.
/// Frontend behavior depends on codes, not message parsing.

use serde::{Deserialize, Serialize};

/// Stable, machine-readable error codes exposed over IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GingerErrorCode {
    // Git
    GitWorktreeCreateFailed,
    GitWorktreeListFailed,
    GitApplyFailed,
    GitCherryPickFailed,
    GitBranchSwitchFailed,
    // Editor
    EditorRpcHandshakeFailed,
    EditorSpawnFailed,
    EditorConfigError,
    // Agent
    AgentSpawnFailed,
    AgentAdapterNotFound,
    AgentAtCapacity,
    // Package
    PackageResolveFailed,
    PackageInstallFailed,
    PackageRollbackFailed,
    // Workspace
    WorkspaceOpenFailed,
    WorkspaceTrustRequired,
    WorkspaceMissing,
    // Recovery
    RecoveryRequired,
    // IPC
    IpcVersionMismatch,
    // Generic
    Internal,
}

impl GingerErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitWorktreeCreateFailed => "GINGER_GIT_WORKTREE_CREATE_FAILED",
            Self::GitWorktreeListFailed => "GINGER_GIT_WORKTREE_LIST_FAILED",
            Self::GitApplyFailed => "GINGER_GIT_APPLY_FAILED",
            Self::GitCherryPickFailed => "GINGER_GIT_CHERRY_PICK_FAILED",
            Self::GitBranchSwitchFailed => "GINGER_GIT_BRANCH_SWITCH_FAILED",
            Self::EditorRpcHandshakeFailed => "GINGER_EDITOR_RPC_HANDSHAKE_FAILED",
            Self::EditorSpawnFailed => "GINGER_EDITOR_SPAWN_FAILED",
            Self::EditorConfigError => "GINGER_EDITOR_CONFIG_ERROR",
            Self::AgentSpawnFailed => "GINGER_AGENT_SPAWN_FAILED",
            Self::AgentAdapterNotFound => "GINGER_AGENT_ADAPTER_NOT_FOUND",
            Self::AgentAtCapacity => "GINGER_AGENT_AT_CAPACITY",
            Self::PackageResolveFailed => "GINGER_PACKAGE_RESOLVE_FAILED",
            Self::PackageInstallFailed => "GINGER_PACKAGE_INSTALL_FAILED",
            Self::PackageRollbackFailed => "GINGER_PACKAGE_ROLLBACK_FAILED",
            Self::WorkspaceOpenFailed => "GINGER_WORKSPACE_OPEN_FAILED",
            Self::WorkspaceTrustRequired => "GINGER_WORKSPACE_TRUST_REQUIRED",
            Self::WorkspaceMissing => "GINGER_WORKSPACE_MISSING",
            Self::RecoveryRequired => "GINGER_RECOVERY_REQUIRED",
            Self::IpcVersionMismatch => "GINGER_IPC_VERSION_MISMATCH",
            Self::Internal => "GINGER_INTERNAL",
        }
    }
}

/// A Ginger error carrying a stable code plus a technical detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GingerError {
    pub code: GingerErrorCode,
    pub detail: String,
}

impl GingerError {
    pub fn new(code: GingerErrorCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }

    pub fn code_str(&self) -> &'static str {
        self.code.as_str()
    }
}

impl std::fmt::Display for GingerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.detail)
    }
}

impl std::error::Error for GingerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_strings() {
        assert_eq!(
            GingerErrorCode::EditorRpcHandshakeFailed.as_str(),
            "GINGER_EDITOR_RPC_HANDSHAKE_FAILED"
        );
        assert_eq!(
            GingerErrorCode::GitWorktreeCreateFailed.as_str(),
            "GINGER_GIT_WORKTREE_CREATE_FAILED"
        );
    }
}