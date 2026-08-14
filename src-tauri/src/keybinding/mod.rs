/// Ginger Code — Keybinding Conflict Detection (LLD 90)
/// Assigning a global Ginger shortcut checks existing Ginger mappings,
/// reserved macOS conflicts, and known Neovim conflicts. Warnings do not
/// block intentional overrides.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictKind {
    Ginger,
    MacOSReserved,
    Neovim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConflict {
    pub kind: ConflictKind,
    pub existing: String,
    pub note: String,
}

pub struct KeybindingChecker;

impl KeybindingChecker {
    /// Reserved macOS system shortcuts that Ginger should warn about.
    const MACOS_RESERVED: &'static [&'static str] = &[
        "cmd+space", "cmd+tab", "cmd+q", "cmd+h", "cmd+m", "cmd+w", "cmd+`",
    ];

    /// Known Neovim default mappings that could conflict.
    const NEOVIM_CONFLICTS: &'static [&'static str] = &[
        "ctrl+w", "ctrl+n", "ctrl+o", "ctrl+i", "ctrl+[", "ctrl+v",
    ];

    /// Check a proposed keybinding for conflicts.
    /// `existing_ginger` = list of keybindings already assigned in Ginger.
    pub fn check(&self, keybinding: &str, existing_ginger: &[String]) -> Vec<KeybindingConflict> {
        let mut conflicts = Vec::new();
        let lower = keybinding.to_lowercase();

        if existing_ginger.iter().any(|k| k.to_lowercase() == lower) {
            conflicts.push(KeybindingConflict {
                kind: ConflictKind::Ginger,
                existing: keybinding.to_string(),
                note: "Already assigned to another Ginger action".to_string(),
            });
        }

        if Self::MACOS_RESERVED.contains(&lower.as_str()) {
            conflicts.push(KeybindingConflict {
                kind: ConflictKind::MacOSReserved,
                existing: lower.clone(),
                note: "Reserved by macOS system shortcuts".to_string(),
            });
        }

        if Self::NEOVIM_CONFLICTS.contains(&lower.as_str()) {
            conflicts.push(KeybindingConflict {
                kind: ConflictKind::Neovim,
                existing: lower.clone(),
                note: "Known Neovim default mapping".to_string(),
            });
        }

        conflicts
    }
}

impl Default for KeybindingChecker {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_macos_reserved() {
        let conflicts = KeybindingChecker.check("cmd+space", &[]);
        assert!(conflicts.iter().any(|c| c.kind == ConflictKind::MacOSReserved));
    }

    #[test]
    fn detects_ginger_duplicate() {
        let conflicts = KeybindingChecker.check("cmd+p", &["cmd+p".to_string()]);
        assert!(conflicts.iter().any(|c| c.kind == ConflictKind::Ginger));
    }

    #[test]
    fn no_conflict_for_clean_binding() {
        let conflicts = KeybindingChecker.check("cmd+shift+k", &[]);
        assert!(conflicts.is_empty());
    }
}