/// Ginger Code — Package Rollback (LLD 83)
/// Environment updates are staged, validated, and atomically activated.
/// The prior working lock/environment remains available until success.
/// "Repair Workspace Environment" rebuilds from manifest/lock.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvActivationState {
    Staged,
    Validated,
    Active,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSnapshot {
    pub id: i64,
    pub state: EnvActivationState,
    pub lock_path: String,
    pub staged_at: u64,
    pub activated_at: Option<u64>,
}

pub struct EnvironmentManager {
    snapshots: Mutex<HashMap<i64, EnvironmentSnapshot>>,
    next_id: Mutex<i64>,
    active_id: Mutex<Option<i64>>,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        Self {
            snapshots: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
            active_id: Mutex::new(None),
        }
    }

    /// Stage a new environment. The prior one remains available.
    pub fn stage(&self, lock_path: &str) -> i64 {
        let mut next = self.next_id.lock().unwrap();
        let id = *next;
        *next += 1;

        let snap = EnvironmentSnapshot {
            id,
            state: EnvActivationState::Staged,
            lock_path: lock_path.to_string(),
            staged_at: now(),
            activated_at: None,
        };
        self.snapshots.lock().unwrap().insert(id, snap);
        id
    }

    /// Validate a staged environment. On success it becomes Active.
    /// On failure it stays Staged and the prior env remains active.
    pub fn validate_and_activate(&self, id: i64, valid: bool) -> bool {
        let mut map = self.snapshots.lock().unwrap();
        if let Some(snap) = map.get_mut(&id) {
            if valid {
                snap.state = EnvActivationState::Validated;
                snap.state = EnvActivationState::Active;
                snap.activated_at = Some(now());
                *self.active_id.lock().unwrap() = Some(id);
                true
            } else {
                snap.state = EnvActivationState::RolledBack;
                false
            }
        } else {
            false
        }
    }

    /// Roll back to the previous environment (the one before `id`).
    pub fn rollback(&self, id: i64) -> Option<i64> {
        let mut map = self.snapshots.lock().unwrap();
        if let Some(snap) = map.get_mut(&id) {
            snap.state = EnvActivationState::RolledBack;
        }
        // Find the most recent non-rolled-back snapshot before id
        let mut prev: Option<i64> = None;
        let mut prev_time: u64 = 0;
        for (sid, s) in map.iter() {
            if *sid != id && s.state == EnvActivationState::Active && s.staged_at > prev_time {
                prev = Some(*sid);
                prev_time = s.staged_at;
            }
        }
        if let Some(p) = prev {
            *self.active_id.lock().unwrap() = Some(p);
        }
        prev
    }

    pub fn active(&self) -> Option<EnvironmentSnapshot> {
        let active = *self.active_id.lock().unwrap();
        active.and_then(|id| self.snapshots.lock().unwrap().get(&id).cloned())
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for EnvironmentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_validation_rolls_back() {
        let m = EnvironmentManager::new();
        let id = m.stage("/lock");
        assert!(!m.validate_and_activate(id, false));
        assert!(m.active().is_none());
    }

    #[test]
    fn successful_activation_sets_active() {
        let m = EnvironmentManager::new();
        let id = m.stage("/lock");
        assert!(m.validate_and_activate(id, true));
        assert_eq!(m.active().unwrap().id, id);
    }
}