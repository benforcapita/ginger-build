/// Ginger Code — Cancellation & Progress Events (LLD 221-222)
/// Project scan, package download before activation, verification, palette
/// search, and runtime validation are cancellable. Git apply, DB migration,
/// and atomic environment switches complete or rollback safely before
/// reporting cancellation.
/// Long operations emit operation ID, phase, completed/total when known,
/// and message. UI renders progress generically.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub operation_id: String,
    pub phase: String,
    pub completed: usize,
    pub total: usize,
    pub message: String,
}

/// A cancellable operation token. Checked cooperatively by long-running work.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// A progress reporter that emits ProgressUpdate events.
pub struct ProgressReporter {
    operation_id: String,
}

impl ProgressReporter {
    pub fn new(operation_id: &str) -> Self {
        Self {
            operation_id: operation_id.to_string(),
        }
    }

    pub fn report(&self, phase: &str, completed: usize, total: usize, message: &str) -> ProgressUpdate {
        ProgressUpdate {
            operation_id: self.operation_id.clone(),
            phase: phase.to_string(),
            completed,
            total,
            message: message.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_token_works() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn progress_reporter_emits_update() {
        let reporter = ProgressReporter::new("op-1");
        let update = reporter.report("scanning", 5, 10, "Scanning files");
        assert_eq!(update.operation_id, "op-1");
        assert_eq!(update.completed, 5);
        assert_eq!(update.total, 10);
    }
}