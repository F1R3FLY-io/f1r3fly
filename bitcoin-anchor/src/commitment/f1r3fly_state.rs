use blake2::{Blake2b, Digest};
use blake2::digest::consts::U32;
use serde::{Deserialize, Serialize};

use crate::error::{AnchorError, AnchorResult};

/// F1r3fly state commitment for Bitcoin anchoring
/// 
/// This structure represents the state data that will be committed to Bitcoin
/// at Casper finalization points.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct F1r3flyStateCommitment {
    /// Last Finalized Block hash from Casper
    pub lfb_hash: [u8; 32],
    
    /// RSpace state root hash (Blake2b256Hash from RSpace++)
    pub rspace_root: [u8; 32],
    
    /// Casper block height when finalized
    pub block_height: i64,
    
    /// Finalization timestamp
    pub timestamp: u64,
    
    /// Hash of active validator set
    pub validator_set_hash: [u8; 32],
}

impl F1r3flyStateCommitment {
    /// Create a new F1r3fly state commitment
    pub fn new(
        lfb_hash: [u8; 32],
        rspace_root: [u8; 32],
        block_height: i64,
        timestamp: u64,
        validator_set_hash: [u8; 32],
    ) -> Self {
        Self {
            lfb_hash,
            rspace_root,
            block_height,
            timestamp,
            validator_set_hash,
        }
    }
    
    /// Generate the 32-byte commitment hash for Bitcoin anchoring
    /// 
    /// This creates a deterministic hash that will be embedded in Bitcoin
    /// transactions using RGB's commitment schemes.
    pub fn to_bitcoin_commitment(&self) -> AnchorResult<[u8; 32]> {
        let serialized = self.serialize_deterministic()?;
        
        let mut hasher = Blake2b::<U32>::new();
        hasher.update(&serialized);
        let result = hasher.finalize();
        
        Ok(result.into())
    }
    
    /// Serialize the commitment data deterministically
    fn serialize_deterministic(&self) -> AnchorResult<Vec<u8>> {
        use crate::commitment::serialization::deterministic_serialize;
        deterministic_serialize(self)
            .map_err(|e| AnchorError::Serialization(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_creation() {
        let commitment = F1r3flyStateCommitment::new(
            [1u8; 32],  // lfb_hash
            [2u8; 32],  // rspace_root
            12345,      // block_height
            1234567890, // timestamp
            [3u8; 32],  // validator_set_hash
        );
        
        assert_eq!(commitment.block_height, 12345);
        assert_eq!(commitment.timestamp, 1234567890);
    }
    
    #[test]
    fn test_bitcoin_commitment_deterministic() {
        let commitment = F1r3flyStateCommitment::new(
            [1u8; 32],
            [2u8; 32],
            12345,
            1234567890,
            [3u8; 32],
        );
        
        let hash1 = commitment.to_bitcoin_commitment().unwrap();
        let hash2 = commitment.to_bitcoin_commitment().unwrap();
        
        // Should be deterministic
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }
} 