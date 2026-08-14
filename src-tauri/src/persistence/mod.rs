// Ginger Code — Persistence Service
// SQLite stores structured metadata; filesystem stores worktrees, logs, packages, etc.

use std::path::PathBuf;
use tauri::AppHandle;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("persistence error: {0}")]
    Inner(String),
}

pub struct PersistenceService {
    data_root: PathBuf,
    db_path: PathBuf,
}

impl PersistenceService {
    pub fn new(_app: &AppHandle) -> Result<Self, PersistenceError> {
        let home = dirs::home_dir()
            .ok_or_else(|| PersistenceError::Inner("no home directory".into()))?;
        let data_root = home.join(".ginger");

        // Create directory structure
        for subdir in ["cache", "workspaces", "worktrees", "logs", "backups", "data"] {
            std::fs::create_dir_all(data_root.join(subdir))
                .map_err(|e| PersistenceError::Inner(format!("create {subdir}: {e}")))?;
        }

        let db_path = data_root.join("data").join("ginger.sqlite");
        tracing::info!("Persistence data root: {}", data_root.display());

        Ok(Self { data_root, db_path })
    }

    pub fn data_root(&self) -> &PathBuf {
        &self.data_root
    }

    pub fn db_path(&self) -> &PathBuf {
        &self.db_path
    }

    /// Run pending migrations. Versioned, transactional, with automatic backup.
    pub fn migrate(&self) -> Result<(), PersistenceError> {
        // Backup before migration
        let backup_dir = self.data_root.join("backups");
        if self.db_path.exists() {
            let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let backup = backup_dir.join(format!("ginger_{ts}.sqlite"));
            std::fs::copy(&self.db_path, &backup)
                .map_err(|e| PersistenceError::Inner(format!("backup: {e}")))?;
            // Keep only last 5 backups
            let mut backups: Vec<_> = std::fs::read_dir(&backup_dir)
                .map_err(|e| PersistenceError::Inner(e.to_string()))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map(|x| x == "sqlite").unwrap_or(false))
                .collect();
            backups.sort_by_key(|e| e.metadata().modified().ok());
            while backups.len() > 5 {
                if let Some(old) = backups.first() {
                    let _ = std::fs::remove_file(&old.path());
                }
                backups.remove(0);
            }
        }

        // Open DB and run migrations
        let conn = rusqlite::Connection::open(&self.db_path)
            .map_err(|e| PersistenceError::Inner(format!("open db: {e}")))?;

        // Run migration files in order
        let migrations_dir = self.data_root.join("migrations");
        // Bundled migrations are embedded at compile time
        let applied = self::run_migration(&conn, "001_initial", include_str!("../../migrations/001_initial.sql"))?;
        if applied {
            tracing::info!("Applied migration: 001_initial");
        }

        Ok(())
    }
}

fn run_migration(conn: &rusqlite::Connection, name: &str, sql: &str) -> Result<bool, PersistenceError> {
    // Check if already applied
    let already: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM schema_version WHERE version = ?1",
            rusqlite::params![1],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if already {
        return Ok(false);
    }

    // Run in transaction
    conn.execute_batch(&format!(
        "BEGIN; {sql} COMMIT;"
    )).map_err(|e| PersistenceError::Inner(format!("migration {name}: {e}")))?;

    Ok(true)
}