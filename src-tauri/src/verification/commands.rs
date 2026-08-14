// Ginger Code — Verification Tauri commands
use crate::verification::{VerificationService, VerificationCommand, VerificationResult};
use tauri::State;

#[tauri::command]
pub async fn verification_run(
    svc: State<'_, VerificationService>,
    agent_id: u64,
    worktree_path: String,
    commands: Vec<VerificationCommand>,
) -> Result<VerificationResult, String> {
    let path = std::path::PathBuf::from(&worktree_path);
    svc.verify(agent_id, &path, commands).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verification_suggest(
    svc: State<'_, VerificationService>,
    worktree_path: String,
) -> Vec<VerificationCommand> {
    let path = std::path::PathBuf::from(&worktree_path);
    svc.suggest_commands(&path)
}