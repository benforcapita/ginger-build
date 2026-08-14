/// Ginger Code — Serialization (LLD 217)
/// Use stable versioned payloads for IPC/events. High-value persistent state
/// uses typed tables/columns rather than uncontrolled JSON blobs.

use serde::{Deserialize, Serialize};

/// A versioned envelope for IPC/event payloads.
/// The `version` field allows forward/backward compatibility negotiation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedPayload<T> {
    pub version: u32,
    pub payload: T,
}

impl<T> VersionedPayload<T> {
    pub fn new(version: u32, payload: T) -> Self {
        Self { version, payload }
    }
}

/// Marker trait for payloads that are safe to persist in typed columns.
/// (Documentation-level contract; real enforcement is via DB schema.)
pub trait TypedPersistable {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Sample {
        name: String,
    }

    #[test]
    fn versioned_payload_roundtrips() {
        let vp = VersionedPayload::new(1, Sample { name: "x".to_string() });
        let json = serde_json::to_string(&vp).unwrap();
        let back: VersionedPayload<Sample> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert_eq!(back.payload.name, "x");
    }
}