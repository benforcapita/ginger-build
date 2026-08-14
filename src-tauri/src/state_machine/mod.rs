/// Ginger Code — State Machines
/// Covers workspace open/close, agent, task, review, verification,
/// package environment, and recovery states (LLD sections 135-142).

use serde::{Deserialize, Serialize};

// --- Workspace Open State Machine (135) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceOpenState {
    Idle,
    Opening,
    Loading,
    Ready,
    EditorStarting,
    EditorReady,
    Failed,
}

// --- Workspace Close State Machine (136) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceCloseState {
    Ready,
    Closing,
    EditorSaving,
    ProcessReconciling,
    PersistenceFlushing,
    Closed,
}

// --- Agent State Machine (137) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Pending,
    Starting,
    Running,
    Completed,
    Failed,
    Interrupted,
}

// --- Task State Machine (138) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    Active,
    Review,
    Completed,
    Failed,
    Cancelled,
}

// --- Review State Machine (139) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewState {
    Pending,
    Open,
    Reviewing,
    Applying,
    Applied,
    Rejected,
}

// --- Verification State Machine (140) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationState {
    Pending,
    Running,
    Completed,
    Failed,
}

// --- Package Environment State Machine (141) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageEnvState {
    Unresolved,
    Resolving,
    Resolved,
    Installing,
    Ready,
    Degraded,
}

// --- Recovery State Machine (142) ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryState {
    Detecting,
    Reconciling,
    Restoring,
    Ready,
    SafeMode,
}

/// A generic transition guard: returns true if the transition is valid.
pub fn can_transition<T: PartialEq>(from: T, to: T, allowed: &[(T, T)]) -> bool {
    allowed.contains(&(from, to))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_open_transitions() {
        let allowed = [
            (WorkspaceOpenState::Idle, WorkspaceOpenState::Opening),
            (WorkspaceOpenState::Opening, WorkspaceOpenState::Loading),
            (WorkspaceOpenState::Loading, WorkspaceOpenState::Ready),
            (WorkspaceOpenState::Ready, WorkspaceOpenState::EditorStarting),
            (WorkspaceOpenState::EditorStarting, WorkspaceOpenState::EditorReady),
        ];
        assert!(can_transition(
            WorkspaceOpenState::Idle,
            WorkspaceOpenState::Opening,
            &allowed
        ));
        assert!(!can_transition(
            WorkspaceOpenState::Idle,
            WorkspaceOpenState::EditorReady,
            &allowed
        ));
    }

    #[test]
    fn agent_states() {
        let allowed = [
            (AgentState::Pending, AgentState::Starting),
            (AgentState::Starting, AgentState::Running),
            (AgentState::Running, AgentState::Completed),
            (AgentState::Running, AgentState::Failed),
            (AgentState::Running, AgentState::Interrupted),
        ];
        assert!(can_transition(AgentState::Running, AgentState::Completed, &allowed));
        assert!(!can_transition(AgentState::Pending, AgentState::Completed, &allowed));
    }
}