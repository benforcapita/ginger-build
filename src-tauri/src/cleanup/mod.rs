/// Ginger Code — Worktree Cleanup Policy (LLD 102)
/// Default: applied clean worktrees become cleanup-eligible after seven days;
/// unapplied worktrees never auto-delete; orphaned worktrees require review.

use serde::{Deserialize, Serialize};

pub const CLEANUP_ELIGIBLE_DAYS: u64 = 7;
pub const SECONDS_PER_DAY: u64 = 86_400;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorktreeCleanupState {
    Keep,
    CleanupEligible,
    NeverAutoDelete,
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeCleanupDecision {
    pub path: String,
    pub state: WorktreeCleanupState,
    pub reason: String,
}

pub struct CleanupPolicy;

impl CleanupPolicy {
    /// Decide cleanup state for a worktree.
    /// `applied` = whether the worktree's changes were applied to primary.
    /// `clean` = whether the worktree has no uncommitted changes.
    /// `orphaned` = whether the worktree is no longer tracked by Ginger.
    /// `last_activity_days` = days since last activity.
    pub fn decide(
        &self,
        path: &str,
        applied: bool,
        clean: bool,
        orphaned: bool,
        last_activity_days: u64,
    ) -> WorktreeCleanupDecision {
        if orphaned {
            return WorktreeCleanupDecision {
                path: path.to_string(),
                state: WorktreeCleanupState::RequiresReview,
                reason: "Orphaned worktree requires manual review".to_string(),
            };
        }

        if !applied {
            return WorktreeCleanupDecision {
                path: path.to_string(),
                state: WorktreeCleanupState::NeverAutoDelete,
                reason: "Unapplied work is never auto-deleted".to_string(),
            };
        }

        if applied && clean && last_activity_days >= CLEANUP_ELIGIBLE_DAYS {
            return WorktreeCleanupDecision {
                path: path.to_string(),
                state: WorktreeCleanupState::CleanupEligible,
                reason: format!(
                    "Applied clean worktree idle for {} days",
                    last_activity_days
                ),
            };
        }

        WorktreeCleanupDecision {
            path: path.to_string(),
            state: WorktreeCleanupState::Keep,
            reason: "Not yet eligible for cleanup".to_string(),
        }
    }
}

impl Default for CleanupPolicy {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unapplied_never_auto_deletes() {
        let d = CleanupPolicy.decide("/wt", false, true, false, 30);
        assert_eq!(d.state, WorktreeCleanupState::NeverAutoDelete);
    }

    #[test]
    fn applied_clean_idle_eligible() {
        let d = CleanupPolicy.decide("/wt", true, true, false, 8);
        assert_eq!(d.state, WorktreeCleanupState::CleanupEligible);
    }

    #[test]
    fn orphaned_requires_review() {
        let d = CleanupPolicy.decide("/wt", true, true, true, 0);
        assert_eq!(d.state, WorktreeCleanupState::RequiresReview);
    }
}