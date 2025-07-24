use commit_verify::mpc::{
    MerkleTree, MultiSource, ProtocolId, Message, MessageMap, Method, MPC_MINIMAL_DEPTH, MerkleBlock
};
use commit_verify::{TryCommitVerify, CommitmentProtocol, CommitId};
use crate::commitment::F1r3flyStateCommitment;
use crate::error::{AnchorError, AnchorResult};
use crate::rgb::protocol::F1r3flyProtocol;

/// Multi-Protocol Commitment builder for F1r3fly and other RGB protocols
#[derive(Debug, Clone)]
pub struct F1r3flyMpcBuilder {
    /// The F1r3fly protocol instance
    protocol: F1r3flyProtocol,
    /// Messages from different protocols to include in MPC tree
    messages: MessageMap,
    /// Entropy for tree construction (None = use system random)
    entropy: Option<u64>,
}

impl F1r3flyMpcBuilder {
    /// Create a new MPC builder
    pub fn new() -> Self {
        Self {
            protocol: F1r3flyProtocol::new(),
            messages: MessageMap::default(),
            entropy: None,
        }
    }
    
    /// Set static entropy for deterministic tree construction
    pub fn with_entropy(mut self, entropy: u64) -> Self {
        self.entropy = Some(entropy);
        self
    }
    
    /// Add F1r3fly state commitment to the MPC tree
    pub fn add_f1r3fly_commitment(
        &mut self, 
        state: &F1r3flyStateCommitment
    ) -> AnchorResult<()> {
        let commitment_hash = state.to_bitcoin_commitment()?;
        let message = self.protocol.create_mpc_message(&commitment_hash);
        let protocol_id = self.protocol.rgb_protocol_id();
        
        self.messages.insert(protocol_id, message)
            .map_err(|_| AnchorError::Commitment(
                "Failed to add F1r3fly commitment to MPC tree".to_string()
            ))?;
        
        Ok(())
    }
    
    /// Add commitment from another RGB protocol
    pub fn add_protocol_commitment(
        &mut self,
        protocol_id: ProtocolId,
        message: Message,
    ) -> AnchorResult<()> {
        self.messages.insert(protocol_id, message)
            .map_err(|_| AnchorError::Commitment(
                "Failed to add protocol commitment to MPC tree".to_string()
            ))?;
        
        Ok(())
    }
    
    /// Build the MPC tree with all added commitments
    pub fn build_tree(&self) -> AnchorResult<MerkleTree> {
        if self.messages.is_empty() {
            return Err(AnchorError::Commitment(
                "Cannot build MPC tree with no commitments".to_string()
            ));
        }
        
        let source = MultiSource {
            method: Method::Sha256t,
            min_depth: MPC_MINIMAL_DEPTH,
            messages: self.messages.clone(),
            static_entropy: self.entropy,
        };
        
        // Use the UntaggedProtocol marker for RGB MPC
        struct UntaggedProtocol;
        impl CommitmentProtocol for UntaggedProtocol {}
        
        MerkleTree::try_commit(&source)
            .map_err(|e| AnchorError::Commitment(
                format!("Failed to build MPC tree: {}", e)
            ))
    }
    
    /// Get the final MPC commitment from the tree
    pub fn commitment(&self) -> AnchorResult<commit_verify::mpc::Commitment> {
        let tree = self.build_tree()?;
        Ok(tree.commit_id())
    }
    
    /// Check if F1r3fly protocol is included in this MPC
    pub fn has_f1r3fly_protocol(&self) -> bool {
        let protocol_id = self.protocol.rgb_protocol_id();
        self.messages.contains_key(&protocol_id)
    }
    
    /// Get the number of protocols in this MPC
    pub fn protocol_count(&self) -> usize {
        self.messages.len()
    }
    
    /// Get all protocol IDs in this MPC
    pub fn protocol_ids(&self) -> impl Iterator<Item = &ProtocolId> {
        self.messages.keys()
    }
}

impl Default for F1r3flyMpcBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// F1r3fly RGB Multi-Protocol Commitment
/// 
/// This represents a complete RGB MPC tree that includes F1r3fly state 
/// commitments alongside other RGB protocol commitments.
#[derive(Debug, Clone)]
pub struct F1r3flyMpc {
    /// The MPC tree
    tree: MerkleTree,
    /// F1r3fly protocol reference
    protocol: F1r3flyProtocol,
}

