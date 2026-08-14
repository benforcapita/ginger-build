// Ginger Code — Git Service
// Wraps git executable behind a strict Rust service.
// Repository mutations use an async repository lock; reads may run concurrently.

use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::process::Command;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git error: {0}")]
    Git(String),
    #[error("not a git repository: {0}")]
    NotARepo(String),
    #[error("git binary not found")]
    BinaryNotFound,
    #[error("repository locked")]
    Locked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub clean: bool,
    pub staged: Vec<FileChange>,
    pub unstaged: Vec<FileChange>,
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub status: String, // modified, added, deleted, renamed
    pub staged: bool,
}

pub struct GitService {
    repo_lock: Arc<Mutex<()>>,
}

impl GitService {
    pub fn new() -> Self {
        Self {
            repo_lock: Arc::new(Mutex::new(())),
        }
    }

    fn git_binary() -> Result<PathBuf, GitError> {
        which::which("git").map_err(|_| GitError::BinaryNotFound)
    }

    async fn run_git(&self, repo: &PathBuf, args: &[&str]) -> Result<String, GitError> {
        let git = Self::git_binary()?;
        let output = Command::new(&git)
            .args(args)
            .current_dir(repo)
            .output()
            .await
            .map_err(|e| GitError::Git(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::Git(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if a path is a git repository.
    pub async fn is_repo(&self, path: &PathBuf) -> bool {
        self.run_git(path, &["rev-parse", "--is-inside-work-tree"])
            .await
            .is_ok()
    }

    /// Get the current branch name.
    pub async fn branch(&self, repo: &PathBuf) -> Result<String, GitError> {
        let out = self.run_git(repo, &["rev-parse", "--abbrev-ref", "HEAD"]).await?;
        Ok(out.trim().to_string())
    }

    /// Get full status of the repository.
    pub async fn status(&self, repo: &PathBuf) -> Result<GitStatus, GitError> {
        let branch = self.branch(repo).await?;
        let out = self.run_git(repo, &["status", "--porcelain=v1", "--branch"]).await?;

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for line in out.lines() {
            if line.starts_with("##") {
                continue;
            }
            let status = &line[..2];
            let path = line[3..].to_string();

            match status {
                s if s == "??" => untracked.push(path),
                s if s.starts_with(['A', 'M', 'D', 'R']) && s.chars().nth(1) == Some(' ') => {
                    staged.push(FileChange { path, status: s.chars().next().unwrap().to_string(), staged: true });
                }
                s if s.starts_with(' ') => {
                    let c = s.chars().nth(1).unwrap();
                    unstaged.push(FileChange { path, status: c.to_string(), staged: false });
                }
                _ => {
                    // Mixed staged/unstaged
                    let staged_c = s.chars().next().unwrap();
                    let unstaged_c = s.chars().nth(1).unwrap();
                    if staged_c != ' ' && staged_c != '?' {
                        staged.push(FileChange { path: path.clone(), status: staged_c.to_string(), staged: true });
                    }
                    if unstaged_c != ' ' && unstaged_c != '?' {
                        unstaged.push(FileChange { path, status: unstaged_c.to_string(), staged: false });
                    }
                }
            }
        }

        let clean = staged.is_empty() && unstaged.is_empty() && untracked.is_empty();

        Ok(GitStatus { branch, clean, staged, unstaged, untracked })
    }

    /// Create a worktree at the given path with a new branch.
    pub async fn create_worktree(
        &self,
        repo: &PathBuf,
        worktree_path: &PathBuf,
        branch: &str,
    ) -> Result<(), GitError> {
        let _lock = self.repo_lock.lock();
        self.run_git(repo, &[
            "worktree", "add",
            &worktree_path.display().to_string(),
            "-b", branch,
        ]).await?;
        tracing::info!("Worktree created: {} (branch: {})", worktree_path.display(), branch);
        Ok(())
    }

    /// Remove a worktree.
    pub async fn remove_worktree(&self, repo: &PathBuf, worktree_path: &PathBuf) -> Result<(), GitError> {
        let _lock = self.repo_lock.lock();
        self.run_git(repo, &["worktree", "remove", "--force", &worktree_path.display().to_string()]).await?;
        Ok(())
    }

    /// Get the current HEAD revision.
    pub async fn head_revision(&self, repo: &PathBuf) -> Result<String, GitError> {
        let out = self.run_git(repo, &["rev-parse", "HEAD"]).await?;
        Ok(out.trim().to_string())
    }

    /// Apply a patch to the repository.
    pub async fn apply_patch(&self, repo: &PathBuf, patch: &str) -> Result<(), GitError> {
        let _lock = self.repo_lock.lock();
        let git = Self::git_binary()?;
        let output = Command::new(&git)
            .args(["apply", "--3way"])
            .current_dir(repo)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| GitError::Git(e.to_string()))?;

        if let Some(mut stdin) = output.stdin {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(patch.as_bytes()).await.ok();
        }

        let result = output.wait_with_output().await
            .map_err(|e| GitError::Git(e.to_string()))?;

        if !result.status.success() {
            return Err(GitError::Git(String::from_utf8_lossy(&result.stderr).to_string()));
        }
        Ok(())
    }

    /// Cherry-pick a commit from another branch.
    pub async fn cherry_pick(&self, repo: &PathBuf, commit: &str) -> Result<(), GitError> {
        let _lock = self.repo_lock.lock();
        self.run_git(repo, &["cherry-pick", commit]).await?;
        Ok(())
    }

    /// Get diff between two refs or worktrees.
    pub async fn diff(&self, repo: &PathBuf, a: &str, b: &str) -> Result<String, GitError> {
        self.run_git(repo, &["diff", a, b]).await
    }
}

impl Default for GitService {
    fn default() -> Self { Self::new() }
}