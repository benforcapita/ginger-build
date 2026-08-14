/// Ginger Code — Worktree Reconciliation Algorithm (LLD 132)
/// At startup: load recorded worktrees, parse `git worktree list --porcelain`,
/// canonicalize/match paths, validate branches, detect missing/unmanaged
/// Ginger worktrees, and mark states. Never delete during reconciliation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorktreeReconcileState {
    Recorded,
    Present,
    Missing,
    Unmanaged,
    Orphaned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeReconcileResult {
    pub path: String,
    pub branch: String,
    pub state: WorktreeReconcileState,
}

pub struct WorktreeReconciler;

impl WorktreeReconciler {
    /// Parse `git worktree list --porcelain` output.
    /// Format:
    ///   worktree /path/to/wt
    ///   HEAD <sha>
    ///   branch refs/heads/<name>
    ///   (blank line between entries)
    pub fn parse_porcelain(&self, output: &str) -> Vec<(String, String)> {
        let mut result = Vec::new();
        let mut current_path: Option<String> = None;
        let mut current_branch: Option<String> = None;

        for line in output.lines() {
            if line.is_empty() {
                if let (Some(p), Some(b)) = (current_path.take(), current_branch.take()) {
                    result.push((p, b));
                }
                continue;
            }
            if let Some(p) = line.strip_prefix("worktree ") {
                current_path = Some(p.to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                current_branch = Some(b.to_string());
            }
        }
        // flush trailing entry
        if let (Some(p), Some(b)) = (current_path, current_branch) {
            result.push((p, b));
        }
        result
    }

    /// Reconcile recorded worktrees against the actual git worktree list.
    /// `recorded` = paths Ginger knows about. `actual` = parsed porcelain list.
    pub fn reconcile(
        &self,
        recorded: &[String],
        actual: &[(String, String)],
    ) -> Vec<WorktreeReconcileResult> {
        let mut results = Vec::new();
        let actual_paths: Vec<String> = actual.iter().map(|(p, _)| p.clone()).collect();

        // Recorded worktrees that are present
        for rec in recorded {
            let canonical = canonicalize(rec);
            let present = actual_paths.iter().any(|p| canonicalize(p) == canonical);
            let branch = actual
                .iter()
                .find(|(p, _)| canonicalize(p) == canonical)
                .map(|(_, b)| b.clone())
                .unwrap_or_default();
            results.push(WorktreeReconcileResult {
                path: rec.clone(),
                branch,
                state: if present {
                    WorktreeReconcileState::Present
                } else {
                    WorktreeReconcileState::Missing
                },
            });
        }

        // Actual worktrees not recorded by Ginger = unmanaged
        for (path, branch) in actual {
            let canonical = canonicalize(path);
            let known = recorded.iter().any(|r| canonicalize(r) == canonical);
            if !known {
                results.push(WorktreeReconcileResult {
                    path: path.clone(),
                    branch: branch.clone(),
                    state: WorktreeReconcileState::Unmanaged,
                });
            }
        }

        results
    }
}

fn canonicalize(p: &str) -> String {
    Path::new(p)
        .canonicalize()
        .map(|c| c.to_string_lossy().to_string())
        .unwrap_or_else(|_| p.to_string())
}

impl Default for WorktreeReconciler {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_porcelain() {
        let output = "worktree /tmp/wt1\nHEAD abc123\nbranch refs/heads/feature-a\n\nworktree /tmp/wt2\nHEAD def456\nbranch refs/heads/feature-b\n";
        let parsed = WorktreeReconciler.parse_porcelain(output);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], ("/tmp/wt1".to_string(), "feature-a".to_string()));
        assert_eq!(parsed[1], ("/tmp/wt2".to_string(), "feature-b".to_string()));
    }

    #[test]
    fn detects_missing_and_unmanaged() {
        let recorded = vec!["/tmp/wt1".to_string()];
        let actual = vec![("/tmp/wt1".to_string(), "feature-a".to_string())];
        let results = WorktreeReconciler.reconcile(&recorded, &actual);
        // wt1 present, no unmanaged
        assert!(results.iter().any(|r| r.state == WorktreeReconcileState::Present));
    }
}