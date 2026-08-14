// Ginger Code — Crash Recovery + Safe Mode
// Heartbeat identifies abnormal shutdown.
// Recovery reconciles workspace root, Git repository, worktrees, agent processes, package env, editor session.
// Recovery prioritizes source/worktree preservation over process resurrection.
// Never auto-delete recovered work.

use std::path::PathBuf;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("recovery error: {0}")]
    Inner(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Open,
    Closing,
    Closed,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub session_id: i64,
    pub last_beat: String,
    pub app_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryReport {
    pub crashed_sessions: Vec<i64>,
    pub recovered_worktrees: Vec<RecoveryWorktree>,
    pub actions: Vec<RecoveryAction>,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryWorktree {
    pub path: String,
    pub branch: String,
    pub status: String,
    pub has_unapplied_changes: bool,
    pub owner_agent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub action: String,
    pub target: String,
    pub result: String,
}

pub struct RecoveryService {
    last_heartbeat: RwLock<Option<Instant>>,
    heartbeat_interval: Duration,
    safe_mode: RwLock<bool>,
}

impl RecoveryService {
    pub fn new() -> Self {
        Self {
            last_heartbeat: RwLock::new(None),
            heartbeat_interval: Duration::from_secs(30),
            safe_mode: RwLock::new(false),
        }
    }

    /// Record a heartbeat — called periodically while the app is running.
    pub fn heartbeat(&self) {
        *self.last_heartbeat.write() = Some(Instant::now());
    }

    /// Check if the last heartbeat is stale (indicates crash).
    pub fn is_stale(&self) -> bool {
        match *self.last_heartbeat.read() {
            Some(t) => t.elapsed() > self.heartbeat_interval * 2,
            None => false,
        }
    }

    /// Enter safe mode — disables user Lua and optional packages.
    pub fn enter_safe_mode(&self) {
        *self.safe_mode.write() = true;
        tracing::warn!("Entering safe mode — user Lua and optional packages disabled");
    }

    /// Exit safe mode.
    pub fn exit_safe_mode(&self) {
        *self.safe_mode.write() = false;
        tracing::info!("Exiting safe mode");
    }

    pub fn is_safe_mode(&self) -> bool {
        *self.safe_mode.read()
    }

    /// Recover worktrees from a workspace.
    /// Scans ~/.ginger/worktrees/ for existing worktrees and reconciles their state.
    pub fn recover_worktrees(&self, worktrees_root: &PathBuf) -> Vec<RecoveryWorktree> {
        let mut recovered = Vec::new();

        if !worktrees_root.exists() {
            return recovered;
        }

        if let Ok(entries) = std::fs::read_dir(worktrees_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue; }

                // Check for .git file (worktree indicator)
                let git_file = path.join(".git");
                if !git_file.exists() { continue; }

                // Try to determine branch
                let branch = std::fs::read_to_string(git_file)
                    .ok()
                    .and_then(|content| {
                        content.lines()
                            .find(|l| l.starts_with("gitdir:"))
                            .and_then(|l| {
                                let gitdir = l.trim_start_matches("gitdir:").trim();
                                std::path::Path::new(gitdir)
                                    .file_name()
                                    .map(|f| f.to_string_lossy().to_string())
                            })
                    })
                    .unwrap_or_else(|| "unknown".into());

                // Check for uncommitted changes
                let has_changes = std::process::Command::new("git")
                    .args(&["status", "--porcelain"])
                    .current_dir(&path)
                    .output()
                    .map(|o| !o.stdout.is_empty())
                    .unwrap_or(false);

                let worktree = RecoveryWorktree {
                    path: path.display().to_string(),
                    branch,
                    status: if has_changes { "active".into() } else { "stale".into() },
                    has_unapplied_changes: has_changes,
                    owner_agent_id: None,
                };

                tracing::info!("Recovered worktree: {} (changes: {})", worktree.path, worktree.has_unapplied_changes);
                recovered.push(worktree);
            }
        }

        recovered
    }

    /// Full recovery flow — called on startup if a crash is detected.
    pub fn recover(&self, data_root: &PathBuf) -> RecoveryReport {
        let mut actions = Vec::new();
        let crashed_sessions = Vec::new(); // TODO: read from SQLite

        // Reconcile worktrees
        let worktrees_root = data_root.join("worktrees");
        let recovered_worktrees = self.recover_worktrees(&worktrees_root);

        if !recovered_worktrees.is_empty() {
            actions.push(RecoveryAction {
                action: "scan_worktrees".into(),
                target: worktrees_root.display().to_string(),
                result: format!("Found {} worktrees", recovered_worktrees.len()),
            });
        }

        // Check if we should enter safe mode
        let user_config = dirs::home_dir()
            .map(|h| h.join(".ginger").join("init.lua"))
            .map(|p| p.exists())
            .unwrap_or(false);

        let safe_mode = false; // Safe mode only if explicitly needed

        if safe_mode {
            self.enter_safe_mode();
            actions.push(RecoveryAction {
                action: "safe_mode".into(),
                target: "app".into(),
                result: "Safe mode enabled due to previous crash".into(),
            });
        }

        // Backup the database
        let db_path = data_root.join("data").join("ginger.sqlite");
        if db_path.exists() {
            let backup_dir = data_root.join("backups");
            let _ = std::fs::create_dir_all(&backup_dir);
            let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let backup = backup_dir.join(format!("recovery_{ts}.sqlite"));
            if std::fs::copy(&db_path, &backup).is_ok() {
                actions.push(RecoveryAction {
                    action: "backup_db".into(),
                    target: backup.display().to_string(),
                    result: "Database backed up before recovery".into(),
                });
            }
        }

        tracing::info!("Recovery complete: {} actions, {} worktrees, safe_mode={}", actions.len(), recovered_worktrees.len(), safe_mode);

        RecoveryReport {
            crashed_sessions,
            recovered_worktrees,
            actions,
            safe_mode,
        }
    }
}

impl Default for RecoveryService {
    fn default() -> Self { Self::new() }
}