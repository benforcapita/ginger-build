mod action;
mod editor;
mod git;
mod persistence;
mod platform;
mod presence;
mod terminal;
mod workspace;

use action::ActionRegistry;
use editor::{commands as editor_commands, NeovimHost};
use git::{commands as git_commands, GitService};
use persistence::PersistenceService;
use platform::PlatformService;
use presence::GingerPresence;
use terminal::{commands as terminal_commands, TerminalHost};
use workspace::{commands as workspace_commands, WorkspaceService};

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

            // Initialize workspace service
            let workspace_svc = WorkspaceService::new();
            app.manage(workspace_svc);

            // Initialize terminal host
            let (output_tx, mut output_rx) = tokio::sync::mpsc::channel(1024);
            let terminal_host = TerminalHost::new(output_tx);
            app.manage(terminal_host);

            // Forward terminal output to frontend via Tauri events
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                while let Some(output) = output_rx.recv().await {
                    let _ = app_handle.emit("terminal_output", &output);
                }
            });

            // Initialize Git service
            let git_svc = GitService::new();
            app.manage(git_svc);

            // Run database migrations
            if let Some(p) = app.try_state::<PersistenceService>() {
                if let Err(e) = p.migrate() {
                    tracing::warn!("Migration failed (non-fatal for first run): {e}");
                }
            }

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
            workspace_commands::workspace_open,
            workspace_commands::workspace_close,
            workspace_commands::workspace_status,
            workspace_commands::workspace_set_pane_state,
            terminal_commands::terminal_create,
            terminal_commands::terminal_write,
            terminal_commands::terminal_resize,
            terminal_commands::terminal_terminate,
            terminal_commands::terminal_list,
            git_commands::git_status,
            git_commands::git_is_repo,
            git_commands::git_branch,
            git_commands::git_create_worktree,
            git_commands::git_remove_worktree,
            git_commands::git_head_revision,
            git_commands::git_diff,
            git_commands::git_apply_patch,
            git_commands::git_cherry_pick,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ginger Code");
}