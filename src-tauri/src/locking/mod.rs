/// Ginger Code — File Locking (LLD 224)
/// Artifact downloads/materialization use locking. Partial artifacts never
/// appear at final cache keys.

use std::collections::HashMap;
use std::sync::Mutex;

pub struct LockManager {
    locks: Mutex<HashMap<String, ()>>,
}

impl LockManager {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Try to acquire a lock for `key`. Returns true if acquired.
    pub fn try_lock(&self, key: &str) -> bool {
        let mut locks = self.locks.lock().unwrap();
        if locks.contains_key(key) {
            false
        } else {
            locks.insert(key.to_string(), ());
            true
        }
    }

    /// Release a lock for `key`.
    pub fn unlock(&self, key: &str) {
        self.locks.lock().unwrap().remove(key);
    }

    pub fn is_locked(&self, key: &str) -> bool {
        self.locks.lock().unwrap().contains_key(key)
    }
}

impl Default for LockManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_is_exclusive() {
        let m = LockManager::new();
        assert!(m.try_lock("artifact-a"));
        assert!(!m.try_lock("artifact-a"));
        m.unlock("artifact-a");
        assert!(m.try_lock("artifact-a"));
    }
}