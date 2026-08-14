// Ginger Code — Agent Tauri commands
use crate::agent::{AgentSupervisor, AgentThread, CreateAgentRequest};
use tauri::State;

#[tauri::command]
pub fn agent_create(
    svc: State<'_, AgentSupervisor>,
    req: CreateAgentRequest,
) -> Result<AgentThread, String> {
    svc.create(req).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_start(
    svc: State<'_, AgentSupervisor>,
    id: u64,
    worktree_path: Option<String>,
    worktree_branch: Option<String>,
    base_revision: Option<String>,
    terminal_id: Option<u64>,
) -> Result<(), String> {
    svc.start(id, worktree_path, worktree_branch, base_revision, terminal_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_complete(svc: State<'_, AgentSupervisor>, id: u64, success: bool) -> Result<(), String> {
    svc.complete(id, success).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_get(svc: State<'_, AgentSupervisor>, id: u64) -> Option<AgentThread> {
    svc.get(id)
}

#[tauri::command]
pub fn agent_list(svc: State<'_, AgentSupervisor>) -> Vec<AgentThread> {
    svc.list()
}

#[tauri::command]
pub fn agent_remove(svc: State<'_, AgentSupervisor>, id: u64) -> Result<(), String> {
    svc.remove(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn agent_active_count(svc: State<'_, AgentSupervisor>) -> usize {
    svc.active_count()
}