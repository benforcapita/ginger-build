// Ginger Code — Platform Services (stub for v0.1 slice 1)
use tauri::{AppHandle, Manager};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("platform error: {0}")]
    Inner(String),
}

pub struct PlatformService {
    app_handle: AppHandle,
}

impl PlatformService {
    pub fn new(app: &AppHandle) -> Result<Self, PlatformError> {
        Ok(Self { app_handle: app.clone() })
    }

    pub fn app_handle(&self) -> &AppHandle {
        &self.app_handle
    }
}