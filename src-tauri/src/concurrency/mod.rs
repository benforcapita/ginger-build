/// Ginger Code — Concurrency Model (LLD 223)
/// Use per-resource locks: repository mutation, workspace runtime resolution,
/// package artifact materialization, and DB transactions. Read-only operations
/// stay concurrent.

use std::collections::HashMap;
use std::sync::Mutex;

/// A per-resource lock registry. Each named resource has an exclusive lock.
/// Read-only operations do not acquire locks and stay concurrent.
pub struct ResourceLocks {
    locks: Mutex<HashMap<String, ()>>,
}

impl ResourceLocks {
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
        }
    }

    /// Acquire an exclusive lock for a resource. Returns false if held.
    pub fn acquire(&self, resource: &str) -> bool {
        let mut locks = self.locks.lock().unwrap();
        if locks.contains_key(resource) {
            false
        } else {
            locks.insert(resource.to_string(), ());
            true
        }
    }

    pub fn release(&self, resource: &str) {
        self.locks.lock().unwrap().remove(resource);
    }

    pub fn is_locked(&self, resource: &str) -> bool {
        self.locks.lock().unwrap().contains_key(resource)
    }
}

impl Default for ResourceLocks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_resource_exclusivity() {
        let locks = ResourceLocks::new();
        assert!(locks.acquire("repo:main"));
        assert!(!locks.acquire("repo:main"));
        // Different resource is independent
        assert!(locks.acquire("repo:other"));
        locks.release("repo:main");
        assert!(locks.acquire("repo:main"));
    }
}