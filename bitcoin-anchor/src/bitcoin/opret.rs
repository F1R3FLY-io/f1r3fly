use crate::commitment::F1r3flyStateCommitment;
use crate::error::{AnchorError, AnchorResult};
use bitcoin::{Amount, ScriptBuf, TxOut};

/// Creates OP_RETURN commitments for F1r3fly state
#[derive(Clone, Debug)]
pub struct OpReturnCommitter;

impl OpReturnCommitter {
    /// Maximum data size for OP_RETURN (Bitcoin protocol limit)
    pub const MAX_DATA_SIZE: usize = 80;

    /// Create new OP_RETURN committer
    pub fn new() -> Self {
        Self
    }

    /// Create OP_RETURN commitment from F1r3fly state
    pub fn create_commitment(
        &self,
        state: &F1r3flyStateCommitment,
    ) -> AnchorResult<OpReturnCommitment> {
        // Use the compact 32-byte hash instead of full data
        let data = state.to_opreturn_data();
        let hash = state.commitment_hash();

        Ok(OpReturnCommitment { data, hash })
    }

    /// Create OP_RETURN output for Bitcoin transaction
    pub fn create_output(commitment: &OpReturnCommitment) -> AnchorResult<TxOut> {
        // Build script with OP_RETURN and commitment data
        let mut script_bytes = vec![0x6a]; // OP_RETURN opcode
        let data = commitment.data();

        if !data.is_empty() {
            script_bytes.push(data.len() as u8);
            script_bytes.extend_from_slice(data);
        }

        let script = ScriptBuf::from_bytes(script_bytes);

        Ok(TxOut {
            value: Amount::ZERO, // OP_RETURN outputs have zero value
            script_pubkey: script,
        })
    }

    /// Get commitment hash from data
    pub fn hash_commitment(data: &[u8]) -> [u8; 32] {
        // Use the same Blake2b hashing as F1r3flyStateCommitment
        use blake2::digest::consts::U32;
        use blake2::{Blake2b, Digest};

        let mut hasher = Blake2b::<U32>::new();
        hasher.update(data);
        hasher.finalize().into()
    }
}

impl Default for OpReturnCommitter {
    fn default() -> Self {
        Self::new()
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
                "Data too large for OP_RETURN".to_string(),
            ));
        }

        let hash = OpReturnCommitter::hash_commitment(&data);

        Ok(Self { data, hash })
    }

    /// Get commitment data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get commitment hash
    pub fn hash(&self) -> &[u8; 32] {
        &self.hash
    }

    /// Get data size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if commitment is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_commitment() {
        let state = F1r3flyStateCommitment::new([1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]);

        let committer = OpReturnCommitter::new();
        let commitment = committer
            .create_commitment(&state)
            .expect("Should create commitment");

        // Should have data within size limits
        assert!(commitment.len() <= OpReturnCommitter::MAX_DATA_SIZE);
        assert!(!commitment.is_empty());

        // Should have a valid hash
        assert_ne!(*commitment.hash(), [0u8; 32]);
    }
}
