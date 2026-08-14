/// Ginger Code — Workspace Search
/// Uses ripgrep as the fast local engine. Supports literal/regex, globs,
/// case sensitivity, opening results in Neovim, and sending selected results
/// to an agent.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub pattern: String,
    pub root: String,
    pub regex: bool,
    pub case_sensitive: bool,
    pub globs: Vec<String>,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
}

pub struct SearchEngine;

impl SearchEngine {
    /// Run a ripgrep search. Returns matching lines.
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut cmd = std::process::Command::new("rg");
        cmd.arg("--line-number")
            .arg("--column")
            .arg("--no-heading")
            .arg("--color=never");

        if query.regex {
            cmd.arg("--regexp");
        } else {
            cmd.arg("--fixed-strings");
        }

        if query.case_sensitive {
            cmd.arg("--case-sensitive");
        } else {
            cmd.arg("--ignore-case");
        }

        for glob in &query.globs {
            cmd.arg("--glob").arg(glob);
        }

        cmd.arg("--max-count").arg(query.max_results.to_string());
        cmd.arg(&query.pattern).arg(&query.root);

        let output = match cmd.output() {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut results = Vec::new();
        for line in stdout.lines() {
            // Format: path:line:column:text
            let mut parts = line.splitn(4, ':');
            if let (Some(path), Some(line_str), Some(col_str), Some(text)) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            {
                if let (Ok(line), Ok(column)) = (line_str.parse(), col_str.parse()) {
                    results.push(SearchResult {
                        path: path.to_string(),
                        line,
                        column,
                        text: text.to_string(),
                    });
                }
            }
        }
        results
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self
    }
}