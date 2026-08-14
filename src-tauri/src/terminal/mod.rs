// Ginger Code — PTY Terminal Host
// Rust owns PTYs. User terminals and agent terminals share PTY infrastructure
// but not lifecycle semantics.
//
// Operations: create, write, resize, terminate, subscribe output, observe exit.

use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use portable_pty::{CommandBuilder, PtySize,native_pty_system, PtyPair};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("pty error: {0}")]
    Pty(String),
    #[error("terminal not found: {0}")]
    NotFound(u64),
    #[error("terminal already exited: {0}")]
    Exited(u64),
}

#[derive(Debug, Clone)]
pub struct TerminalOutput {
    pub terminal_id: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TerminalExit {
    pub terminal_id: u64,
    pub exit_code: Option<i32>,
}

pub struct TerminalSession {
    pub id: u64,
    pub cwd: PathBuf,
    pub shell: String,
    pub owner_type: TerminalOwner,
    pub owner_id: Option<u64>,
    pub pair: PtyPair,
    pub writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    pub exit_rx: mpsc::Receiver<TerminalExit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalOwner {
    User,
    Agent,
}

pub struct TerminalHost {
    next_id: Arc<Mutex<u64>>,
    sessions: Arc<Mutex<std::collections::HashMap<u64, TerminalSession>>>,
    output_tx: mpsc::Sender<TerminalOutput>,
}

impl TerminalHost {
    pub fn new(output_tx: mpsc::Sender<TerminalOutput>) -> Self {
        Self {
            next_id: Arc::new(Mutex::new(1)),
            sessions: Arc::new(Mutex::new(std::collections::HashMap::new())),
            output_tx,
        }
    }

    /// Create a new terminal session.
    pub fn create(
        &self,
        cwd: &PathBuf,
        shell: Option<&str>,
        owner: TerminalOwner,
        owner_id: Option<u64>,
    ) -> Result<u64, TerminalError> {
        let id = {
            let mut next = self.next_id.lock();
            let current = *next;
            *next += 1;
            current
        };

        let shell = shell.map(String::from)
            .unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into()));

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TerminalError::Pty(e.to_string()))?;

        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);

        let writer = pair
            .take_writer()
            .map_err(|e| TerminalError::Pty(format!("take_writer: {e}")))?;

        let reader = pair
            .try_clone_reader()
            .map_err(|e| TerminalError::Pty(format!("clone_reader: {e}")))?;

        // Spawn the shell process
        let _child = pair
            .spawn_command(cmd)
            .map_err(|e| TerminalError::Pty(format!("spawn: {e}")))?;

        // Forward output to the channel
        let output_tx = self.output_tx.clone();
        let sessions = self.sessions.clone();
        let reader_id = id;
        tokio::spawn(async move {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = output_tx.send(TerminalOutput {
                            terminal_id: reader_id,
                            data: buf[..n].to_vec(),
                        }).await;
                    }
                    Err(_) => break,
                }
            }
            // Mark as exited
            let mut sess = sessions.lock();
            if let Some(s) = sess.get_mut(&reader_id) {
                s.exit_rx.try_recv().ok();
            }
        });

        let (_exit_tx, exit_rx) = mpsc::channel::<TerminalExit>(1);

        let session = TerminalSession {
            id,
            cwd: cwd.clone(),
            shell: shell.clone(),
            owner_type: owner,
            owner_id,
            pair,
            writer: Arc::new(Mutex::new(writer)),
            exit_rx,
        };

        self.sessions.lock().insert(id, session);
        tracing::info!("Terminal {} created (shell: {}, cwd: {})", id, shell, cwd.display());
        Ok(id)
    }

    /// Write data to a terminal's stdin.
    pub fn write(&self, id: u64, data: &[u8]) -> Result<(), TerminalError> {
        let sessions = self.sessions.lock();
        let session = sessions.get(&id).ok_or(TerminalError::NotFound(id))?;
        let mut writer = session.writer.lock();
        writer.write_all(data).map_err(|e| TerminalError::Pty(e.to_string()))?;
        writer.flush().map_err(|e| TerminalError::Pty(e.to_string()))?;
        Ok(())
    }

    /// Resize a terminal.
    pub fn resize(&self, id: u64, rows: u16, cols: u16) -> Result<(), TerminalError> {
        let sessions = self.sessions.lock();
        let session = sessions.get(&id).ok_or(TerminalError::NotFound(id))?;
        session.pair
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| TerminalError::Pty(e.to_string()))?;
        Ok(())
    }

    /// Terminate a terminal session.
    pub fn terminate(&self, id: u64) -> Result<(), TerminalError> {
        let mut sessions = self.sessions.lock();
        let session = sessions.remove(&id).ok_or(TerminalError::NotFound(id))?;
        // Dropping the pair will kill the child process
        drop(session);
        tracing::info!("Terminal {} terminated", id);
        Ok(())
    }

    /// List all terminal sessions.
    pub fn list(&self) -> Vec<TerminalInfo> {
        self.sessions.lock().values().map(|s| TerminalInfo {
            id: s.id,
            cwd: s.cwd.display().to_string(),
            shell: s.shell.clone(),
            owner_type: s.owner_type,
            owner_id: s.owner_id,
        }).collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TerminalInfo {
    pub id: u64,
    pub cwd: String,
    pub shell: String,
    pub owner_type: TerminalOwner,
    pub owner_id: Option<u64>,
}