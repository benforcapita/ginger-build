/// Ginger Code — Agent Concurrency Policy & Queue (LLD 159-160)
/// Default max active coding agents is 3. At capacity, users can queue,
/// stop an agent, or change the limit. Queued starts are durable. By default,
/// when capacity frees Ginger prompts before starting the next queued coding
/// task unless auto-start is enabled.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Mutex;

pub const DEFAULT_MAX_ACTIVE_AGENTS: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedAgentStart {
    pub agent_id: i64,
    pub task_id: Option<i64>,
    pub queued_at: u64,
}

pub struct AgentScheduler {
    max_active: usize,
    active: Mutex<Vec<i64>>,
    queue: Mutex<VecDeque<QueuedAgentStart>>,
    auto_start: Mutex<bool>,
}

impl AgentScheduler {
    pub fn new() -> Self {
        Self {
            max_active: DEFAULT_MAX_ACTIVE_AGENTS,
            active: Mutex::new(Vec::new()),
            queue: Mutex::new(VecDeque::new()),
            auto_start: Mutex::new(false),
        }
    }

    pub fn set_max_active(&mut self, n: usize) {
        self.max_active = n;
    }

    pub fn set_auto_start(&self, enabled: bool) {
        *self.auto_start.lock().unwrap() = enabled;
    }

    /// Try to start an agent. Returns Ok if started, Err(queued) if at capacity.
    pub fn try_start(&self, agent_id: i64, task_id: Option<i64>) -> Result<(), QueuedAgentStart> {
        let mut active = self.active.lock().unwrap();
        if active.len() < self.max_active {
            active.push(agent_id);
            Ok(())
        } else {
            let queued = QueuedAgentStart {
                agent_id,
                task_id,
                queued_at: now(),
            };
            self.queue.lock().unwrap().push_back(queued);
            Err(queued)
        }
    }

    /// Mark an agent as finished, freeing a slot.
    /// Returns the next queued agent to start, if any and auto-start enabled.
    pub fn finish(&self, agent_id: i64) -> Option<QueuedAgentStart> {
        let mut active = self.active.lock().unwrap();
        active.retain(|a| *a != agent_id);

        if *self.auto_start.lock().unwrap() {
            self.queue.lock().unwrap().pop_front()
        } else {
            None
        }
    }

    pub fn active_count(&self) -> usize {
        self.active.lock().unwrap().len()
    }

    pub fn queued(&self) -> Vec<QueuedAgentStart> {
        self.queue.lock().unwrap().iter().cloned().collect()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for AgentScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queues_when_at_capacity() {
        let s = AgentScheduler::new();
        s.set_max_active(1);
        assert!(s.try_start(1, None).is_ok());
        assert!(s.try_start(2, None).is_err());
        assert_eq!(s.queued().len(), 1);
    }

    #[test]
    fn frees_slot_on_finish() {
        let s = AgentScheduler::new();
        s.set_max_active(1);
        s.set_auto_start(true);
        s.try_start(1, None).unwrap();
        s.try_start(2, None).unwrap_err();
        let next = s.finish(1);
        assert!(next.is_some());
        assert_eq!(next.unwrap().agent_id, 2);
    }
}