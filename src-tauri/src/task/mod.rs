/// Ginger Code — Task Model (LLD 93-94)
/// Task is independent of agent. One task may have multiple implementation
/// agents, reviewers, verification runs, and an applied result.
/// v0.1 task management remains lightweight.

use crate::types::TaskId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Active,
    Review,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub status: TaskStatus,
    pub agent_ids: Vec<i64>,
    pub reviewer_ids: Vec<i64>,
    pub verification_run_ids: Vec<i64>,
    pub applied_result: Option<String>,
    pub created_at: u64,
}

pub struct TaskStore {
    tasks: Mutex<HashMap<TaskId, Task>>,
    next_id: Mutex<i64>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn create(&self, title: &str) -> TaskId {
        let mut next = self.next_id.lock().unwrap();
        let id = TaskId::new(*next);
        *next += 1;

        let task = Task {
            id,
            title: title.to_string(),
            status: TaskStatus::Pending,
            agent_ids: Vec::new(),
            reviewer_ids: Vec::new(),
            verification_run_ids: Vec::new(),
            applied_result: None,
            created_at: now(),
        };
        self.tasks.lock().unwrap().insert(id, task);
        id
    }

    pub fn add_agent(&self, id: TaskId, agent_id: i64) {
        if let Some(t) = self.tasks.lock().unwrap().get_mut(&id) {
            if !t.agent_ids.contains(&agent_id) {
                t.agent_ids.push(agent_id);
            }
            t.status = TaskStatus::Active;
        }
    }

    pub fn add_reviewer(&self, id: TaskId, reviewer_id: i64) {
        if let Some(t) = self.tasks.lock().unwrap().get_mut(&id) {
            if !t.reviewer_ids.contains(&reviewer_id) {
                t.reviewer_ids.push(reviewer_id);
            }
            t.status = TaskStatus::Review;
        }
    }

    pub fn set_status(&self, id: TaskId, status: TaskStatus) {
        if let Some(t) = self.tasks.lock().unwrap().get_mut(&id) {
            t.status = status;
        }
    }

    pub fn get(&self, id: TaskId) -> Option<Task> {
        self.tasks.lock().unwrap().get(&id).cloned()
    }

    pub fn list(&self) -> Vec<Task> {
        self.tasks.lock().unwrap().values().cloned().collect()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}