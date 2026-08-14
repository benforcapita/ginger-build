/// Ginger Code — Test/Build Command Detection
/// Verification commands are suggested from package.json, Cargo, Go, .NET,
/// Python, Makefile, and justfile conventions. Users approve or edit them
/// before trust-based execution.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedCommand {
    pub command: String,
    pub source: String, // e.g. "package.json", "Cargo.toml", "Makefile"
    pub confidence: f32,
}

pub struct CommandDetector;

impl CommandDetector {
    /// Detect verification commands for a project root.
    pub fn detect(&self, root: &Path) -> Vec<SuggestedCommand> {
        let mut suggestions = Vec::new();

        // package.json (Node/JS/TS)
        if root.join("package.json").exists() {
            suggestions.push(SuggestedCommand {
                command: "npm test".to_string(),
                source: "package.json".to_string(),
                confidence: 0.9,
            });
            suggestions.push(SuggestedCommand {
                command: "npm run build".to_string(),
                source: "package.json".to_string(),
                confidence: 0.8,
            });
        }

        // Cargo.toml (Rust)
        if root.join("Cargo.toml").exists() {
            suggestions.push(SuggestedCommand {
                command: "cargo test".to_string(),
                source: "Cargo.toml".to_string(),
                confidence: 0.95,
            });
            suggestions.push(SuggestedCommand {
                command: "cargo build".to_string(),
                source: "Cargo.toml".to_string(),
                confidence: 0.85,
            });
        }

        // go.mod (Go)
        if root.join("go.mod").exists() {
            suggestions.push(SuggestedCommand {
                command: "go test ./...".to_string(),
                source: "go.mod".to_string(),
                confidence: 0.9,
            });
        }

        // .NET
        if root.join("*.sln").exists() || root.join("*.csproj").exists() {
            suggestions.push(SuggestedCommand {
                command: "dotnet test".to_string(),
                source: ".NET".to_string(),
                confidence: 0.85,
            });
        }

        // Python
        if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
            suggestions.push(SuggestedCommand {
                command: "pytest".to_string(),
                source: "Python".to_string(),
                confidence: 0.7,
            });
        }

        // Makefile
        if root.join("Makefile").exists() {
            suggestions.push(SuggestedCommand {
                command: "make test".to_string(),
                source: "Makefile".to_string(),
                confidence: 0.6,
            });
        }

        // justfile
        if root.join("justfile").exists() {
            suggestions.push(SuggestedCommand {
                command: "just test".to_string(),
                source: "justfile".to_string(),
                confidence: 0.6,
            });
        }

        suggestions
    }
}

impl Default for CommandDetector {
    fn default() -> Self {
        Self
    }
}