/// Ginger Code — Apply Record
/// Every Ginger-mediated apply records source agent/task, target branch,
/// pre/post head, strategy, and timestamp for audit and constrained undo.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyStrategy {
    Patch,
    CherryPick,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyRecord {
    pub id: i64,
    pub agent_id: i64,
    pub task_id: Option<i64>,
    pub target_branch: String,
    pub pre_head: String,
    pub post_head: String,
    pub strategy: ApplyStrategy,
    pub timestamp: u64,
}

pub struct ApplyLog {
    records: Mutex<HashMap<i64, ApplyRecord>>,
    next_id: Mutex<i64>,
}

impl ApplyLog {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn record(
        &self,
        agent_id: i64,
        task_id: Option<i64>,
        target_branch: &str,
        pre_head: &str,
        post_head: &str,
        strategy: ApplyStrategy,
    ) -> i64 {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;

        let rec = ApplyRecord {
            id,
            agent_id,
            task_id,
            target_branch: target_branch.to_string(),
            pre_head: pre_head.to_string(),
            post_head: post_head.to_string(),
            strategy,
            timestamp: now(),
        };
        self.records.lock().unwrap().insert(id, rec);
        id
    }

    pub fn get(&self, id: i64) -> Option<ApplyRecord> {
        self.records.lock().unwrap().get(&id).cloned()
    }

    pub fn list(&self) -> Vec<ApplyRecord> {
        let mut all: Vec<ApplyRecord> = self.records.lock().unwrap().values().cloned().collect();
        all.sort_by_key(|r| std::cmp::Reverse(r.timestamp));
        all
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for ApplyLog {
    fn default() -> Self {
        Self::new()
    }
}