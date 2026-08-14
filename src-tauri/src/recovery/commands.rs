// Ginger Code — Recovery Tauri commands
use crate::recovery::{RecoveryService, RecoveryReport, SessionStatus};
use tauri::State;

#[tauri::command]
pub fn recovery_heartbeat(svc: State<'_, RecoveryService>) {
    svc.heartbeat();
}

#[tauri::command]
pub fn recovery_is_stale(svc: State<'_, RecoveryService>) -> bool {
    svc.is_stale()
}

#[tauri::command]
pub fn recovery_safe_mode(svc: State<'_, RecoveryService>) -> bool {
    svc.is_safe_mode()
}

#[tauri::command]
pub fn recovery_enter_safe_mode(svc: State<'_, RecoveryService>) {
    svc.enter_safe_mode();
}

#[tauri::command]
pub fn recovery_exit_safe_mode(svc: State<'_, RecoveryService>) {
    svc.exit_safe_mode();
}

#[tauri::command]
pub fn recovery_run(
    svc: State<'_, RecoveryService>,
    data_root: String,
) -> RecoveryReport {
    svc.recover(&std::path::PathBuf::from(&data_root))
}