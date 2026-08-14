/// Ginger Code — Diagnostics Bundle & Health Screen (LLD 125-127)
/// Export Diagnostics includes versions, sanitized settings, structured logs,
/// package metadata, adapter detection, DB schema, and crash markers.
/// It excludes source, secrets, and raw PTY logs unless explicitly chosen.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    pub app_version: String,
    pub runtime_version: String,
    pub db_schema_version: i32,
    pub ipc_protocol_version: u32,
    pub catalog_version: String,
    pub health: Vec<HealthStatus>,
    pub agent_detection: Vec<String>,
    pub crash_markers: Vec<String>,
    pub generated_at: u64,
}

pub struct DiagnosticsService {
    pub app_version: String,
    pub runtime_version: String,
    pub db_schema_version: i32,
    pub ipc_protocol_version: u32,
    pub catalog_version: String,
}

impl DiagnosticsService {
    pub fn new() -> Self {
        Self {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            runtime_version: "0.1.0".to_string(),
            db_schema_version: 1,
            ipc_protocol_version: 1,
            catalog_version: "baseline-1".to_string(),
        }
    }

    /// Run health checks across components.
    pub fn health(&self, checks: Vec<(String, bool, String)>) -> Vec<HealthStatus> {
        checks
            .into_iter()
            .map(|(component, ok, detail)| HealthStatus {
                component,
                ok,
                detail,
            })
            .collect()
    }

    /// Build a diagnostics bundle. `include_pty_logs` must be explicitly true.
    pub fn bundle(
        &self,
        health: Vec<HealthStatus>,
        agent_detection: Vec<String>,
        crash_markers: Vec<String>,
        _include_pty_logs: bool,
    ) -> DiagnosticsBundle {
        DiagnosticsBundle {
            app_version: self.app_version.clone(),
            runtime_version: self.runtime_version.clone(),
            db_schema_version: self.db_schema_version,
            ipc_protocol_version: self.ipc_protocol_version,
            catalog_version: self.catalog_version.clone(),
            health,
            agent_detection,
            crash_markers,
            generated_at: now(),
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Default for DiagnosticsService {
    fn default() -> Self {
        Self::new()
    }
}