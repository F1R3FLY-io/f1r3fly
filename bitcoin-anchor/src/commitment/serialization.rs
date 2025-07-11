use crate::commitment::F1r3flyStateCommitment;

/// Deterministically serialize F1r3fly state for Bitcoin commitments
/// 
/// This function ensures that the same state data always produces the same
/// byte sequence, which is critical for Bitcoin anchoring consistency.
pub fn deterministic_serialize(commitment: &F1r3flyStateCommitment) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Use a simple, deterministic binary format
    // This is more reliable than JSON for cryptographic purposes
    let mut buffer = Vec::new();
    
    // Serialize fields in a fixed order to ensure determinism
    buffer.extend_from_slice(&commitment.lfb_hash);
    buffer.extend_from_slice(&commitment.rspace_root);
    buffer.extend_from_slice(&commitment.block_height.to_le_bytes());
    buffer.extend_from_slice(&commitment.timestamp.to_le_bytes());
    buffer.extend_from_slice(&commitment.validator_set_hash);
    
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        let commitment = F1r3flyStateCommitment::new(
            [1u8; 32],
            [2u8; 32],
            12345,
            1234567890,
            [3u8; 32],
        );
        
        let serialized1 = deterministic_serialize(&commitment).unwrap();
        let serialized2 = deterministic_serialize(&commitment).unwrap();
        
        // Should be identical
        assert_eq!(serialized1, serialized2);
        
        // Should have expected length: 32 + 32 + 8 + 8 + 32 = 112 bytes
        assert_eq!(serialized1.len(), 112);
    }
    
    #[test]
    fn test_field_order_consistency() {
        let commitment1 = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 100, 200, [3u8; 32],
        );
        let commitment2 = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 100, 200, [3u8; 32],
        );
        
        let serialized1 = deterministic_serialize(&commitment1).unwrap();
        let serialized2 = deterministic_serialize(&commitment2).unwrap();
        
        assert_eq!(serialized1, serialized2);
    }
} 