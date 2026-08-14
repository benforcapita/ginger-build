// Ginger Code — Packaging Tauri commands
use crate::packaging::{PackagingService, AppVersion, UpdateCandidate, ValidationResult};
use tauri::State;
use std::path::PathBuf;

#[tauri::command]
pub fn packaging_version(svc: State<'_, PackagingService>) -> Option<AppVersion> {
    svc.version()
}

#[tauri::command]
pub fn packaging_set_version(svc: State<'_, PackagingService>, version: AppVersion) {
    svc.set_version(version);
}

#[tauri::command]
pub async fn packaging_validate_update(
    svc: State<'_, PackagingService>,
    candidate: UpdateCandidate,
    downloaded_path: String,
) -> Result<ValidationResult, String> {
    svc.validate_update(candidate, &PathBuf::from(&downloaded_path))
        .await
        .map_err(|e| e.to_string())
}