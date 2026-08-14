/// Ginger Code — Typed Domain Events
/// Events use domain.entity.action naming: agent.thread.started, git.worktree.changed, etc.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DomainEvent {
    // Workspace
    #[serde(rename = "workspace.opened")]
    WorkspaceOpened { workspace_id: i64, root: String },
    #[serde(rename = "workspace.closed")]
    WorkspaceClosed { workspace_id: i64 },

    // Editor
    #[serde(rename = "editor.ready")]
    EditorReady { session_id: i64 },
    #[serde(rename = "editor.config.error")]
    EditorConfigError { error: String, safe_mode: bool },

    // Agent
    #[serde(rename = "agent.thread.created")]
    AgentThreadCreated { agent_id: i64, adapter: String },
    #[serde(rename = "agent.thread.started")]
    AgentThreadStarted { agent_id: i64, worktree_path: String },
    #[serde(rename = "agent.thread.status.changed")]
    AgentThreadStatusChanged { agent_id: i64, status: String },
    #[serde(rename = "agent.thread.completed")]
    AgentThreadCompleted { agent_id: i64, success: bool },
    #[serde(rename = "agent.thread.failed")]
    AgentThreadFailed { agent_id: i64, error: String },

    // Worktree
    #[serde(rename = "worktree.created")]
    WorktreeCreated { worktree_id: i64, path: String, branch: String },
    #[serde(rename = "worktree.changed")]
    WorktreeChanged { worktree_id: i64, files_changed: usize },
    #[serde(rename = "worktree.applied")]
    WorktreeApplied { worktree_id: i64, strategy: String },

    // Verification
    #[serde(rename = "verification.started")]
    VerificationStarted { run_id: i64, agent_id: i64 },
    #[serde(rename = "verification.completed")]
    VerificationCompleted { run_id: i64, success: bool },

    // Package
    #[serde(rename = "package.recommendations.updated")]
    PackageRecommendationsUpdated { workspace_id: i64, count: usize },
    #[serde(rename = "package.install.started")]
    PackageInstallStarted { package_id: String },
    #[serde(rename = "package.install.completed")]
    PackageInstallCompleted { package_id: String, success: bool },

    // Recovery
    #[serde(rename = "recovery.required")]
    RecoveryRequired { reason: String },
    #[serde(rename = "recovery.completed")]
    RecoveryCompleted { worktrees_recovered: usize, safe_mode: bool },

    // Background job
    #[serde(rename = "job.progress")]
    JobProgress { job_id: String, phase: String, completed: usize, total: usize },
    #[serde(rename = "job.cancelled")]
    JobCancelled { job_id: String },
}