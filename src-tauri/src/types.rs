/// Type-safe ID wrappers for Ginger Code domain entities.
/// Events use domain.entity.action naming, e.g. agent.thread.started.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! typed_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub i64);

        impl $name {
            pub fn new(v: i64) -> Self { Self(v) }
            pub fn as_i64(&self) -> i64 { self.0 }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}", $prefix, self.0)
            }
        }
    };
}

typed_id!(WorkspaceId, "ws");
typed_id!(SessionId, "session");
typed_id!(TaskId, "task");
typed_id!(AgentThreadId, "agent");
typed_id!(WorktreeId, "wt");
typed_id!(TerminalSessionId, "term");
typed_id!(VerificationRunId, "verify");
typed_id!(ReviewSessionId, "review");
typed_id!(ProcessId, "proc");