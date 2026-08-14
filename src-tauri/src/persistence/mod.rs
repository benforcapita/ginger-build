// Ginger Code — Persistence Service (stub for v0.1 slice 1)
use tauri::{AppHandle, Manager};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("persistence error: {0}")]
    Inner(String),
}

pub struct PersistenceService {
    data_root: std::path::PathBuf,
}

impl PersistenceService {
    pub fn new(app: &AppHandle) -> Result<Self, PersistenceError> {
        let home = dirs::home_dir()
            .ok_or_else(|| PersistenceError::Inner("no home directory".into()))?;
        let data_root = home.join(".ginger");
        std::fs::create_dir_all(&data_root)
            .map_err(|e| PersistenceError::Inner(format!("create data root: {e}")))?;
        tracing::info!("Persistence data root: {}", data_root.display());
        Ok(Self { data_root })
    }

    pub fn data_root(&self) -> &std::path::Path {
        &self.data_root
    }
}