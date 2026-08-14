// Ginger Code — Verification Service
// Deterministic verification of agent work.
// Second-agent review receives original task, base revision, target diff and verification output.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("verification error: {0}")]
    Inner(String),
    #[error("verification command failed: {0}")]
    CommandFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationType {
    Build,
    Test,
    Lint,
    TypeCheck,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCommand {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub agent_id: u64,
    pub commands_run: Vec<VerificationCommand>,
    pub success: bool,
    pub outputs: Vec<CommandOutput>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandOutput {
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub reviewer_agent_id: u64,
    pub original_task: String,
    pub base_revision: String,
    pub diff: String,
    pub verification_result: VerificationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub reviewer_agent_id: u64,
    pub approved: bool,
    pub comments: String,
    pub concerns: Vec<String>,
}

pub struct VerificationService;

impl VerificationService {
    pub fn new() -> Self { Self }

    /// Run verification commands in a worktree.
    pub async fn verify(
        &self,
        agent_id: u64,
        worktree_path: &PathBuf,
        commands: Vec<VerificationCommand>,
    ) -> Result<VerificationResult, VerificationError> {
        use tokio::process::Command;
        use std::time::Instant;

        let start = Instant::now();
        let mut outputs = Vec::new();
        let mut all_success = true;

        for cmd in &commands {
            let mut command = Command::new(&cmd.command);
            command.args(&cmd.args);
            if let Some(cwd) = &cmd.cwd {
                command.current_dir(cwd);
            } else {
                command.current_dir(worktree_path);
            }

            let timeout = std::time::Duration::from_secs(
                cmd.timeout_seconds.unwrap_or(120)
            );

            let result = tokio::time::timeout(timeout, command.output()).await;

            let output = match result {
                Ok(Ok(out)) => out,
                Ok(Err(e)) => {
                    all_success = false;
                    outputs.push(CommandOutput {
                        command: format!("{} {}", cmd.command, cmd.args.join(" ")),
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    });
                    continue;
                }
                Err(_) => {
                    all_success = false;
                    outputs.push(CommandOutput {
                        command: format!("{} {}", cmd.command, cmd.args.join(" ")),
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: "timeout".into(),
                        success: false,
                    });
                    continue;
                }
            };

            let success = output.status.success();
            if !success { all_success = false; }

            outputs.push(CommandOutput {
                command: format!("{} {}", cmd.command, cmd.args.join(" ")),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                success,
            });
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(VerificationResult {
            agent_id,
            commands_run: commands,
            success: all_success,
            outputs,
            duration_ms,
        })
    }

    /// Detect project type and suggest verification commands.
    pub fn suggest_commands(&self, worktree_path: &PathBuf) -> Vec<VerificationCommand> {
        let mut commands = Vec::new();

        // Node.js / TypeScript
        if worktree_path.join("package.json").exists() {
            if worktree_path.join("node_modules").exists() {
                if worktree_path.join("tsconfig.json").exists() {
                    commands.push(VerificationCommand {
                        command: "npx".into(),
                        args: vec!["tsc".into(), "--noEmit".into()],
                        cwd: None,
                        timeout_seconds: Some(60),
                    });
                }
                if worktree_path.join("vitest.config.ts").exists()
                    || worktree_path.join("vitest.config.js").exists() {
                    commands.push(VerificationCommand {
                        command: "npx".into(),
                        args: vec!["vitest".into(), "run".into()],
                        cwd: None,
                        timeout_seconds: Some(120),
                    });
                }
                commands.push(VerificationCommand {
                    command: "npm".into(),
                    args: vec!["test".into()],
                    cwd: None,
                    timeout_seconds: Some(120),
                });
            }
        }

        // Rust
        if worktree_path.join("Cargo.toml").exists() {
            commands.push(VerificationCommand {
                command: "cargo".into(),
                args: vec!["check".into()],
                cwd: None,
                timeout_seconds: Some(120),
            });
            commands.push(VerificationCommand {
                command: "cargo".into(),
                args: vec!["test".into()],
                cwd: None,
                timeout_seconds: Some(180),
            });
            commands.push(VerificationCommand {
                command: "cargo".into(),
                args: vec!["clippy".into()],
                cwd: None,
                timeout_seconds: Some(60),
            });
        }

        // Python
        if worktree_path.join("pyproject.toml").exists()
            || worktree_path.join("setup.py").exists() {
            commands.push(VerificationCommand {
                command: "python".into(),
                args: vec!["-m".into(), "pytest".into()],
                cwd: None,
                timeout_seconds: Some(120),
            });
        }

        // Go
        if worktree_path.join("go.mod").exists() {
            commands.push(VerificationCommand {
                command: "go".into(),
                args: vec!["build".into(), "./...".into()],
                cwd: None,
                timeout_seconds: Some(60),
            });
            commands.push(VerificationCommand {
                command: "go".into(),
                args: vec!["test".into(), "./...".into()],
                cwd: None,
                timeout_seconds: Some(120),
            });
        }

        commands
    }
}

impl Default for VerificationService {
    fn default() -> Self { Self::new() }
}