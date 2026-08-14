/// Ginger Code — Cache Management (LLD 84)
/// The shared cache supports size reporting, verification, pruning unused
/// artifacts, and repair. Artifacts referenced by active workspace locks
/// are never removed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheArtifact {
    pub key: String,
    pub size_bytes: u64,
    pub referenced_by_lock: bool,
}

pub struct CacheManager {
    artifacts: Mutex<HashMap<String, CacheArtifact>>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            artifacts: Mutex::new(HashMap::new()),
        }
    }

    pub fn add(&self, key: &str, size_bytes: u64, referenced: bool) {
        self.artifacts.lock().unwrap().insert(
            key.to_string(),
            CacheArtifact {
                key: key.to_string(),
                size_bytes,
                referenced_by_lock: referenced,
            },
        );
    }

    pub fn total_size(&self) -> u64 {
        self.artifacts
            .lock()
            .unwrap()
            .values()
            .map(|a| a.size_bytes)
            .sum()
    }

    /// Prune unused artifacts. Never removes lock-referenced artifacts.
    pub fn prune(&self) -> Vec<String> {
        let mut removed = Vec::new();
        let mut map = self.artifacts.lock().unwrap();
        let keys: Vec<String> = map.keys().cloned().collect();
        for key in keys {
            if let Some(a) = map.get(&key) {
                if !a.referenced_by_lock {
                    removed.push(key.clone());
                    map.remove(&key);
                }
            }
        }
        removed
    }

    pub fn list(&self) -> Vec<CacheArtifact> {
        self.artifacts.lock().unwrap().values().cloned().collect()
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new()
    }
}