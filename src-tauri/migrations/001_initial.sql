-- Ginger Code v0.1 — Initial schema migration
-- SQLite migrations are versioned, transactional, and preceded by an automatic DB backup.

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_path TEXT NOT NULL UNIQUE,
    display_name TEXT,
    git_repository_id INTEGER,
    runtime_version TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_opened_at TEXT,
    active_session_id INTEGER
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'open',  -- open, closing, closed, crashed
    opened_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    heartbeat_at TEXT,
    app_version TEXT,
    runtime_version TEXT
);

CREATE TABLE IF NOT EXISTS pane_state (
    session_id INTEGER PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    explorer_visible INTEGER NOT NULL DEFAULT 1,
    explorer_width INTEGER NOT NULL DEFAULT 240,
    agent_dock_visible INTEGER NOT NULL DEFAULT 0,
    agent_dock_width INTEGER NOT NULL DEFAULT 300,
    bottom_panel_visible INTEGER NOT NULL DEFAULT 0,
    bottom_panel_height INTEGER NOT NULL DEFAULT 240,
    bottom_panel_tab TEXT DEFAULT 'terminal'
);

CREATE TABLE IF NOT EXISTS editor_state (
    session_id INTEGER PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    nvim_session_file TEXT,
    active_buffer TEXT,
    safe_mode INTEGER NOT NULL DEFAULT 0,
    last_saved_at TEXT
);

CREATE TABLE IF NOT EXISTS terminal_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    owner_type TEXT NOT NULL DEFAULT 'user',  -- user, agent
    owner_id INTEGER,
    cwd TEXT,
    shell TEXT,
    status TEXT NOT NULL DEFAULT 'running',  -- running, exited, killed
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at TEXT
);

CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title TEXT,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, in_progress, completed, failed
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS agent_threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
    adapter_id TEXT NOT NULL,  -- claude-code, codex, ollama, custom
    title TEXT,
    mode TEXT NOT NULL DEFAULT 'coding',  -- coding, review, research
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, running, completed, failed
    isolation TEXT NOT NULL DEFAULT 'worktree',  -- worktree, read-only, primary
    worktree_id INTEGER,
    terminal_session_id INTEGER REFERENCES terminal_sessions(id) ON DELETE SET NULL,
    started_at TEXT,
    finished_at TEXT
);

CREATE TABLE IF NOT EXISTS agent_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_thread_id INTEGER NOT NULL REFERENCES agent_threads(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    payload_json TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS worktrees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    branch TEXT NOT NULL,
    base_revision TEXT,
    owner_agent_thread_id INTEGER REFERENCES agent_threads(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'active',  -- active, review, applied, discarded, stale, orphaned
    has_unapplied_changes INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS package_environments (
    workspace_id INTEGER PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    runtime_version TEXT NOT NULL,
    manifest_hash TEXT,
    lock_hash TEXT,
    last_resolved_at TEXT
);

CREATE TABLE IF NOT EXISTS package_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    package_id TEXT NOT NULL,
    reason_code TEXT,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, installed, dismissed
    detected_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS recent_actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    action_id TEXT NOT NULL,
    last_used_at TEXT NOT NULL DEFAULT (datetime('now')),
    use_count INTEGER NOT NULL DEFAULT 1,
    UNIQUE(workspace_id, action_id)
);

INSERT OR IGNORE INTO schema_version (version) VALUES (1);