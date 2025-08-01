//! Bitcoin L1 anchoring for F1r3fly RSpace state
//!
//! This crate provides Bitcoin Layer 1 anchoring functionality for F1r3fly's RSpace state
//! using RGB Multi-Protocol Commitments (MPC) for standards compliance.
//!
//! ## Production Workflow (with Blockchain Integration)
//!
//! ```ignore
//! use bitcoin_anchor::{F1r3flyBitcoinAnchor, F1r3flyStateCommitment, AnchorConfig, EsploraClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create F1r3fly state to commit
//!     let state = F1r3flyStateCommitment::new(
//!         [1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]
//!     );
//!
//!     // Create Esplora client for testnet
//!     let esplora_client = EsploraClient::testnet();
//!     
//!     // Create anchor service with blockchain connectivity
//!     let config = AnchorConfig::testnet().with_rgb();
//!     let anchor = F1r3flyBitcoinAnchor::with_esplora(config, esplora_client)?;
//!
//!     // Production workflow: Create → Sign → Broadcast → Monitor
//!     let source_addr = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".parse()?;
//!     let change_addr = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx".parse()?;
//!
//!     // 1. Create PSBT with real UTXOs and dynamic fees
//!     let (psbt, instructions) = anchor.create_commitment_workflow(
//!         &state, &source_addr, &change_addr, None // Use network fee estimates
//!     ).await?;
//!
//!     println!("{}", instructions);
//!
//!     // 2. External signing happens here (hardware wallet, etc.)
//!     // let signed_psbt = external_sign_psbt(psbt);
//!
//!     // 3. Broadcast and monitor (after signing)
//!     // let (broadcast_result, final_status) = anchor.broadcast_and_monitor(
//!     //     &signed_psbt, 1, 3600 // Wait for 1 confirmation, timeout 1 hour
//!     // ).await?;
//!
//!     // println!("Transaction broadcast: {}", broadcast_result.txid);
//!     // println!("Status: {}", final_status.status_string());
//!     Ok(())
//! }
//! ```
//!
//! ## Simple Demo Workflow (Testing)
//!
//! ```rust
//! use bitcoin_anchor::{F1r3flyBitcoinAnchor, F1r3flyStateCommitment, AnchorConfig, Sats};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create F1r3fly state
//! let state = F1r3flyStateCommitment::new(
//!     [1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]
//! );
//!
//! // Create anchor service  
//! let config = AnchorConfig::testnet();
//! let anchor = F1r3flyBitcoinAnchor::new(config)?;
//!
//! // Create RGB MPC commitment (standards compliant)
//! let opret = anchor.create_opret_commitment(&state)?;
//!
//! // Build Bitcoin transaction (for testing)
//! let fee_sats = Sats::from_sats(1000u64);
//! let psbt = anchor.build_psbt_transaction(&state, fee_sats)?;
//! # Ok(())
//! # }
//! ```

pub mod bitcoin;
pub mod commitment;
pub mod config;
pub mod error;

// Essential exports only
pub use bitcoin::{
    AnchorPsbt, OpReturnCommitment, OpReturnCommitter, PsbtRequest, PsbtTransaction,
};
pub use bitcoin::{
    CoinSelection, EnhancedUtxo, EsploraClient, EsploraError, EsploraTransaction, EsploraUtxo,
    FeeEstimates, RetryConfig,
};
pub use commitment::F1r3flyStateCommitment;
pub use config::AnchorConfig;
pub use error::{AnchorError, AnchorResult, ErrorContext, RecoveryStrategy};

// Re-export bp-std and bitcoin types for convenience
use ::bitcoin::Address;
pub use bpstd::{Network, Sats, Txid};

/// Main Bitcoin anchoring service for F1r3fly
pub struct F1r3flyBitcoinAnchor {
    config: AnchorConfig,
    psbt_constructor: AnchorPsbt,
}

