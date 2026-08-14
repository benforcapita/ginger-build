/// Ginger Code — Package Reconciliation Algorithm (LLD 134)
/// On workspace open: read manifest/lock, verify runtime compatibility and
/// cache artifacts, check project-local preferred tools, classify environment
/// readiness, start editor with safe available environment, and repair
/// optional gaps asynchronously.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvReadiness {
    Ready,
    Degraded,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageReconcileResult {
    pub manifest_present: bool,
    pub lock_present: bool,
    pub runtime_compatible: bool,
    pub cache_artifacts_ok: bool,
    pub project_local_tools: Vec<String>,
    pub readiness: EnvReadiness,
    pub repair_needed: Vec<String>,
}

pub struct PackageReconciler;

impl PackageReconciler {
    /// Classify environment readiness from manifest/lock presence and
    /// runtime compatibility. `repair_needed` lists optional gaps.
    pub fn reconcile(
        &self,
        manifest_present: bool,
        lock_present: bool,
        runtime_compatible: bool,
        cache_artifacts_ok: bool,
        project_local_tools: &[String],
    ) -> PackageReconcileResult {
        let mut repair_needed = Vec::new();

        if !lock_present {
            repair_needed.push("lock".to_string());
        }
        if !runtime_compatible {
            repair_needed.push("runtime".to_string());
        }
        if !cache_artifacts_ok {
            repair_needed.push("cache".to_string());
        }

        let readiness = if manifest_present && runtime_compatible && cache_artifacts_ok {
            EnvReadiness::Ready
        } else if manifest_present {
            EnvReadiness::Degraded
        } else {
            EnvReadiness::Missing
        };

        PackageReconcileResult {
            manifest_present,
            lock_present,
            runtime_compatible,
            cache_artifacts_ok,
            project_local_tools: project_local_tools.to_vec(),
            readiness,
            repair_needed,
        }
    }
}

impl Default for PackageReconciler {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_when_all_ok() {
        let r = PackageReconciler.reconcile(true, true, true, true, &["npm".to_string()]);
        assert_eq!(r.readiness, EnvReadiness::Ready);
        assert!(r.repair_needed.is_empty());
    }

    #[test]
    fn degraded_when_runtime_incompatible() {
        let r = PackageReconciler.reconcile(true, true, false, true, &[]);
        assert_eq!(r.readiness, EnvReadiness::Degraded);
        assert!(r.repair_needed.contains(&"runtime".to_string()));
    }
}