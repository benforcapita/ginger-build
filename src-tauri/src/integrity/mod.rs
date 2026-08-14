/// Ginger Code — Runtime Integrity (LLD 124)
/// Bundled runtime has an integrity manifest covering Neovim, Ginger core,
/// required assets, and catalog baseline. Repair replaces corrupt runtime/cache
/// artifacts without touching user code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityEntry {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityManifest {
    pub version: String,
    pub entries: Vec<IntegrityEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrityStatus {
    Ok,
    Corrupt,
    Missing,
}

pub struct IntegrityChecker;

impl IntegrityChecker {
    /// Verify a manifest against actual files on disk.
    /// Returns per-entry status. `repair` would re-download corrupt artifacts.
    pub fn verify(&self, manifest: &IntegrityManifest, base_dir: &std::path::Path) -> Vec<(String, IntegrityStatus)> {
        let mut results = Vec::new();
        for entry in &manifest.entries {
            let full = base_dir.join(&entry.path);
            if !full.exists() {
                results.push((entry.path.clone(), IntegrityStatus::Missing));
                continue;
            }
            match sha256_file(&full) {
                Some(hash) if hash == entry.sha256 => {
                    results.push((entry.path.clone(), IntegrityStatus::Ok));
                }
                _ => {
                    results.push((entry.path.clone(), IntegrityStatus::Corrupt));
                }
            }
        }
        results
    }
}

fn sha256_file(path: &std::path::Path) -> Option<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    Some(format!("{:x}", sha2::Sha256::digest(&buf)))
}

impl Default for IntegrityChecker {
    fn default() -> Self {
        Self
    }
}

/// Helper to build a manifest from a map of path -> sha256.
pub fn build_manifest(version: &str, entries: HashMap<String, String>) -> IntegrityManifest {
    let mut list: Vec<IntegrityEntry> = entries
        .into_iter()
        .map(|(path, sha256)| IntegrityEntry { path, sha256 })
        .collect();
    list.sort_by(|a, b| a.path.cmp(&b.path));
    IntegrityManifest {
        version: version.to_string(),
        entries: list,
    }
}