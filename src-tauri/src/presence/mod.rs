// Ginger Code — Ginger Presence Layer (expanded from stub)
// Ginger is a UI/presence subsystem, not the business-logic owner.
// States: idle, listening, thinking, coding, testing, reviewing, success, warning, failure.
// Asset tiers: full ANSI portrait, medium portrait, compact status glyph.
// Copy rules: sarcasm targets bugs/tests/deps; never insults user; serious states override humor.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

impl GingerState {
    pub fn is_serious(&self) -> bool {
        matches!(self, Self::Failure | Self::Warning)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Personality {
    Standard,
    Quiet,
    Extra,
}

impl Default for Personality {
    fn default() -> Self { Self::Standard }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GingerConfig {
    pub personality: Personality,
    pub commentary: bool,
}

impl Default for GingerConfig {
    fn default() -> Self {
        Self { personality: Personality::Standard, commentary: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GingerMessage {
    pub text: String,
    pub tier: MessageTier,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageTier {
    Full,    // welcome, major result, empty agent dock
    Medium,  // package recommendations, task/recovery screens
    Compact, // status bar, palette, notifications
}

pub struct GingerPresence {
    state: RwLock<GingerState>,
    config: RwLock<GingerConfig>,
    messages: HashMap<GingerState, Vec<String>>,
}

impl GingerPresence {
    pub fn new() -> Self {
        let mut messages = HashMap::new();

        messages.insert(GingerState::Idle, vec![
            "Just sitting here. Not like I have anything better to do.".into(),
            "Waiting for something to happen. Like always.".into(),
            "Idle hands are the devil's workshop. Good thing I don't have hands.".into(),
        ]);

        messages.insert(GingerState::Coding, vec![
            "Oh good, more code to review. My favorite.".into(),
            "Writing code? Let me watch and judge silently.".into(),
            "I could write this faster. Just saying.".into(),
        ]);

        messages.insert(GingerState::Testing, vec![
            "Running tests... because hope is not a strategy.".into(),
            "Tests are passing. For now. Enjoy it while it lasts.".into(),
            "Ah yes, tests. The things we write to feel less paranoid.".into(),
        ]);

        messages.insert(GingerState::Reviewing, vec![
            "Reviewing changes. I promise to be... mostly fair.".into(),
            "Let's see what mess the agent made this time.".into(),
        ]);

        messages.insert(GingerState::Success, vec![
            "It worked. I'm as surprised as you are.".into(),
            "Success! Don't get used to it.".into(),
            "Nice. That actually worked.".into(),
        ]);

        messages.insert(GingerState::Warning, vec![
            "Something's not quite right. You might want to check that.".into(),
            "Heads up — things look a bit off.".into(),
        ]);

        messages.insert(GingerState::Failure, vec![
            "Well, that didn't work.".into(),
            "Build failed. Shocker.".into(),
            "Something broke. I'd say I'm shocked, but I'm an AI.".into(),
        ]);

        messages.insert(GingerState::Thinking, vec![
            "Processing... give me a moment.".into(),
            "Thinking. Don't rush me.".into(),
        ]);

        messages.insert(GingerState::Listening, vec![
            "I'm listening. Mostly.".into(),
            "Go ahead, I'm paying attention.".into(),
        ]);

        Self {
            state: RwLock::new(GingerState::Idle),
            config: RwLock::new(GingerConfig::default()),
            messages,
        }
    }

    pub fn state(&self) -> GingerState { *self.state.read() }

    pub fn set_state(&self, s: GingerState) {
        *self.state.write() = s;
    }

    pub fn config(&self) -> GingerConfig { self.config.read().clone() }

    pub fn set_config(&self, cfg: GingerConfig) {
        *self.config.write() = cfg;
    }

    /// Get a message for the current state, respecting config.
    pub fn message(&self) -> Option<GingerMessage> {
        let cfg = self.config.read();
        if !cfg.commentary {
            return None;
        }

        let state = self.state.read();
        let pool = self.messages.get(&state)?;

        // Quiet personality: only show messages for serious states
        if cfg.personality == Personality::Quiet && !state.is_serious() {
            return None;
        }

        let idx = (chrono::Utc::now().timestamp() as usize) % pool.len();
        let text = pool[idx].clone();

        let tier = match state {
            GingerState::Idle | GingerState::Success | GingerState::Failure => MessageTier::Full,
            GingerState::Coding | GingerState::Testing | GingerState::Reviewing => MessageTier::Medium,
            GingerState::Thinking | GingerState::Listening | GingerState::Warning => MessageTier::Compact,
        };

        Some(GingerMessage { text, tier })
    }

    /// Toggle commentary on/off.
    pub fn toggle_commentary(&self) {
        let mut cfg = self.config.write();
        cfg.commentary = !cfg.commentary;
    }

    /// Cycle personality: Standard → Quiet → Extra → Standard.
    pub fn cycle_personality(&self) {
        let mut cfg = self.config.write();
        cfg.personality = match cfg.personality {
            Personality::Standard => Personality::Quiet,
            Personality::Quiet => Personality::Extra,
            Personality::Extra => Personality::Standard,
        };
    }
}

impl Default for GingerPresence {
    fn default() -> Self { Self::new() }
}