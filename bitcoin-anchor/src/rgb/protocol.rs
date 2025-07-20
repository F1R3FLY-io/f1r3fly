use blake2::{Blake2b, Digest};
use blake2::digest::consts::U32;
use std::fmt;

// RGB MPC integration
use commit_verify::mpc::ProtocolId;

/// F1r3fly Protocol ID for RGB MPC tree
/// 
/// This 32-byte identifier uniquely identifies the F1r3fly protocol
/// within the RGB Multi-Protocol Commitments (MPC) system.
/// 
/// Generated from: BLAKE2b("F1R3FLY_RSPACE_ANCHOR_PROTOCOL_v1")
pub const F1R3FLY_PROTOCOL_ID: [u8; 32] = [
    0x0c, 0x26, 0xa9, 0x57, 0xcb, 0x7d, 0xd0, 0x79,
    0x99, 0x93, 0x65, 0xcf, 0xe9, 0x0e, 0x9e, 0x9b,
    0x3f, 0xf8, 0x75, 0xe5, 0x87, 0x02, 0xe7, 0xd7,
    0x26, 0x75, 0xa7, 0x3d, 0x2f, 0x0e, 0x8b, 0x73
];

/// F1r3fly Protocol implementation for RGB MPC
/// 
/// This protocol defines how F1r3fly RSpace states are committed
/// to Bitcoin transactions using RGB's Multi-Protocol Commitments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct F1r3flyProtocol;

impl F1r3flyProtocol {
    /// Create a new F1r3fly protocol instance
    pub fn new() -> Self {
        Self
    }
    
    /// Get the protocol ID
    pub fn protocol_id(&self) -> [u8; 32] {
        F1R3FLY_PROTOCOL_ID
    }
    
    /// Get the RGB protocol ID for MPC integration
    pub fn rgb_protocol_id(&self) -> ProtocolId {
        ProtocolId::from(F1R3FLY_PROTOCOL_ID)
    }
    
    /// Get the protocol name
    pub fn protocol_name(&self) -> &'static str {
        "F1r3fly RSpace Anchor Protocol"
    }
    
    /// Get the protocol version
    pub fn protocol_version(&self) -> &'static str {
        "v1"
    }
    
    /// Generate the protocol ID from the canonical string
    /// 
    /// This is used for verification and testing purposes.
    pub fn generate_protocol_id() -> [u8; 32] {
        let mut hasher = Blake2b::<U32>::new();
        hasher.update(b"F1R3FLY_RSPACE_ANCHOR_PROTOCOL_v1");
        let result = hasher.finalize();
        let mut id = [0u8; 32];
        id.copy_from_slice(&result[..32]);
        id
    }
    
    /// Create RGB MPC message from F1r3fly state commitment
    pub fn create_mpc_message(&self, commitment: &[u8; 32]) -> commit_verify::mpc::Message {
        commit_verify::mpc::Message::from(*commitment)
    }
}

impl Default for F1r3flyProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for F1r3flyProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.protocol_name(), self.protocol_version())
    }
}

// RGB MPC Protocol Integration - ACTIVATED
impl From<F1r3flyProtocol> for ProtocolId {
    fn from(_protocol: F1r3flyProtocol) -> Self {
        ProtocolId::from(F1R3FLY_PROTOCOL_ID)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_id_generation() {
        let generated = F1r3flyProtocol::generate_protocol_id();
        assert_eq!(generated, F1R3FLY_PROTOCOL_ID);
    }
    
    #[test]
    fn test_protocol_properties() {
        let protocol = F1r3flyProtocol::new();
        assert_eq!(protocol.protocol_id(), F1R3FLY_PROTOCOL_ID);
        assert_eq!(protocol.protocol_name(), "F1r3fly RSpace Anchor Protocol");
        assert_eq!(protocol.protocol_version(), "v1");
    }
    
    #[test]
    fn test_protocol_display() {
        let protocol = F1r3flyProtocol::new();
        assert_eq!(protocol.to_string(), "F1r3fly RSpace Anchor Protocol v1");
    }
    
    #[test]
    fn test_rgb_protocol_id_conversion() {
        let protocol = F1r3flyProtocol::new();
        let rgb_id = protocol.rgb_protocol_id();
        assert_eq!(rgb_id.to_byte_array(), F1R3FLY_PROTOCOL_ID);
        
        // Test conversion trait
        let protocol_id: ProtocolId = protocol.into();
        assert_eq!(protocol_id, rgb_id);
    }
    
    #[test]
    fn test_mpc_message_creation() {
        let protocol = F1r3flyProtocol::new();
        let commitment = [42u8; 32];
        let message = protocol.create_mpc_message(&commitment);
        assert_eq!(message.to_byte_array(), commitment);
    }
} 