// Ginger Code — Project Detection Tauri commands
use crate::detection::{ProjectScanner, ProjectInfo, Recommendation};
use tauri::State;

#[tauri::command]
pub fn detection_scan(scanner: State<'_, ProjectScanner>, root: String) -> ProjectInfo {
    scanner.scan(&std::path::PathBuf::from(&root))
}

#[tauri::command]
pub fn detection_recommend(scanner: State<'_, ProjectScanner>, capabilities: Vec<String>) -> Vec<Recommendation> {
    scanner.recommend(&capabilities)
}