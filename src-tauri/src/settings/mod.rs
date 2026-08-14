/// Ginger Code — Settings Storage
/// Settings layers: defaults → global user → workspace user → safe project-shared config.
/// Effective values and source are inspectable.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsSource {
    Defaults,
    GlobalUser,
    WorkspaceUser,
    ProjectShared,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingValue {
    pub value: String,
    pub source: SettingsSource,
}

pub struct SettingsStore {
    layers: HashMap<SettingsSource, HashMap<String, String>>,
}

impl SettingsStore {
    pub fn new() -> Self {
        let mut layers = HashMap::new();
        layers.insert(SettingsSource::Defaults, HashMap::new());
        layers.insert(SettingsSource::GlobalUser, HashMap::new());
        layers.insert(SettingsSource::WorkspaceUser, HashMap::new());
        layers.insert(SettingsSource::ProjectShared, HashMap::new());
        Self { layers }
    }

    pub fn set(&mut self, source: SettingsSource, key: &str, value: &str) {
        self.layers
            .entry(source)
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    /// Effective value: highest-priority layer wins.
    /// Priority: ProjectShared > WorkspaceUser > GlobalUser > Defaults.
    pub fn get(&self, key: &str) -> Option<SettingValue> {
        let order = [
            SettingsSource::ProjectShared,
            SettingsSource::WorkspaceUser,
            SettingsSource::GlobalUser,
            SettingsSource::Defaults,
        ];
        for source in order {
            if let Some(v) = self.layers.get(&source).and_then(|m| m.get(key)) {
                return Some(SettingValue {
                    value: v.clone(),
                    source,
                });
            }
        }
        None
    }

    /// All effective settings with their source, for inspection.
    pub fn effective(&self) -> Vec<SettingValue> {
        let mut keys: Vec<String> = Vec::new();
        for layer in self.layers.values() {
            for k in layer.keys() {
                if !keys.contains(k) {
                    keys.push(k.clone());
                }
            }
        }
        keys.iter()
            .filter_map(|k| self.get(k))
            .collect()
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new()
    }
}