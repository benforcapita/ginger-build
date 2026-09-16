mod language_servers;
mod action;
mod agent;
mod agent_adapter;
mod apply;
mod atomic;
mod cache;
mod cleanup;
mod command_detect;
mod compat;
mod concurrency;
mod correlation;
mod detection;
mod diagnostics;
mod diff;
mod editor;
mod environment;
mod error;
mod events;
mod flags;
mod ginger_config;
mod git;
mod integrity;
mod ipc;
mod jobs;
mod keybinding;
mod locking;
mod ollama;
mod package;
mod package_plan;
mod packaging;
mod path;
mod persistence;
mod platform;
mod presence;
mod process;
mod progress;
mod recommend;
mod reconcile;
mod recovery;
mod review;
mod scheduler;
mod search;
mod serialization;
mod settings;
mod stabilization;
mod state_machine;
mod supply_chain;
mod task;
mod terminal;
mod terminal_state;
mod time;
mod trust;
mod types;
mod verification;
mod verification_runs;
mod watcher;
mod workspace;

use action::ActionRegistry;
use agent::{commands as agent_commands, AgentSupervisor};
use agent_adapter::AgentAdapterRegistry;
use apply::ApplyLog;
use cache::CacheManager;
use cleanup::CleanupPolicy;
use command_detect::CommandDetector;
use compat::CompatibilityMatrix;
use concurrency::ResourceLocks;
use correlation::CorrelationId;
use detection::{commands as detection_commands, ProjectScanner};
use diagnostics::DiagnosticsService;
use diff::{commands as diff_commands, ReviewService};
use editor::{commands as editor_commands, NeovimHost};
use environment::EnvironmentManager;
use flags::FlagRegistry;
use git::{commands as git_commands, GitService};
use integrity::IntegrityChecker;
use ipc::IpcVersion;
use jobs::JobManager;
use keybinding::KeybindingChecker;
use locking::LockManager;
use ollama::OllamaClient;
use package::{commands as package_commands, PackageManager, init_curated_catalog};
use package_plan::PackagePlanner;
use packaging::{commands as packaging_commands, PackagingService, AppVersion};
use path::PathValidator;
use persistence::PersistenceService;
use platform::PlatformService;
use presence::{commands as presence_commands, GingerPresence};
use process::ProcessSupervisor;
use recommend::RecommendationEngine;
use reconcile::WorktreeReconciler;
use recovery::{commands as recovery_commands, RecoveryService};
use review::ReviewStore;
use scheduler::AgentScheduler;
use search::SearchEngine;
use settings::SettingsStore;
use stabilization::{commands as stabilization_commands};
use supply_chain::SupplyChainGuard;
use task::TaskStore;
use terminal::{commands as terminal_commands, TerminalHost};
use trust::TrustStore;
use verification::{commands as verification_commands, VerificationService};
use verification_runs::VerificationStore;
use watcher::DebouncedWatcher;
use workspace::{commands as workspace_commands, WorkspaceService};

