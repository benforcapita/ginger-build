// Ginger Code — Diff, Review and Apply Pipeline
// Review selection is non-destructive until Apply.
// Apply strategies: uncommitted → patch; clean commits → cherry-pick; explicit → merge.
// Default apply does not auto-commit. No silent stash/rebase/AI conflict resolution.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReviewError {
    #[error("review error: {0}")]
    Inner(String),
    #[error("conflict detected: {0}")]
    Conflict(String),
    #[error("agent not found: {0}")]
    AgentNotFound(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffFile {
    pub path: String,
    pub status: String, // added, modified, deleted, renamed
    pub hunks: Vec<DiffHunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffHunk {
    pub header: String,
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    pub line_type: DiffLineType,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiffLineType {
    Context,
    Addition,
    Deletion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSet {
    pub agent_id: u64,
    pub base_revision: String,
    pub files: Vec<DiffFile>,
    pub accepted_files: Vec<String>,
    pub accepted_hunks: Vec<HunkRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HunkRef {
    pub file: String,
    pub hunk_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApplyStrategy {
    Patch,
    CherryPick,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyRequest {
    pub agent_id: u64,
    pub strategy: ApplyStrategy,
    pub accepted_files: Vec<String>,
    pub accepted_hunks: Vec<HunkRef>,
    pub auto_commit: bool,
}

pub struct ReviewService;

impl ReviewService {
    pub fn new() -> Self { Self }

    /// Parse a unified diff into structured data.
    pub fn parse_diff(&self, diff: &str) -> Vec<DiffFile> {
        let mut files = Vec::new();
        let mut current_file: Option<DiffFile> = None;
        let mut current_hunk: Option<DiffHunk> = None;
        let mut old_line = 0u32;
        let mut new_line = 0u32;

        for raw in diff.lines() {
            if raw.starts_with("diff --git") {
                if let Some(h) = current_hunk.take() {
                    if let Some(f) = current_file.as_mut() {
                        f.hunks.push(h);
                    }
                }
                if let Some(f) = current_file.take() {
                    files.push(f);
                }
                let path = raw.split(' ').nth(2).unwrap_or("").trim_start_matches("a/").to_string();
                current_file = Some(DiffFile {
                    path,
                    status: "modified".into(),
                    hunks: Vec::new(),
                });
            } else if raw.starts_with("@@ ") {
                if let Some(h) = current_hunk.take() {
                    if let Some(f) = current_file.as_mut() {
                        f.hunks.push(h);
                    }
                }
                let parts: Vec<&str> = raw.split(' ').collect();
                if parts.len() >= 4 {
                    let old_parts: Vec<&str> = parts[1].trim_start_matches('-').split(',').collect();
                    let new_parts: Vec<&str> = parts[2].trim_start_matches('+').split(',').collect();
                    old_line = old_parts[0].parse().unwrap_or(0);
                    let old_count = old_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                    new_line = new_parts[0].parse().unwrap_or(0);
                    let new_count = new_parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
                    current_hunk = Some(DiffHunk {
                        header: raw.to_string(),
                        old_start: old_line,
                        old_count,
                        new_start: new_line,
                        new_count,
                        lines: Vec::new(),
                    });
                }
            } else if raw.starts_with('+') && !raw.starts_with("+++") {
                if let Some(h) = current_hunk.as_mut() {
                    new_line += 1;
                    h.lines.push(DiffLine {
                        line_type: DiffLineType::Addition,
                        content: raw[1..].to_string(),
                        old_line: None,
                        new_line: Some(new_line),
                    });
                }
            } else if raw.starts_with('-') && !raw.starts_with("---") {
                if let Some(h) = current_hunk.as_mut() {
                    old_line += 1;
                    h.lines.push(DiffLine {
                        line_type: DiffLineType::Deletion,
                        content: raw[1..].to_string(),
                        old_line: Some(old_line),
                        new_line: None,
                    });
                }
            } else if raw.starts_with(' ') {
                if let Some(h) = current_hunk.as_mut() {
                    old_line += 1;
                    new_line += 1;
                    h.lines.push(DiffLine {
                        line_type: DiffLineType::Context,
                        content: raw[1..].to_string(),
                        old_line: Some(old_line),
                        new_line: Some(new_line),
                    });
                }
            }
        }

        if let Some(h) = current_hunk.take() {
            if let Some(f) = current_file.as_mut() {
                f.hunks.push(h);
            }
        }
        if let Some(f) = current_file.take() {
            files.push(f);
        }
        files
    }

    /// Check if agent changes overlap with primary tree changes.
    pub fn check_conflict(&self, primary_changes: &[String], agent_changes: &[String]) -> bool {
        primary_changes.iter().any(|p| agent_changes.contains(p))
    }

    /// Build a patch from accepted hunks only.
    pub fn build_partial_patch(&self, files: &[DiffFile], accepted_hunks: &[HunkRef]) -> String {
        let mut patch = String::new();
        for file in files {
            let file_hunks: Vec<usize> = accepted_hunks
                .iter()
                .filter(|h| h.file == file.path)
                .map(|h| h.hunk_index)
                .collect();
            if file_hunks.is_empty() { continue; }
            patch.push_str(&format!("diff --git a/{} b/{}\n", file.path, file.path));
            for idx in file_hunks {
                if let Some(hunk) = file.hunks.get(idx) {
                    patch.push_str(&hunk.header);
                    patch.push('\n');
                    for line in &hunk.lines {
                        match line.line_type {
                            DiffLineType::Addition => { patch.push('+'); }
                            DiffLineType::Deletion => { patch.push('-'); }
                            DiffLineType::Context => { patch.push(' '); }
                        }
                        patch.push_str(&line.content);
                        patch.push('\n');
                    }
                }
            }
        }
        patch
    }
}

impl Default for ReviewService {
    fn default() -> Self { Self::new() }
}