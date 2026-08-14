/// Ginger Code — Review Session
/// A review session stores agent thread, base/target revisions, diff
/// fingerprint, accepted/rejected hunks, and status. It is never silently
/// applied against a changed fingerprint.

use crate::types::ReviewSessionId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    Open,
    Reviewing,
    Applying,
    Applied,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSession {
    pub id: ReviewSessionId,
    pub agent_id: i64,
    pub base_revision: String,
    pub target_revision: String,
    pub diff_fingerprint: String,
    pub status: ReviewStatus,
    pub accepted_hunks: Vec<String>,
    pub rejected_hunks: Vec<String>,
}

pub struct ReviewStore {
    sessions: Mutex<HashMap<ReviewSessionId, ReviewSession>>,
    next_id: Mutex<i64>,
}

impl ReviewStore {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            next_id: Mutex::new(1),
        }
    }

    pub fn create(
        &self,
        agent_id: i64,
        base: &str,
        target: &str,
        fingerprint: &str,
    ) -> ReviewSessionId {
        let mut next = self.next_id.lock().unwrap();
        let id = ReviewSessionId::new(*next);
        *next += 1;

        let session = ReviewSession {
            id,
            agent_id,
            base_revision: base.to_string(),
            target_revision: target.to_string(),
            diff_fingerprint: fingerprint.to_string(),
            status: ReviewStatus::Open,
            accepted_hunks: Vec::new(),
            rejected_hunks: Vec::new(),
        };
        self.sessions.lock().unwrap().insert(id, session);
        id
    }

    pub fn accept_hunk(&self, id: ReviewSessionId, hunk_id: &str) {
        if let Some(s) = self.sessions.lock().unwrap().get_mut(&id) {
            if !s.accepted_hunks.contains(&hunk_id.to_string()) {
                s.accepted_hunks.push(hunk_id.to_string());
            }
        }
    }

    pub fn reject_hunk(&self, id: ReviewSessionId, hunk_id: &str) {
        if let Some(s) = self.sessions.lock().unwrap().get_mut(&id) {
            if !s.rejected_hunks.contains(&hunk_id.to_string()) {
                s.rejected_hunks.push(hunk_id.to_string());
            }
        }
    }

    pub fn set_status(&self, id: ReviewSessionId, status: ReviewStatus) {
        if let Some(s) = self.sessions.lock().unwrap().get_mut(&id) {
            s.status = status;
        }
    }

    pub fn get(&self, id: ReviewSessionId) -> Option<ReviewSession> {
        self.sessions.lock().unwrap().get(&id).cloned()
    }

    /// Fingerprint must match before applying. Returns false if stale.
    pub fn fingerprint_matches(&self, id: ReviewSessionId, fingerprint: &str) -> bool {
        self.sessions
            .lock()
            .unwrap()
            .get(&id)
            .map(|s| s.diff_fingerprint == fingerprint)
            .unwrap_or(false)
    }
}

impl Default for ReviewStore {
    fn default() -> Self {
        Self::new()
    }
}