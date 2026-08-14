// Ginger Code — Package Manager Tauri commands
use crate::package::{PackageManager, CatalogEntry, InstalledPackage};
use tauri::State;

#[tauri::command]
pub fn package_list_catalog(pm: State<'_, PackageManager>) -> Vec<CatalogEntry> {
    pm.list_catalog()
}

#[tauri::command]
pub fn package_search(pm: State<'_, PackageManager>, query: String) -> Vec<CatalogEntry> {
    pm.search(&query)
}

#[tauri::command]
pub fn package_get(pm: State<'_, PackageManager>, id: String) -> Option<CatalogEntry> {
    pm.get_entry(&id)
}

#[tauri::command]
pub fn package_install(pm: State<'_, PackageManager>, id: String) -> Result<InstalledPackage, String> {
    pm.install(&id).map_err(|e| e.to_string())
}