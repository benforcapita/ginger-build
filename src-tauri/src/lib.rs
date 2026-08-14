mod action;
mod editor;
mod persistence;
mod platform;
mod presence;

use action::ActionRegistry;
use editor::{commands as editor_commands, NeovimHost};
use persistence::PersistenceService;
use platform::PlatformService;
use presence::GingerPresence;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ginger_code=info,tauri=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize persistence
            let persistence = PersistenceService::new(&handle)?;
            app.manage(persistence);

            // Initialize platform services
            let platform = PlatformService::new(&handle)?;
            app.manage(platform);

            // Initialize Ginger presence
            let presence = GingerPresence::new();
            app.manage(presence);

            // Initialize Action Registry
            let registry = ActionRegistry::new();
            action::register_core_actions(&registry);
            app.manage(registry);

            // Initialize Neovim host (not started yet)
            let runtime_path = persistence.data_root().join("runtime");
            let host = tokio::sync::Mutex::new(NeovimHost::new(runtime_path));
            app.manage(host);

            tracing::info!("Ginger Code initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            action::invoke_action,
            action::list_actions,
            action::get_action_context,
            editor_commands::editor_start,
            editor_commands::editor_stop,
            editor_commands::editor_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ginger Code");
}