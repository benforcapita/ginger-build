// Ginger Code — Git Tauri commands
// Exposes Git operations to the frontend.

use crate::git::{GitService, GitStatus};
use tauri::State;

#[tauri::command]
pub async fn git_status(svc: State<'_, GitService>, repo: String) -> Result<GitStatus, String> {
    let path = std::path::PathBuf::from(&repo);
    svc.status(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_is_repo(svc: State<'_, GitService>, path: String) -> Result<bool, String> {
    let p = std::path::PathBuf::from(&path);
    Ok(svc.is_repo(&p).await)
}

#[tauri::command]
pub async fn git_branch(svc: State<'_, GitService>, repo: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&repo);
    svc.branch(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_create_worktree(
    svc: State<'_, GitService>,
    repo: String,
    worktree_path: String,
    branch: String,
) -> Result<(), String> {
    let repo_path = std::path::PathBuf::from(&repo);
    let wt_path = std::path::PathBuf::from(&worktree_path);
    svc.create_worktree(&repo_path, &wt_path, &branch)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_remove_worktree(
    svc: State<'_, GitService>,
    repo: String,
    worktree_path: String,
) -> Result<(), String> {
    let repo_path = std::path::PathBuf::from(&repo);
    let wt_path = std::path::PathBuf::from(&worktree_path);
    svc.remove_worktree(&repo_path, &wt_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_head_revision(svc: State<'_, GitService>, repo: String) -> Result<String, String> {
    let path = std::path::PathBuf::from(&repo);
    svc.head_revision(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_diff(
    svc: State<'_, GitService>,
    repo: String,
    a: String,
    b: String,
) -> Result<String, String> {
    let path = std::path::PathBuf::from(&repo);
    svc.diff(&path, &a, &b).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_apply_patch(
    svc: State<'_, GitService>,
    repo: String,
    patch: String,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(&repo);
    svc.apply_patch(&path, &patch).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn git_cherry_pick(
    svc: State<'_, GitService>,
    repo: String,
    commit: String,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(&repo);
    svc.cherry_pick(&path, &commit).await.map_err(|e| e.to_string())
}