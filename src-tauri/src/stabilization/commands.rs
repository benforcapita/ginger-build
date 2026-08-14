// Ginger Code — Stabilization Tauri commands
use crate::stabilization::{all_e2e_tests, verify_wiring, E2ETest};
use tauri::State;

#[tauri::command]
pub fn e2e_tests() -> Vec<E2ETest> {
    all_e2e_tests()
}

#[tauri::command]
pub fn e2e_verify_wiring() -> Vec<String> {
    verify_wiring()
}