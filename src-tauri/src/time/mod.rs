/// Ginger Code — Time Handling (LLD 218)
/// Persist UTC timestamps; render local time. Use monotonic clocks for
/// active durations where practical.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Current UTC unix timestamp (seconds).
pub fn utc_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A monotonic stopwatch for measuring active durations.
/// Not affected by wall-clock changes.
pub struct Stopwatch {
    start: Instant,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Elapsed duration since start.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Elapsed milliseconds since start.
    pub fn elapsed_ms(&self) -> u128 {
        self.elapsed().as_millis()
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopwatch_measures_elapsed() {
        let sw = Stopwatch::start();
        std::thread::sleep(Duration::from_millis(5));
        assert!(sw.elapsed_ms() >= 5);
    }
}