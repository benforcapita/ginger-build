// Ginger Code — Neovim Editor Host
// Manages a bundled Neovim process via nvim --embed + Msgpack-RPC.

use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditorError {
    #[error("neovim binary not found: {0}")]
    BinaryNotFound(String),
    #[error("failed to start neovim: {0}")]
    StartFailed(String),
    #[error("rpc error: {0}")]
    Rpc(String),
    #[error("user config error: {0}")]
    UserConfig(String),
}

pub struct NeovimHost {
    child: Option<Child>,
    runtime_path: PathBuf,
}

impl NeovimHost {
    /// Create a new host with a bundled Neovim runtime path.
    pub fn new(runtime_path: PathBuf) -> Self {
        Self {
            child: None,
            runtime_path,
        }
    }

    /// Locate the Neovim binary — bundled first, system fallback.
    fn find_binary(&self) -> Result<PathBuf, EditorError> {
        // 1. Bundled runtime
        let bundled = self.runtime_path.join("bin").join("nvim");
        if bundled.exists() {
            return Ok(bundled);
        }
        // 2. System fallback (which crate)
        which::which("nvim")
            .map_err(|_| EditorError::BinaryNotFound(
                "nvim not found in bundled runtime or PATH".into()
            ))
    }

    /// Launch Neovim in --embed mode.
    pub async fn start(&mut self) -> Result<(), EditorError> {
        let nvim = self.find_binary()?;

        let child = Command::new(&nvim)
            .arg("--embed")
            .arg("--headless")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| EditorError::StartFailed(e.to_string()))?;

        self.child = Some(child);
        tracing::info!("Neovim host started: {}", nvim.display());
        Ok(())
    }

    /// Graceful stop — send :q to Neovim, then kill if needed.
    pub async fn stop(&mut self) -> Result<(), EditorError> {
        if let Some(child) = self.child.as_mut() {
            // Try graceful exit first
            // TODO: send :q via RPC
            // Force kill as fallback
            let _ = child.kill().await;
            self.child = None;
            tracing::info!("Neovim host stopped");
        }
        Ok(())
    }

    /// Check if the Neovim process is still alive.
    pub fn is_alive(&self) -> bool {
        self.child.as_ref().map(|c| {
            c.try_wait().map(|opt| opt.is_none()).unwrap_or(false)
        }).unwrap_or(false)
    }

    /// Get the runtime path.
    pub fn runtime_path(&self) -> &PathBuf {
        &self.runtime_path
    }
}