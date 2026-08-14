/// Ginger Code — Logging Correlation IDs (LLD 220)
/// Workspace sessions, agent threads, jobs, installs, and verification runs
/// carry correlation IDs through logs.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CORRELATION: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub String);

impl CorrelationId {
    pub fn new() -> Self {
        let n = NEXT_CORRELATION.fetch_add(1, Ordering::Relaxed);
        Self(format!("corr-{:x}", n))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A log line carrying a correlation ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelatedLog {
    pub correlation_id: CorrelationId,
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

pub struct CorrelatedLogger;

impl CorrelatedLogger {
    pub fn log(correlation_id: &CorrelationId, level: &str, message: &str) -> CorrelatedLog {
        let entry = CorrelatedLog {
            correlation_id: correlation_id.clone(),
            level: level.to_string(),
            message: message.to_string(),
            timestamp: now(),
        };
        // In a real impl this would write to the structured log sink.
        eprintln!("[{}] {} {}", correlation_id, level, message);
        entry
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correlation_ids_are_unique() {
        let a = CorrelationId::new();
        let b = CorrelationId::new();
        assert_ne!(a, b);
    }
}