// Ginger Code — Ginger Presence (stub for v0.1 slice 1)
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GingerState {
    Idle,
    Listening,
    Thinking,
    Coding,
    Testing,
    Reviewing,
    Success,
    Warning,
    Failure,
}

impl Default for GingerState {
    fn default() -> Self { Self::Idle }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GingerConfig {
    pub personality: Personality,
    pub commentary: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Personality {
    Standard,
    Quiet,
    Extra,
}

impl Default for GingerConfig {
    fn default() -> Self {
        Self { personality: Personality::Standard, commentary: true }
    }
}

pub struct GingerPresence {
    state: RwLock<GingerState>,
    config: RwLock<GingerConfig>,
}

impl GingerPresence {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(GingerState::Idle),
            config: RwLock::new(GingerConfig::default()),
        }
    }

    pub fn state(&self) -> GingerState { *self.state.read() }
    pub fn set_state(&self, s: GingerState) { *self.state.write() = s; }
    pub fn config(&self) -> GingerConfig { self.config.read().clone() }
}

impl Default for GingerPresence { fn default() -> Self { Self::new() } }