/// Ginger Code — Version Compatibility Matrix (LLD 169)
/// Maintain explicit compatibility between App, Runtime, DB schema, IPC
/// protocol, and Catalog versions. Do not infer all compatibility only
/// from semver.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionSet {
    pub app: String,
    pub runtime: String,
    pub db_schema: i32,
    pub ipc_protocol: u32,
    pub catalog: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityCheck {
    pub compatible: bool,
    pub mismatches: Vec<String>,
}

pub struct CompatibilityMatrix {
    /// Known-good version sets. A set is compatible if it matches a known row.
    known_sets: Vec<VersionSet>,
}

impl CompatibilityMatrix {
    pub fn new() -> Self {
        Self {
            known_sets: vec![VersionSet {
                app: "0.1.0".to_string(),
                runtime: "0.1.0".to_string(),
                db_schema: 1,
                ipc_protocol: 1,
                catalog: "baseline-1".to_string(),
            }],
        }
    }

    /// Check a version set against known-good combinations.
    pub fn check(&self, set: &VersionSet) -> CompatibilityCheck {
        let mut mismatches = Vec::new();
        let mut compatible = false;

        for known in &self.known_sets {
            if known.app == set.app
                && known.runtime == set.runtime
                && known.db_schema == set.db_schema
                && known.ipc_protocol == set.ipc_protocol
                && known.catalog == set.catalog
            {
                compatible = true;
                break;
            }
        }

        if !compatible {
            // Report which dimensions differ from the baseline
            let base = &self.known_sets[0];
            if base.app != set.app {
                mismatches.push(format!("app: {} != {}", set.app, base.app));
            }
            if base.runtime != set.runtime {
                mismatches.push(format!("runtime: {} != {}", set.runtime, base.runtime));
            }
            if base.db_schema != set.db_schema {
                mismatches.push(format!("db_schema: {} != {}", set.db_schema, base.db_schema));
            }
            if base.ipc_protocol != set.ipc_protocol {
                mismatches.push(format!("ipc_protocol: {} != {}", set.ipc_protocol, base.ipc_protocol));
            }
            if base.catalog != set.catalog {
                mismatches.push(format!("catalog: {} != {}", set.catalog, base.catalog));
            }
        }

        CompatibilityCheck {
            compatible,
            mismatches,
        }
    }
}

impl Default for CompatibilityMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_set_is_compatible() {
        let set = VersionSet {
            app: "0.1.0".to_string(),
            runtime: "0.1.0".to_string(),
            db_schema: 1,
            ipc_protocol: 1,
            catalog: "baseline-1".to_string(),
        };
        assert!(CompatibilityMatrix::new().check(&set).compatible);
    }

    #[test]
    fn mismatched_db_schema_detected() {
        let set = VersionSet {
            app: "0.1.0".to_string(),
            runtime: "0.1.0".to_string(),
            db_schema: 2,
            ipc_protocol: 1,
            catalog: "baseline-1".to_string(),
        };
        let check = CompatibilityMatrix::new().check(&set);
        assert!(!check.compatible);
        assert!(check.mismatches.iter().any(|m| m.contains("db_schema")));
    }
}