impl F1r3flyBitcoinAnchor {
    /// Create new F1r3fly Bitcoin anchor service
    pub fn new(config: AnchorConfig) -> AnchorResult<Self> {
        let psbt_constructor = AnchorPsbt::new(config.network);

        Ok(Self {
            config,
            psbt_constructor,
        })
    }

    /// Create new F1r3fly Bitcoin anchor service with Esplora client
    pub fn with_esplora(config: AnchorConfig, esplora_client: EsploraClient) -> AnchorResult<Self> {
        let psbt_constructor = AnchorPsbt::with_esplora(config.network, esplora_client);

        Ok(Self {
            config,
            psbt_constructor,
        })
    }

    /// Create OP_RETURN commitment for F1r3fly state
    ///
    /// Creates an OP_RETURN commitment containing the F1r3fly state data.
    pub fn create_opret_commitment(
        &self,
        state: &F1r3flyStateCommitment,
    ) -> AnchorResult<OpReturnCommitment> {
        let committer = OpReturnCommitter::new();
        committer.create_commitment(state)
    }

    /// **RECOMMENDED**: Build PSBT transaction for F1r3fly state commitment
    ///
    /// Creates a PSBT (Partially Signed Bitcoin Transaction) that embeds the
    /// F1r3fly state commitment. This is the recommended approach for production.
    pub fn build_psbt_transaction(
        &self,
        state: &F1r3flyStateCommitment,
        fee: Sats,
    ) -> AnchorResult<PsbtTransaction> {
        self.psbt_constructor.build_psbt_transaction(state, fee)
    }

    /// **PRODUCTION**: Build PSBT transaction with real UTXOs from address
    ///
    /// Creates a PSBT that fetches real UTXOs from the blockchain, performs coin selection,
    /// and builds a broadcastable transaction with the F1r3fly commitment.
    pub async fn build_psbt_transaction_from_address(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>, // sat/vB, None = use network estimate
    ) -> AnchorResult<PsbtTransaction> {
        self.psbt_constructor
            .build_psbt_transaction_from_address(
                state,
                source_address,
                change_address,
                target_fee_rate,
            )
            .await
    }

    /// Get the configured Bitcoin network
    pub fn network(&self) -> Network {
        self.config.network
    }

    /// **PRODUCTION WORKFLOW**: Create PSBT for F1r3fly commitment (ready for signing)
    ///
    /// This is the main production method that:
    /// 1. Fetches real UTXOs from the source address
    /// 2. Performs professional coin selection
    /// 3. Estimates network fees dynamically
    /// 4. Creates a PSBT ready for external signing

    pub async fn create_commitment_psbt_for_signing(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>, // sat/vB, None = use network estimate
    ) -> AnchorResult<PsbtTransaction> {
        self.psbt_constructor
            .create_commitment_psbt_for_signing(
                state,
                source_address,
                change_address,
                target_fee_rate,
            )
            .await
    }

    /// **COMPLETE WORKFLOW**: End-to-end F1r3fly state commitment to Bitcoin
    ///
    /// This method demonstrates the complete production workflow:
    /// 1. Create PSBT with real UTXOs and dynamic fees
    /// 2. Return PSBT for external signing
    /// 3. After signing, call `broadcast_and_monitor()` to complete
    ///
    /// This is the recommended pattern for production integrations.

    pub async fn create_commitment_workflow(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>,
    ) -> AnchorResult<(PsbtTransaction, String)> {
        let psbt = self
            .create_commitment_psbt_for_signing(
                state,
                source_address,
                change_address,
                target_fee_rate,
            )
            .await?;

        let instructions = format!(
            "PSBT created successfully! Next steps:\n\
            1. Sign the PSBT externally (hardware wallet, etc.)\n\
            2. Call broadcast_and_monitor() with the signed PSBT\n\
            3. Monitor transaction status until confirmed\n\
            \n\
            PSBT TXID: {}\n\
            Fee: {} sats\n\
            F1r3fly commitment: {} bytes",
            psbt.txid(),
            psbt.fee.btc_sats().0,
            psbt.commitment.data().len()
        );

        Ok((psbt, instructions))
    }
}
