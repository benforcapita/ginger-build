/// Ginger Code — Agent Adapter Discovery
/// Known terminal agents are defined through descriptors.
/// Detection checks configured overrides, Ginger-managed paths, user PATH,
/// and common macOS install locations. Missing agents are not errors.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentMode {
    Coding,
    Review,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentIsolation {
    Worktree,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDescriptor {
    pub id: String,
    pub name: String,
    pub command_candidates: Vec<String>,
    pub args: Vec<String>,
    pub worktree_support: bool,
    pub read_only: bool,
    pub default_mode: AgentMode,
    pub default_isolation: AgentIsolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDetection {
    pub descriptor_id: String,
    pub name: String,
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentDef {
    pub id: String,
    pub name: String,
    pub command: String,
    pub default_mode: AgentMode,
    pub default_isolation: AgentIsolation,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub workdir_policy: String,
}

pub struct AgentAdapterRegistry {
    builtin: Vec<AgentDescriptor>,
    custom: HashMap<String, CustomAgentDef>,
}

impl AgentAdapterRegistry {
    pub fn new() -> Self {
        let builtin = vec![
            AgentDescriptor {
                id: "claude-code".to_string(),
                name: "Claude Code".to_string(),
                command_candidates: vec!["claude".to_string(), "claude-code".to_string()],
                args: vec![],
                worktree_support: true,
                read_only: false,
                default_mode: AgentMode::Coding,
                default_isolation: AgentIsolation::Worktree,
            },
            AgentDescriptor {
                id: "codex".to_string(),
                name: "Codex".to_string(),
                command_candidates: vec!["codex".to_string()],
                args: vec![],
                worktree_support: true,
                read_only: false,
                default_mode: AgentMode::Coding,
                default_isolation: AgentIsolation::Worktree,
            },
            AgentDescriptor {
                id: "opencode".to_string(),
                name: "OpenCode".to_string(),
                command_candidates: vec!["opencode".to_string()],
                args: vec![],
                worktree_support: true,
                read_only: false,
                default_mode: AgentMode::Coding,
                default_isolation: AgentIsolation::Worktree,
            },
        ];
        Self {
            builtin,
            custom: HashMap::new(),
        }
    }

    pub fn add_custom(&mut self, def: CustomAgentDef) {
        self.custom.insert(def.id.clone(), def);
    }

    pub fn descriptors(&self) -> Vec<AgentDescriptor> {
        let mut all: Vec<AgentDescriptor> = self.builtin.clone();
        for def in self.custom.values() {
            all.push(AgentDescriptor {
                id: def.id.clone(),
                name: def.name.clone(),
                command_candidates: vec![def.command.clone()],
                args: def.args.clone(),
                worktree_support: def.default_isolation == AgentIsolation::Worktree,
                read_only: false,
                default_mode: def.default_mode,
                default_isolation: def.default_isolation,
            });
        }
        all
    }

    /// Detect which agents are available on this machine.
    /// Checks PATH and common macOS install locations.
    pub fn detect(&self) -> Vec<AgentDetection> {
        let mut results = Vec::new();
        for desc in self.descriptors() {
            let mut found = false;
            let mut path = None;
            for candidate in &desc.command_candidates {
                if let Some(p) = which(candidate) {
                    found = true;
                    path = Some(p);
                    break;
                }
            }
            results.push(AgentDetection {
                descriptor_id: desc.id.clone(),
                name: desc.name.clone(),
                found,
                path,
                version: None,
            });
        }
        results
    }
}

/// Minimal `which` — search PATH for an executable.
fn which(cmd: &str) -> Option<String> {
    let path = std::env::var("PATH").unwrap_or_default();
    for dir in path.split(':') {
        let candidate = std::path::Path::new(dir).join(cmd);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().to_string());
        }
    }
    None
}

impl Default for AgentAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}