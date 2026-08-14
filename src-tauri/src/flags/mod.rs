/// Ginger Code — Feature Flags (LLD 165)
/// Internal flags may gate unfinished functionality but should not become
/// permanent architecture. Release builds expose only intentional
/// experimental controls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagVisibility {
    Internal,
    Experimental,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub key: String,
    pub enabled: bool,
    pub visibility: FlagVisibility,
    pub description: String,
}

pub struct FlagRegistry {
    flags: Mutex<HashMap<String, FeatureFlag>>,
}

impl FlagRegistry {
    pub fn new() -> Self {
        Self {
            flags: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, key: &str, enabled: bool, visibility: FlagVisibility, description: &str) {
        self.flags.lock().unwrap().insert(
            key.to_string(),
            FeatureFlag {
                key: key.to_string(),
                enabled,
                visibility,
                description: description.to_string(),
            },
        );
    }

    pub fn is_enabled(&self, key: &str) -> bool {
        self.flags
            .lock()
            .unwrap()
            .get(key)
            .map(|f| f.enabled)
            .unwrap_or(false)
    }

    pub fn set(&self, key: &str, enabled: bool) {
        if let Some(f) = self.flags.lock().unwrap().get_mut(key) {
            f.enabled = enabled;
        }
    }

    /// In release builds, only experimental/stable flags are exposed.
    pub fn exposed(&self, is_release: bool) -> Vec<FeatureFlag> {
        self.flags
            .lock()
            .unwrap()
            .values()
            .filter(|f| !is_release || f.visibility != FlagVisibility::Internal)
            .cloned()
            .collect()
    }
}

impl Default for FlagRegistry {
    fn default() -> Self {
        Self::new()
    }
}