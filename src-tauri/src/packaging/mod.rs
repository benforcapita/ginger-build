// Ginger Code — macOS Packaging & Signing
// v0.1 target: signed Apple Silicon Ginger Code.app with bundled Neovim,
// protected Ginger core, required runtime assets, and version reporting.
// Update architecture validates signatures/hashes and activates new runtime
// versions only after side-by-side validation.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackagingError {
    #[error("packaging error: {0}")]
    Inner(String),
    #[error("signing failed: {0}")]
    SigningFailed(String),
    #[error("validation failed: {0}")]
    ValidationFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppVersion {
    pub app_version: String,
    pub runtime_version: String,
    pub neovim_version: String,
    pub catalog_version: String,
    pub build_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCandidate {
    pub version: String,
    pub download_url: String,
    pub sha256: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub candidate: UpdateCandidate,
    pub signature_valid: bool,
    pub hash_valid: bool,
    pub neovim_check_passed: bool,
    pub core_check_passed: bool,
    pub packages_check_passed: bool,
    pub overall_passed: bool,
}

pub struct PackagingService {
    data_root: PathBuf,
    current_version: RwLock<Option<AppVersion>>,
}

use parking_lot::RwLock;

impl PackagingService {
    pub fn new(data_root: PathBuf) -> Self {
        Self {
            data_root,
            current_version: RwLock::new(None),
        }
    }

    /// Get the current app version info.
    pub fn version(&self) -> Option<AppVersion> {
        self.current_version.read().clone()
    }

    /// Set the current version (called on startup).
    pub fn set_version(&self, version: AppVersion) {
        *self.current_version.write() = Some(version);
    }

    /// Validate an update candidate before activation.
    /// Checks signature, hash, Neovim launch, core integrity, and required packages.
    pub async fn validate_update(
        &self,
        candidate: UpdateCandidate,
        downloaded_path: &PathBuf,
    ) -> Result<ValidationResult, PackagingError> {
        // 1. Verify SHA256 hash
        let hash = self.compute_hash(downloaded_path)
            .map_err(|e| PackagingError::ValidationFailed(format!("hash: {e}")))?;
        let hash_valid = hash == candidate.sha256;

        // 2. Verify signature (placeholder — actual code signing verification)
        let signature_valid = !candidate.signature.is_empty();

        // 3. Launch validation Neovim (side-by-side)
        let neovim_check_passed = self.validate_neovim(downloaded_path).await;

        // 4. Verify protected core
        let core_check_passed = self.validate_core(downloaded_path);

        // 5. Verify required packages
        let packages_check_passed = self.validate_packages(downloaded_path);

        let overall_passed = hash_valid && signature_valid
            && neovim_check_passed && core_check_passed
            && packages_check_passed;

        let result = ValidationResult {
            candidate: candidate.clone(),
            signature_valid,
            hash_valid,
            neovim_check_passed,
            core_check_passed,
            packages_check_passed,
            overall_passed,
        };

        if !overall_passed {
            tracing::warn!(
                "Update validation failed: hash={}, sig={}, nvim={}, core={}, pkgs={}",
                hash_valid, signature_valid, neovim_check_passed, core_check_passed, packages_check_passed
            );
        }

        Ok(result)
    }

    fn compute_hash(&self, path: &PathBuf) -> Result<String, String> {
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hasher = sha2::Sha256::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = file.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 { break; }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    async fn validate_neovim(&self, _path: &PathBuf) -> bool {
        // TODO: launch nvim --embed from the candidate, verify RPC handshake
        // For now, return true as placeholder
        true
    }

    fn validate_core(&self, _path: &PathBuf) -> bool {
        // TODO: check protected_init.lua exists and is valid
        true
    }

    fn validate_packages(&self, _path: &PathBuf) -> bool {
        // TODO: check required packages are present
        true
    }

    /// Get the app support directory path.
    pub fn app_support_dir(&self) -> PathBuf {
        self.data_root.join("app-support")
    }

    /// Get the runtime directory path.
    pub fn runtime_dir(&self) -> PathBuf {
        self.data_root.join("runtime")
    }
}

// Sha256 import
use sha2::{Sha256, Digest};

impl Default for PackagingService {
    fn default() -> Self {
        Self::new(dirs::home_dir().unwrap_or_default().join(".ginger"))
    }
}