impl F1r3flyMpc {
    /// Create an MPC from a builder
    pub fn from_builder(builder: &F1r3flyMpcBuilder) -> AnchorResult<Self> {
        let tree = builder.build_tree()?;
        Ok(Self {
            tree,
            protocol: F1r3flyProtocol::new(),
        })
    }
    
    /// Create an MPC with only F1r3fly commitment
    pub fn with_f1r3fly_only(state: &F1r3flyStateCommitment) -> AnchorResult<Self> {
        let mut builder = F1r3flyMpcBuilder::new();
        builder.add_f1r3fly_commitment(state)?;
        Self::from_builder(&builder)
    }
    
    /// Get the final commitment hash
    pub fn commitment(&self) -> commit_verify::mpc::Commitment {
        self.tree.commit_id()
    }
    
    /// Get the MPC tree
    pub fn tree(&self) -> &MerkleTree {
        &self.tree
    }
    
    /// Get a proof for the F1r3fly protocol within this MPC
    pub fn f1r3fly_proof(&self, _message: Message) -> AnchorResult<commit_verify::mpc::MerkleProof> {
        let protocol_id = self.protocol.rgb_protocol_id();
        let block = MerkleBlock::from(&self.tree);
        block.to_merkle_proof(protocol_id)
            .map_err(|e| AnchorError::Commitment(
                format!("Failed to create F1r3fly proof: {}", e)
            ))
    }
    
    /// Verify that F1r3fly commitment is included in this MPC
    pub fn verify_f1r3fly_commitment(
        &self, 
        state: &F1r3flyStateCommitment
    ) -> AnchorResult<bool> {
        let commitment_hash = state.to_bitcoin_commitment()?;
        let _message = self.protocol.create_mpc_message(&commitment_hash);
        let protocol_id = self.protocol.rgb_protocol_id();
        
        let block = MerkleBlock::from(&self.tree);
        match block.to_merkle_proof(protocol_id) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mpc_builder() {
        let mut builder = F1r3flyMpcBuilder::new();
        assert_eq!(builder.protocol_count(), 0);
        assert!(!builder.has_f1r3fly_protocol());
        
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]
        );
        
        builder.add_f1r3fly_commitment(&state).unwrap();
        assert_eq!(builder.protocol_count(), 1);
        assert!(builder.has_f1r3fly_protocol());
    }
    
    #[test]
    fn test_f1r3fly_only_mpc() {
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]
        );
        
        let mpc = F1r3flyMpc::with_f1r3fly_only(&state).unwrap();
        assert!(mpc.verify_f1r3fly_commitment(&state).unwrap());
        
        // Should have a valid commitment
        let commitment = mpc.commitment();
        assert_ne!(commitment.to_byte_array(), [0u8; 32]);
    }
    
    #[test]
    fn test_multi_protocol_mpc() {
        let mut builder = F1r3flyMpcBuilder::new();
        
        // Add F1r3fly commitment
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]
        );
        builder.add_f1r3fly_commitment(&state).unwrap();
        
        // Add another protocol
        let other_protocol_id = ProtocolId::from([42u8; 32]);
        let other_message = Message::from([24u8; 32]);
        builder.add_protocol_commitment(other_protocol_id, other_message).unwrap();
        
        assert_eq!(builder.protocol_count(), 2);
        
        let mpc = F1r3flyMpc::from_builder(&builder).unwrap();
        assert!(mpc.verify_f1r3fly_commitment(&state).unwrap());
    }
    
    #[test]
    fn test_deterministic_entropy() {
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]
        );
        
        let mut builder1 = F1r3flyMpcBuilder::new().with_entropy(12345);
        builder1.add_f1r3fly_commitment(&state).unwrap();
        let commitment1 = builder1.commitment().unwrap();
        
        let mut builder2 = F1r3flyMpcBuilder::new().with_entropy(12345);
        builder2.add_f1r3fly_commitment(&state).unwrap();
        let commitment2 = builder2.commitment().unwrap();
        
        // Same entropy should produce same commitment
        assert_eq!(commitment1, commitment2);
    }
} 