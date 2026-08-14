/// Ginger Code — Process Supervisor
/// All child processes launched by Ginger are registered here.
/// No subsystem spawns unmanaged processes directly.

use crate::types::ProcessId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessCategory {
    Editor,
    Terminal,
    Agent,
    PackageTool,
    Verification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: ProcessId,
    pub category: ProcessCategory,
    pub label: String,
    pub pid: u32,
    pub started_at: u64,
    pub exit_code: Option<i32>,
    pub abnormal_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequest {
    pub category: ProcessCategory,
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: Vec<(String, String)>,
}

pub struct ProcessSupervisor {
    processes: Mutex<HashMap<ProcessId, ProcessInfo>>,
    next_id: Mutex<i64>,
}

impl ProcessSupervisor {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn register(&self, req: &SpawnRequest, pid: u32) -> ProcessId {
        let mut next = self.next_id.lock().unwrap();
        let id = ProcessId::new(*next);
        *next += 1;

        let info = ProcessInfo {
            id,
            category: req.category,
            label: req.label.clone(),
            pid,
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            exit_code: None,
            abnormal_exit: false,
        };

        self.processes.lock().unwrap().insert(id, info);
        id
    }

    pub fn mark_exit(&self, id: ProcessId, exit_code: Option<i32>) {
        if let Some(p) = self.processes.lock().unwrap().get_mut(&id) {
            p.exit_code = exit_code;
            p.abnormal_exit = exit_code.map(|c| c != 0).unwrap_or(true);
        }
    }

    pub fn list(&self) -> Vec<ProcessInfo> {
        self.processes.lock().unwrap().values().cloned().collect()
    }

    pub fn get(&self, id: ProcessId) -> Option<ProcessInfo> {
        self.processes.lock().unwrap().get(&id).cloned()
    }

    /// Reconcile orphans: processes that exited but were never marked.
    pub fn reconcile_orphans(&self) -> Vec<ProcessInfo> {
        let mut orphans = Vec::new();
        let map = self.processes.lock().unwrap();
        for info in map.values() {
            if info.exit_code.is_none() {
                orphans.push(info.clone());
            }
        }
        orphans
    }
}

impl Default for ProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}