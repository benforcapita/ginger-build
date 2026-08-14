// Ginger Code — Action Registry
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    pub id: String,
    pub title: String,
    pub category: ActionCategory,
    pub keybinding: Option<String>,
    pub icon: Option<String>,
    pub when: Option<String>,
    pub args_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ActionCategory {
    File, Edit, View, Git, Agent, Terminal, Package, Workspace, Settings, Ginger, Help,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionContext {
    pub workspace_open: bool,
    pub editor_ready: bool,
    pub agent_count: usize,
    pub terminal_count: usize,
    pub git_clean: bool,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInvocation {
    pub id: String,
    pub args: Option<serde_json::Value>,
}

#[derive(Debug, Error, Serialize)]
pub enum ActionError {
    #[error("action not found: {0}")]
    NotFound(String),
    #[error("action execution failed: {0}")]
    ExecutionFailed(String),
}

type ActionHandler = Arc<dyn Fn(Option<serde_json::Value>) -> Result<serde_json::Value, String> + Send + Sync>;

pub struct ActionRegistry {
    actions: DashMap<String, ActionDef>,
    handlers: DashMap<String, ActionHandler>,
    context: RwLock<ActionContext>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            actions: DashMap::new(),
            handlers: DashMap::new(),
            context: RwLock::new(ActionContext::default()),
        }
    }

    pub fn register(&self, def: ActionDef, handler: ActionHandler) {
        self.actions.insert(def.id.clone(), def);
        self.handlers.insert(def.id.clone(), handler);
    }

    pub fn list(&self) -> Vec<ActionDef> {
        self.actions.iter().map(|e| e.value().clone()).collect()
    }

    pub fn get_context(&self) -> ActionContext {
        self.context.read().clone()
    }

    pub fn update_context(&self, ctx: ActionContext) {
        *self.context.write() = ctx;
    }

    pub fn invoke(&self, inv: ActionInvocation) -> Result<serde_json::Value, ActionError> {
        let handler = self.handlers.get(&inv.id).ok_or_else(|| ActionError::NotFound(inv.id.clone()))?;
        handler(inv.args).map_err(ActionError::ExecutionFailed)
    }
}

impl Default for ActionRegistry {
    fn default() -> Self { Self::new() }
}

pub fn register_core_actions(registry: &ActionRegistry) {
    let make = |id: &str, title: &str, cat: ActionCategory, kb: Option<&str>, when: Option<&str>| {
        (ActionDef {
            id: id.into(), title: title.into(), category: cat,
            keybinding: kb.map(String::from), icon: None, when: when.map(String::from),
            args_schema: None,
        }, Arc::new(move |_args| Ok(serde_json::json!({ "status": format!("{} invoked", id) }))) as ActionHandler)
    };

    let actions = vec![
        make("file.open-folder", "Open Folder", ActionCategory::File, Some("cmd+o"), None),
        make("file.save", "Save", ActionCategory::File, Some("cmd+s"), Some("editor.ready")),
        make("view.toggle-explorer", "Toggle Explorer", ActionCategory::View, Some("cmd+shift+e"), Some("workspace.open")),
        make("view.toggle-agent-dock", "Toggle Agent Dock", ActionCategory::View, Some("cmd+shift+a"), Some("workspace.open")),
        make("palette.open", "Open Command Palette", ActionCategory::View, Some("cmd+p"), None),
        make("agent.new", "New Agent", ActionCategory::Agent, Some("cmd+shift+n"), Some("workspace.open")),
        make("terminal.new", "New Terminal", ActionCategory::Terminal, Some("cmd+shift+t"), Some("workspace.open")),
        make("git.status", "Git Status", ActionCategory::Git, None, Some("workspace.open")),
        make("ginger.toggle-presence", "Toggle Ginger Presence", ActionCategory::Ginger, None, None),
    ];

    for (def, handler) in actions {
        registry.register(def, handler);
    }
    tracing::info!("Registered {} core actions", registry.list().len());
}

#[tauri::command]
pub fn invoke_action(registry: State<'_, ActionRegistry>, inv: ActionInvocation) -> Result<serde_json::Value, ActionError> {
    registry.invoke(inv)
}

#[tauri::command]
pub fn list_actions(registry: State<'_, ActionRegistry>) -> Vec<ActionDef> {
    registry.list()
}

#[tauri::command]
pub fn get_action_context(registry: State<'_, ActionRegistry>) -> ActionContext {
    registry.get_context()
}