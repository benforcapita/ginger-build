/// Ginger Code — Project .ginger Configuration
/// Optional .ginger/workspace.toml may declare verification commands,
/// recommended package IDs, preferred agent templates, and display metadata.
/// It may not silently execute startup commands, store secrets, or trigger
/// destructive Git operations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GingerWorkspaceConfig {
    pub verification: Option<VerificationConfig>,
    pub recommended_packages: Vec<String>,
    pub agent_templates: Vec<AgentTemplate>,
    pub display: Option<DisplayMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationConfig {
    pub commands: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl GingerWorkspaceConfig {
    /// Parse a .ginger/workspace.toml string. Returns None on parse error.
    pub fn parse(toml: &str) -> Option<Self> {
        toml::from_str(toml).ok()
    }
}