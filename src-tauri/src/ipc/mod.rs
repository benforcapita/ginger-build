/// Ginger Code — IPC Versioning (LLD 130)
/// Frontend and Rust backend negotiate an IPC protocol version.
/// Mismatch triggers Restart/Repair rather than mysterious runtime errors.

use serde::{Deserialize, Serialize};

pub const IPC_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcNegotiation {
    Compatible,
    Mismatch { frontend: u32, backend: u32 },
}

pub struct IpcVersion;

impl IpcVersion {
    /// Negotiate protocol version between frontend and backend.
    pub fn negotiate(frontend_version: u32) -> IpcNegotiation {
        if frontend_version == IPC_PROTOCOL_VERSION {
            IpcNegotiation::Compatible
        } else {
            IpcNegotiation::Mismatch {
                frontend: frontend_version,
                backend: IPC_PROTOCOL_VERSION,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_when_equal() {
        assert_eq!(IpcVersion::negotiate(1), IpcNegotiation::Compatible);
    }

    #[test]
    fn mismatch_when_different() {
        match IpcVersion::negotiate(2) {
            IpcNegotiation::Mismatch { frontend, backend } => {
                assert_eq!(frontend, 2);
                assert_eq!(backend, 1);
            }
            _ => panic!("expected mismatch"),
        }
    }
}