use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ginger_code=info,tauri=info".into()),
        )
        .init();

    tauri::Builder::default()
        .menu(|app| {
            let menu = tauri::menu::Menu::default(app)?;
            #[cfg(target_os = "macos")]
            if let Some(tauri::menu::MenuItemKind::Submenu(application)) = menu.items()?.first() {
                // The default macOS application submenu ends with native Quit.
                // Replace it so unsaved terminal editors get a confirmation.
                let position = application.items()?.len().saturating_sub(1);
                application.remove_at(position)?;
                let quit = tauri::menu::MenuItem::with_id(app, "ginger-quit", "Quit Ginger Code", true, Some("CmdOrCtrl+Q"))?;
                application.insert(&quit, position)?;
            }
            Ok(menu)
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "ginger-quit" {
                let _ = app.emit("ginger-quit-requested", ());
            }
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let handle = app.handle().clone();

            let persistence = PersistenceService::new(&handle)?;
            let data_root = persistence.data_root().clone();
            app.manage(persistence);

            let platform = PlatformService::new(&handle)?;
            app.manage(platform);

            let presence = GingerPresence::new();
            app.manage(presence);

            let registry = ActionRegistry::new();
            action::register_core_actions(&registry);
            app.manage(registry);

            let runtime_path = data_root.as_path().join("runtime");
            let host = tokio::sync::Mutex::new(NeovimHost::new(runtime_path));
            app.manage(host);

            let workspace_svc = WorkspaceService::new();
            app.manage(workspace_svc);

            app.manage(TerminalHost::default());
            app.manage(language_servers::LanguageServers::default());

            let git_svc = GitService::new();
            app.manage(git_svc);

            let agent_sup = AgentSupervisor::new(3);
            app.manage(agent_sup);

            let review_svc = ReviewService::new();
            app.manage(review_svc);

            let verify_svc = VerificationService::new();
            app.manage(verify_svc);

            let pkg_cache = data_root.as_path().join("cache").join("packages");
            let pkg_mgr = PackageManager::new(pkg_cache);
            init_curated_catalog(&pkg_mgr);
            app.manage(pkg_mgr);

            let scanner = ProjectScanner::new();
            app.manage(scanner);

            let packaging_svc = PackagingService::new(data_root.clone());
            packaging_svc.set_version(AppVersion {
                app_version: "0.1.0".into(),
                runtime_version: "0.1.0".into(),
                neovim_version: "bundled".into(),
                catalog_version: "1".into(),
                build_date: chrono::Utc::now().to_rfc3339(),
            });
            app.manage(packaging_svc);

            let recovery_svc = RecoveryService::new();
            if recovery_svc.is_stale() {
                tracing::warn!("Stale heartbeat detected — running recovery");
                let report = recovery_svc.recover(&data_root);
                if report.safe_mode {
                    recovery_svc.enter_safe_mode();
                }
            }
            app.manage(recovery_svc);

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(svc) = app_handle.try_state::<RecoveryService>() {
                        svc.heartbeat();
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
            });

            if let Some(p) = app.try_state::<PersistenceService>() {
                if let Err(e) = p.migrate() {
                    tracing::warn!("Migration failed (non-fatal): {e}");
                }
            }

            // --- LLD Part II services ---
            let process_sup = ProcessSupervisor::new();
            app.manage(process_sup);

            let job_mgr = JobManager::new();
            app.manage(job_mgr);

            let adapter_registry = AgentAdapterRegistry::new();
            app.manage(adapter_registry);

            let trust_store = TrustStore::new();
            app.manage(trust_store);

            let settings_store = SettingsStore::new();
            app.manage(settings_store);

            let search_engine = SearchEngine::default();
            app.manage(search_engine);

            let verify_runs = VerificationStore::new();
            app.manage(verify_runs);

            let cmd_detector = CommandDetector::default();
            app.manage(cmd_detector);

            let review_store = ReviewStore::new();
            app.manage(review_store);

            let apply_log = ApplyLog::new();
            app.manage(apply_log);

            let task_store = TaskStore::new();
            app.manage(task_store);

            let scheduler = AgentScheduler::new();
            app.manage(scheduler);

            let env_mgr = EnvironmentManager::new();
            app.manage(env_mgr);

            let cache_mgr = CacheManager::new();
            app.manage(cache_mgr);

            let diagnostics = DiagnosticsService::new();
            app.manage(diagnostics);

            let integrity = IntegrityChecker::default();
            app.manage(integrity);

            let ollama = OllamaClient::default();
            app.manage(ollama);

            let planner = PackagePlanner::default();
            app.manage(planner);

            let recommender = RecommendationEngine::default();
            app.manage(recommender);

            let supply_guard = SupplyChainGuard::default();
            app.manage(supply_guard);

            let compat = CompatibilityMatrix::new();
            app.manage(compat);

            let flags = FlagRegistry::new();
            app.manage(flags);

            let watcher = DebouncedWatcher::default();
            app.manage(watcher);

            let locks = ResourceLocks::new();
            app.manage(locks);

            let lock_mgr = LockManager::new();
            app.manage(lock_mgr);

            let keybinding = KeybindingChecker::default();
            app.manage(keybinding);

            let cleanup = CleanupPolicy::default();
            app.manage(cleanup);

            let reconciler = WorktreeReconciler::default();
            app.manage(reconciler);

            let _corr = CorrelationId::new();
            let _ipc = IpcVersion::negotiate(ipc::IPC_PROTOCOL_VERSION);
            let _path = PathValidator::default();

            tracing::info!("Ginger Code v0.1.0 initialized — all 15 slices + LLD Part II services wired");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            action::invoke_action, action::list_actions, action::get_action_context,
            editor_commands::editor_start, editor_commands::editor_stop, editor_commands::editor_status,
            workspace::documents::workspace_read_file, workspace::documents::workspace_save_file,
            workspace_commands::workspace_file_index, workspace_commands::workspace_list_directory, workspace_commands::workspace_open, workspace_commands::workspace_close, workspace_commands::workspace_status, workspace_commands::workspace_set_pane_state,
            language_servers::language_server_start, language_servers::language_server_send, language_servers::language_server_ack, language_servers::language_server_stop, language_servers::language_server_status, language_servers::language_servers_stop_all,
            terminal_commands::terminal_launch, terminal_commands::terminal_subscribe, terminal_commands::terminal_harnesses, terminal_commands::terminal_terminate_all, terminal_commands::terminal_create, terminal_commands::terminal_write, terminal_commands::terminal_resize, terminal_commands::terminal_terminate, terminal_commands::terminal_list,
            git_commands::git_status, git_commands::git_is_repo, git_commands::git_branch, git_commands::git_create_worktree, git_commands::git_remove_worktree, git_commands::git_head_revision, git_commands::git_diff, git_commands::git_apply_patch, git_commands::git_cherry_pick,
            agent_commands::agent_create, agent_commands::agent_start, agent_commands::agent_complete, agent_commands::agent_get, agent_commands::agent_list, agent_commands::agent_remove, agent_commands::agent_active_count,
            diff_commands::diff_parse, diff_commands::diff_get, diff_commands::diff_check_conflict, diff_commands::diff_build_patch, diff_commands::diff_apply,
            verification_commands::verification_run, verification_commands::verification_suggest,
            package_commands::package_list_catalog, package_commands::package_search, package_commands::package_get, package_commands::package_install,
            detection_commands::detection_scan, detection_commands::detection_recommend,
            presence_commands::presence_state, presence_commands::presence_set_state, presence_commands::presence_config, presence_commands::presence_set_config, presence_commands::presence_message, presence_commands::presence_toggle_commentary, presence_commands::presence_cycle_personality,
            recovery_commands::recovery_heartbeat, recovery_commands::recovery_is_stale, recovery_commands::recovery_safe_mode, recovery_commands::recovery_enter_safe_mode, recovery_commands::recovery_exit_safe_mode, recovery_commands::recovery_run,
            packaging_commands::packaging_version, packaging_commands::packaging_set_version, packaging_commands::packaging_validate_update,
            stabilization_commands::e2e_tests, stabilization_commands::e2e_verify_wiring,
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                window.state::<TerminalHost>().terminate_all();
                window.state::<language_servers::LanguageServers>().stop_all();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building Ginger Code")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if app.state::<TerminalHost>().list().iter().any(|session| !session.exited) {
                    api.prevent_exit();
                    let _ = app.emit("ginger-quit-requested", ());
                }
            }
        });
}