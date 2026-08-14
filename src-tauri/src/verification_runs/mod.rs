/// Ginger Code — Verification Runs
/// Verification runs are durable objects with commands, worktree, status,
/// timestamps, exit codes, and output locations. Material diff changes
/// invalidate prior verification.

use crate::types::VerificationRunId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRun {
    pub id: VerificationRunId,
    pub agent_id: i64,
    pub worktree: String,
    pub command: String,
    pub status: VerificationStatus,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub exit_code: Option<i32>,
    pub output_path: Option<String>,
    pub diff_fingerprint: String,
}

pub struct VerificationStore {
    runs: Mutex<HashMap<VerificationRunId, VerificationRun>>,
    next_id: Mutex<i64>,
}

impl VerificationStore {
    pub fn new() -> Self {
        Self {
            runs: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn start(&self, agent_id: i64, worktree: &str, command: &str, fingerprint: &str) -> VerificationRunId {
        let mut next = self.next_id.lock().unwrap();
        let id = VerificationRunId::new(*next);
        *next += 1;

        let run = VerificationRun {
            id,
            agent_id,
            worktree: worktree.to_string(),
            command: command.to_string(),
            status: VerificationStatus::Running,
            started_at: now(),
            finished_at: None,
            exit_code: None,
            output_path: None,
            diff_fingerprint: fingerprint.to_string(),
        };
        self.runs.lock().unwrap().insert(id, run);
        id
    }

    pub fn complete(&self, id: VerificationRunId, exit_code: i32, output_path: &str) {
        if let Some(run) = self.runs.lock().unwrap().get_mut(&id) {
            run.status = if exit_code == 0 {
                VerificationStatus::Completed
            } else {
                VerificationStatus::Failed
            };
            run.exit_code = Some(exit_code);
            run.finished_at = Some(now());
            run.output_path = Some(output_path.to_string());
        }
    }

    pub fn get(&self, id: VerificationRunId) -> Option<VerificationRun> {
        self.runs.lock().unwrap().get(&id).cloned()
    }

    pub fn list_for_agent(&self, agent_id: i64) -> Vec<VerificationRun> {
        self.runs
            .lock()
            .unwrap()
            .values()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for VerificationStore {
    fn default() -> Self {
        Self::new()
    }
}