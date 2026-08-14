// Ginger Code — Diff/Review Tauri commands
use crate::diff::{ReviewService, DiffFile, HunkRef, ApplyStrategy};
use crate::git::GitService;
use tauri::State;

#[tauri::command]
pub async fn diff_parse(
    svc: State<'_, ReviewService>,
    diff: String,
) -> Vec<DiffFile> {
    svc.parse_diff(&diff)
}

#[tauri::command]
pub async fn diff_get(
    git: State<'_, GitService>,
    repo: String,
    base: String,
    head: String,
) -> Result<Vec<DiffFile>, String> {
    let path = std::path::PathBuf::from(&repo);
    let raw = git.diff(&path, &base, &head).await.map_err(|e| e.to_string())?;
    let svc = ReviewService::new();
    Ok(svc.parse_diff(&raw))
}

#[tauri::command]
pub fn diff_check_conflict(
    svc: State<'_, ReviewService>,
    primary_changes: Vec<String>,
    agent_changes: Vec<String>,
) -> bool {
    svc.check_conflict(&primary_changes, &agent_changes)
}

#[tauri::command]
pub fn diff_build_patch(
    svc: State<'_, ReviewService>,
    files: Vec<DiffFile>,
    accepted_hunks: Vec<HunkRef>,
) -> String {
    svc.build_partial_patch(&files, &accepted_hunks)
}

#[tauri::command]
pub async fn diff_apply(
    git: State<'_, GitService>,
    repo: String,
    patch: String,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(&repo);
    git.apply_patch(&path, &patch).await.map_err(|e| e.to_string())
}