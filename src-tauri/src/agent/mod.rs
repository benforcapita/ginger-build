// Ginger Code — Agent Supervisor
// Manages coding agents (Claude Code, Codex, Ollama, custom) in isolated worktrees.
// Durable thread: create intent → allocate worktree → start PTY/agent → stream output → detect completion → verify → review → apply/discard.

use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent error: {0}")]
    Inner(String),
    #[error("adapter not found: {0}")]
    AdapterNotFound(String),
    #[error("max concurrent agents reached ({0})")]
    MaxConcurrent(usize),
    #[error("agent not found: {0}")]
    NotFound(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AdapterId {
    ClaudeCode,
    Codex,
    Ollama,
    Custom(String),
}

impl std::fmt::Display for AdapterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterId::ClaudeCode => write!(f, "claude-code"),
            AdapterId::Codex => write!(f, "codex"),
            AdapterId::Ollama => write!(f, "ollama"),
            AdapterId::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentMode {
    Coding,
    Review,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum IsolationMode {
    Worktree,
    ReadOnly,
    Primary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentThread {
    pub id: u64,
    pub adapter_id: AdapterId,
    pub title: String,
    pub mode: AgentMode,
    pub status: AgentStatus,
    pub isolation: IsolationMode,
    pub worktree_path: Option<String>,
    pub worktree_branch: Option<String>,
    pub terminal_id: Option<u64>,
    pub base_revision: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

pub struct AgentSupervisor {
    threads: Arc<RwLock<std::collections::HashMap<u64, AgentThread>>>,
    next_id: Arc<RwLock<u64>>,
    max_concurrent: usize,
}

impl AgentSupervisor {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            threads: Arc::new(RwLock::new(std::collections::HashMap::new())),
            next_id: Arc::new(RwLock::new(1)),
            max_concurrent,
        }
    }

    pub fn active_count(&self) -> usize {
        self.threads.read().values()
            .filter(|t| t.status == AgentStatus::Running || t.status == AgentStatus::Pending)
            .count()
    }

    pub fn create(&self, req: CreateAgentRequest) -> Result<AgentThread, AgentError> {
        if self.active_count() >= self.max_concurrent {
            return Err(AgentError::MaxConcurrent(self.max_concurrent));
        }

        let id = {
            let mut next = self.next_id.write();
            let current = *next;
            *next += 1;
            current
        };

        let thread = AgentThread {
            id,
            adapter_id: req.adapter_id,
            title: req.title,
            mode: req.mode,
            status: AgentStatus::Pending,
            isolation: req.isolation,
            worktree_path: None,
            worktree_branch: None,
            terminal_id: None,
            base_revision: None,
            started_at: None,
            finished_at: None,
        };

        self.threads.write().insert(id, thread.clone());
        tracing::info!("Agent {} created: {}", id, thread.title);
        Ok(thread)
    }

    pub fn start(&self, id: u64, worktree_path: Option<String>, worktree_branch: Option<String>, base_revision: Option<String>, terminal_id: Option<u64>) -> Result<(), AgentError> {
        let mut threads = self.threads.write();
        let thread = threads.get_mut(&id).ok_or(AgentError::NotFound(id))?;
        thread.status = AgentStatus::Running;
        thread.worktree_path = worktree_path;
        thread.worktree_branch = worktree_branch;
        thread.base_revision = base_revision;
        thread.terminal_id = terminal_id;
        thread.started_at = Some(chrono::Utc::now().to_rfc3339());
        tracing::info!("Agent {} started", id);
        Ok(())
    }

    pub fn complete(&self, id: u64, success: bool) -> Result<(), AgentError> {
        let mut threads = self.threads.write();
        let thread = threads.get_mut(&id).ok_or(AgentError::NotFound(id))?;
        thread.status = if success { AgentStatus::Completed } else { AgentStatus::Failed };
        thread.finished_at = Some(chrono::Utc::now().to_rfc3339());
        tracing::info!("Agent {} completed (success: {})", id, success);
        Ok(())
    }

    pub fn get(&self, id: u64) -> Option<AgentThread> {
        self.threads.read().get(&id).cloned()
    }

    pub fn list(&self) -> Vec<AgentThread> {
        self.threads.read().values().cloned().collect()
    }

    pub fn remove(&self, id: u64) -> Result<(), AgentError> {
        let mut threads = self.threads.write();
        threads.remove(&id).ok_or(AgentError::NotFound(id))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub adapter_id: AdapterId,
    pub title: String,
    pub mode: AgentMode,
    pub isolation: IsolationMode,
}