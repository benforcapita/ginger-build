/// Ginger Code — Package Supply-Chain Safety (LLD 123)
/// Curated metadata pins source/version/revision and integrity hashes where
/// feasible. Locked workspaces do not silently move to branch heads.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedPackage {
    pub package_id: String,
    pub source: String,
    pub version: String,
    pub revision: String,
    pub integrity_hash: Option<String>,
}

pub struct SupplyChainGuard;

impl SupplyChainGuard {
    /// Verify a locked package is pinned to a specific revision, not a branch head.
    /// Returns true if the package is safely pinned.
    pub fn is_safely_pinned(&self, pkg: &PinnedPackage) -> bool {
        // A branch head reference (e.g. "main", "master", "HEAD") is unsafe.
        let unsafe_refs = ["main", "master", "HEAD", "develop", "latest"];
        if unsafe_refs.contains(&pkg.revision.as_str()) {
            return false;
        }
        // Integrity hash strongly recommended for curated packages.
        pkg.integrity_hash.is_some()
    }
}

impl Default for SupplyChainGuard {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_head_is_unsafe() {
        let pkg = PinnedPackage {
            package_id: "x".to_string(),
            source: "git".to_string(),
            version: "1.0".to_string(),
            revision: "main".to_string(),
            integrity_hash: Some("abc".to_string()),
        };
        assert!(!SupplyChainGuard.is_safely_pinned(&pkg));
    }

    #[test]
    fn pinned_revision_with_hash_is_safe() {
        let pkg = PinnedPackage {
            package_id: "x".to_string(),
            source: "git".to_string(),
            version: "1.0".to_string(),
            revision: "a1b2c3d4".to_string(),
            integrity_hash: Some("abc".to_string()),
        };
        assert!(SupplyChainGuard.is_safely_pinned(&pkg));
    }
}