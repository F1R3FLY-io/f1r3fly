//! Bitcoin L1 anchoring for F1r3fly RSpace state
//!
//! This crate provides Bitcoin Layer 1 anchoring functionality for F1r3fly's RSpace state.

pub mod bitcoin;
pub mod commitment;
pub mod config;
pub mod error;
pub mod rgb;

// Re-export main types
pub use bitcoin::{CommitmentTransaction, OpReturnCommitter};
pub use commitment::F1r3flyStateCommitment;
pub use config::AnchorConfig;
pub use error::{AnchorError, AnchorResult};
pub use rgb::F1r3flyProtocol;

/// Main Bitcoin anchoring service for F1r3fly
pub struct F1r3flyBitcoinAnchor {
    config: AnchorConfig,
}

impl F1r3flyBitcoinAnchor {
    /// Create a new Bitcoin anchor service
    pub fn new(config: AnchorConfig) -> AnchorResult<Self> {
        Ok(Self { config })
    }

    /// Create OP_RETURN commitment for F1r3fly state
    pub fn create_opret_commitment(
        &self,
        state: &F1r3flyStateCommitment,
    ) -> AnchorResult<bitcoin::opret::OpReturnCommitment> {
        bitcoin::opret::OpReturnCommitter::create_commitment(state)
    }

    /// Build Bitcoin transaction with OP_RETURN commitment
    pub fn build_commitment_transaction(
        &self,
        state: &F1r3flyStateCommitment,
        inputs: &[::bitcoin::TxIn],
        outputs: &[::bitcoin::TxOut],
    ) -> AnchorResult<CommitmentTransaction> {
        let commitment = self.create_opret_commitment(state)?;

        CommitmentTransaction::build_with_opret_commitment(inputs, outputs, &commitment)
    }

    /// Build transaction with commitment only (no other outputs)
    pub fn build_commitment_only(
        &self,
        state: &F1r3flyStateCommitment,
        inputs: &[::bitcoin::TxIn],
        change_output: ::bitcoin::TxOut,
    ) -> AnchorResult<CommitmentTransaction> {
        let commitment = self.create_opret_commitment(state)?;

        CommitmentTransaction::build_commitment_only(inputs, change_output, &commitment)
    }

    /// Anchor current state to Bitcoin testnet
    pub async fn anchor_current_state(&self) -> AnchorResult<()> {
        // TODO: Implement state anchoring logic
        Ok(())
    }
}
