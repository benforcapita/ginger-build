/// Ginger Code — File Watcher (LLD 150)
/// Watcher normalizes/debounces file create/delete/rename, relevant manifest
/// changes, recommendation evidence changes, and worktree changes.
/// It never auto-installs from a manifest update.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchEventKind {
    Created,
    Deleted,
    Renamed,
    Modified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    pub path: String,
    pub kind: WatchEventKind,
    pub timestamp: u64,
}

pub struct DebouncedWatcher {
    pending: Mutex<HashMap<String, (WatchEvent, Instant)>>,
    debounce_ms: u64,
}

impl DebouncedWatcher {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            debounce_ms,
        }
    }

    /// Record an event. Returns the debounced event if it's ready to emit.
    pub fn record(&self, path: &str, kind: WatchEventKind) -> Option<WatchEvent> {
        let now = Instant::now();
        let mut map = self.pending.lock().unwrap();
        let key = path.to_string();
        let evt = WatchEvent {
            path: path.to_string(),
            kind,
            timestamp: unix_now(),
        };
        map.insert(key.clone(), (evt, now));

        // Check if the oldest pending event for this path is past debounce.
        if let Some((_, first)) = map.get(&key) {
            if now.duration_since(*first) >= Duration::from_millis(self.debounce_ms) {
                if let Some((evt, _)) = map.remove(&key) {
                    return Some(evt);
                }
            }
        }
        None
    }

    /// Flush all pending events (e.g. on shutdown).
    pub fn flush(&self) -> Vec<WatchEvent> {
        let mut map = self.pending.lock().unwrap();
        let events: Vec<WatchEvent> = map.drain().map(|(_, (e, _))| e).collect();
        events
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for DebouncedWatcher {
    fn default() -> Self {
        Self::new(200)
    }
}