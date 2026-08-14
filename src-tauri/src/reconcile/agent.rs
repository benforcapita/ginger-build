/// Ginger Code — Agent Reconciliation Algorithm (LLD 133)
/// Persisted running/waiting threads are checked for surviving processes.
/// If unavailable, mark interrupted, inspect worktree, refresh changes,
/// and offer restart/review.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentReconcileState {
    Alive,
    Interrupted,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentReconcileResult {
    pub agent_id: i64,
    pub state: AgentReconcileState,
    pub worktree_changed: bool,
    pub offer_restart: bool,
    pub offer_review: bool,
}

pub struct AgentReconciler;

impl AgentReconciler {
    /// Reconcile a persisted agent thread.
    /// `process_alive` = whether the underlying process still exists.
    /// `worktree_changed` = whether the agent's worktree has modifications.
    pub fn reconcile(
        &self,
        agent_id: i64,
        process_alive: bool,
        worktree_changed: bool,
    ) -> AgentReconcileResult {
        if process_alive {
            AgentReconcileResult {
                agent_id,
                state: AgentReconcileState::Alive,
                worktree_changed,
                offer_restart: false,
                offer_review: false,
            }
        } else {
            // Process gone. If worktree changed, work may be recoverable.
            AgentReconcileResult {
                agent_id,
                state: AgentReconcileState::Interrupted,
                worktree_changed,
                offer_restart: true,
                offer_review: worktree_changed,
            }
        }
    }
}

impl Default for AgentReconciler {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alive_agent_no_offers() {
        let r = AgentReconciler.reconcile(1, true, false);
        assert_eq!(r.state, AgentReconcileState::Alive);
        assert!(!r.offer_restart);
        assert!(!r.offer_review);
    }

    #[test]
    fn interrupted_with_changes_offers_review() {
        let r = AgentReconciler.reconcile(2, false, true);
        assert_eq!(r.state, AgentReconcileState::Interrupted);
        assert!(r.offer_restart);
        assert!(r.offer_review);
    }
}