use bitcoin::{TxOut, Amount, Script, opcodes::all::OP_RETURN};
use crate::error::{AnchorError, AnchorResult};
use crate::commitment::F1r3flyStateCommitment;

/// OP_RETURN commitment implementation using RGB infrastructure
pub struct OpReturnCommitter;

impl OpReturnCommitter {
    /// Maximum data size for OP_RETURN (Bitcoin protocol limit)
    pub const MAX_DATA_SIZE: usize = 80;
    
    /// Create OP_RETURN commitment from F1r3fly state
    pub fn create_commitment(
        state: &F1r3flyStateCommitment,
    ) -> AnchorResult<OpReturnCommitment> {
        // For OP_RETURN, we embed the 32-byte commitment hash (fits in 80-byte limit)
        let commitment_hash = state.to_bitcoin_commitment()?;
        let commitment_data = commitment_hash.to_vec();
        
        // Check size limit for OP_RETURN (should always pass with 32-byte hash)
        if commitment_data.len() > Self::MAX_DATA_SIZE {
            return Err(AnchorError::Commitment(
                format!("Data size {} exceeds OP_RETURN limit of {} bytes", 
                        commitment_data.len(), Self::MAX_DATA_SIZE)
            ));
        }
        
        Ok(OpReturnCommitment {
            data: commitment_data.clone(),
            hash: Self::hash_commitment(&commitment_data),
        })
    }
    
    /// Build OP_RETURN output for Bitcoin transaction
    pub fn create_output(
        commitment: &OpReturnCommitment
    ) -> AnchorResult<TxOut> {
        // Build script with commitment data
        let mut script_bytes = vec![OP_RETURN.to_u8()];
        if !commitment.data.is_empty() {
            script_bytes.push(commitment.data.len() as u8);
            script_bytes.extend_from_slice(&commitment.data);
        }
        
        let script = Script::from_bytes(&script_bytes);
        
        Ok(TxOut {
            value: Amount::ZERO, // OP_RETURN outputs have zero value
            script_pubkey: script.into(),
        })
    }
    
    /// Check if data can fit in OP_RETURN
    pub fn can_fit_data(data: &[u8]) -> bool {
        data.len() <= Self::MAX_DATA_SIZE
    }
    
    /// Get commitment hash from data
    pub fn hash_commitment(data: &[u8]) -> [u8; 32] {
        // Use the same Blake2b hashing as F1r3flyStateCommitment
        use blake2::{Blake2b, Digest};
        use blake2::digest::consts::U32;
        
        let mut hasher = Blake2b::<U32>::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

/// OP_RETURN commitment structure
#[derive(Clone, Debug)]
pub struct OpReturnCommitment {
    /// The commitment data to embed
    pub data: Vec<u8>,
    /// The commitment hash for verification
    pub hash: [u8; 32],
}

impl OpReturnCommitment {
    /// Create new OP_RETURN commitment
    pub fn new(data: Vec<u8>) -> AnchorResult<Self> {
        if data.len() > OpReturnCommitter::MAX_DATA_SIZE {
            return Err(AnchorError::Commitment(
                "Data too large for OP_RETURN".to_string()
            ));
        }
        
        let hash = OpReturnCommitter::hash_commitment(&data);
        
        Ok(Self { data, hash })
    }
    
    /// Get the commitment hash
    pub fn commitment_hash(&self) -> [u8; 32] {
        self.hash
    }
    
    /// Get the raw data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Verify the commitment hash matches the data
    pub fn verify(&self) -> bool {
        let computed_hash = OpReturnCommitter::hash_commitment(&self.data);
        computed_hash == self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_opret_commitment_creation() {
        let state = F1r3flyStateCommitment::new(
            [1u8; 32],  // lfb_hash
            [2u8; 32],  // rspace_root
            12345,      // block_height
            1234567890, // timestamp
            [3u8; 32],  // validator_set_hash
        );
        
        let commitment = OpReturnCommitter::create_commitment(&state).unwrap();
        
        // Should have commitment data
        assert!(!commitment.data.is_empty());
        assert_eq!(commitment.hash.len(), 32);
        
        // Should verify correctly
        assert!(commitment.verify());
    }
    
    #[test]
    fn test_opret_size_validation() {
        // Test data that fits
        let small_data = vec![0u8; 50];
        assert!(OpReturnCommitter::can_fit_data(&small_data));
        
        let commitment = OpReturnCommitment::new(small_data).unwrap();
        assert!(commitment.verify());
        
        // Test data that's too large
        let large_data = vec![0u8; 100];
        assert!(!OpReturnCommitter::can_fit_data(&large_data));
        
        let result = OpReturnCommitment::new(large_data);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_opret_output_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let commitment = OpReturnCommitment::new(data).unwrap();
        
        let output = OpReturnCommitter::create_output(&commitment).unwrap();
        
        // Should be zero value
        assert_eq!(output.value, Amount::ZERO);
        
        // Should start with OP_RETURN
        let script_bytes = output.script_pubkey.as_bytes();
        assert_eq!(script_bytes[0], OP_RETURN.to_u8());
    }
    
    #[test]
    fn test_f1r3fly_state_commitment_size() {
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]
        );
        
        let hash = state.to_bitcoin_commitment().unwrap();
        let data = hash.to_vec();
        
        // F1r3fly commitment should fit in OP_RETURN
        // 32 + 32 + 8 + 8 + 32 = 112 bytes, but we compress to 32-byte hash
        assert!(OpReturnCommitter::can_fit_data(&data));
    }
} 