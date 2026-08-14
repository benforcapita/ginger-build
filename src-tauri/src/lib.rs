mod action;
mod agent;
mod detection;
mod diff;
mod editor;
mod git;
mod package;
mod persistence;
mod platform;
mod presence;
mod terminal;
mod verification;
mod workspace;

use action::ActionRegistry;
use agent::{commands as agent_commands, AgentSupervisor};
use detection::{commands as detection_commands, ProjectScanner};
use diff::{commands as diff_commands, ReviewService};
use editor::{commands as editor_commands, NeovimHost};
use git::{commands as git_commands, GitService};
use package::{commands as package_commands, PackageManager, init_curated_catalog};
use persistence::PersistenceService;
use platform::PlatformService;
use presence::GingerPresence;
use terminal::{commands as terminal_commands, TerminalHost};
use verification::{commands as verification_commands, VerificationService};
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

            let persistence = PersistenceService::new(&handle)?;
            app.manage(persistence);

            let platform = PlatformService::new(&handle)?;
            app.manage(platform);

            let presence = GingerPresence::new();
            app.manage(presence);

            let registry = ActionRegistry::new();
            action::register_core_actions(&registry);
            app.manage(registry);

            let runtime_path = persistence.data_root().join("runtime");
            let host = tokio::sync::Mutex::new(NeovimHost::new(runtime_path));
            app.manage(host);

            let workspace_svc = WorkspaceService::new();
            app.manage(workspace_svc);

            let (output_tx, mut output_rx) = tokio::sync::mpsc::channel(1024);
            let terminal_host = TerminalHost::new(output_tx);
            app.manage(terminal_host);

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                while let Some(output) = output_rx.recv().await {
                    let _ = app_handle.emit("terminal_output", &output);
                }
            });

            let git_svc = GitService::new();
            app.manage(git_svc);

            let agent_sup = AgentSupervisor::new(3);
            app.manage(agent_sup);

            let review_svc = ReviewService::new();
            app.manage(review_svc);

            let verify_svc = VerificationService::new();
            app.manage(verify_svc);

            let pkg_cache = persistence.data_root().join("cache").join("packages");
            let pkg_mgr = PackageManager::new(pkg_cache);
            init_curated_catalog(&pkg_mgr);
            app.manage(pkg_mgr);

            let scanner = ProjectScanner::new();
            app.manage(scanner);

            if let Some(p) = app.try_state::<PersistenceService>() {
                if let Err(e) = p.migrate() {
                    tracing::warn!("Migration failed (non-fatal): {e}");
                }
            }

            tracing::info!("Ginger Code initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            action::invoke_action, action::list_actions, action::get_action_context,
            editor_commands::editor_start, editor_commands::editor_stop, editor_commands::editor_status,
            workspace_commands::workspace_open, workspace_commands::workspace_close, workspace_commands::workspace_status, workspace_commands::workspace_set_pane_state,
            terminal_commands::terminal_create, terminal_commands::terminal_write, terminal_commands::terminal_resize, terminal_commands::terminal_terminate, terminal_commands::terminal_list,
            git_commands::git_status, git_commands::git_is_repo, git_commands::git_branch, git_commands::git_create_worktree, git_commands::git_remove_worktree, git_commands::git_head_revision, git_commands::git_diff, git_commands::git_apply_patch, git_commands::git_cherry_pick,
            agent_commands::agent_create, agent_commands::agent_start, agent_commands::agent_complete, agent_commands::agent_get, agent_commands::agent_list, agent_commands::agent_remove, agent_commands::agent_active_count,
            diff_commands::diff_parse, diff_commands::diff_get, diff_commands::diff_check_conflict, diff_commands::diff_build_patch, diff_commands::diff_apply,
            verification_commands::verification_run, verification_commands::verification_suggest,
            package_commands::package_list_catalog, package_commands::package_search, package_commands::package_get, package_commands::package_install,
            detection_commands::detection_scan, detection_commands::detection_recommend,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Ginger Code");
}