/// Ginger Code — Package Trust Levels & Install Plan (LLD 81-82)
/// Badges: Core, Curated, Community/Custom, Local. Curated means Ginger owns
/// metadata and install rules; it is not a security certification.
/// Before installation, Ginger computes and displays the install plan.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageTrust {
    Core,
    Curated,
    Community,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPlanItem {
    pub package_id: String,
    pub version: String,
    pub source: String,
    pub command: String,
    pub environment_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPlan {
    pub items: Vec<InstallPlanItem>,
    pub custom_commands_explicit: bool,
}

pub struct PackagePlanner;

impl PackagePlanner {
    /// Build an install plan. Custom installs show executable commands explicitly.
    pub fn plan(&self, items: Vec<InstallPlanItem>, has_custom: bool) -> InstallPlan {
        InstallPlan {
            items,
            custom_commands_explicit: has_custom,
        }
    }
}

impl Default for PackagePlanner {
    fn default() -> Self {
        Self
    }
}