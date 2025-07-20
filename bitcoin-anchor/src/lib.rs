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
//! let config = AnchorConfig::testnet().with_rgb();
//! let anchor = F1r3flyBitcoinAnchor::new(config)?;
//!
//! // Create RGB MPC commitment (standards compliant)
//! let mpc = anchor.create_rgb_mpc_commitment(&state)?;
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
pub mod recovery;
pub mod rgb;
pub mod validation;

// Essential exports only
pub use bitcoin::{CommitmentTransaction, OpReturnCommitter, OpReturnCommitment, AnchorPsbt, PsbtRequest, PsbtTransaction};
pub use bitcoin::{EsploraClient, EsploraError, EsploraUtxo, EsploraTransaction, FeeEstimates, BroadcastResult, TransactionStatus, CoinSelection, EnhancedUtxo};
pub use commitment::{F1r3flyStateCommitment, F1r3flyCommitmentId};
pub use config::AnchorConfig;
pub use error::{AnchorError, AnchorResult, ErrorContext, RecoveryStrategy};
pub use recovery::{RecoveryService, ErrorReport, RecoveryAction, ActionPriority};
pub use rgb::{F1r3flyProtocol, F1R3FLY_PROTOCOL_ID, F1r3flyMpcBuilder, F1r3flyMpc};
pub use validation::{TransactionValidator, ValidationConfig, ValidationResult, ValidatedUtxo};

// Re-export bp-std and bitcoin types for convenience  
pub use bpstd::{Network, Sats, Txid};
use ::bitcoin::Address;

/// System health diagnostic report
#[derive(Debug)]
pub struct SystemHealthReport {
    pub network: Network,
    pub blockchain_connectivity: bool,
    pub fee_estimation_available: bool,
    pub rgb_features_enabled: bool,
    pub current_block_height: Option<u32>,
    pub current_fee_rates: Option<(f64, f64, f64)>, // (fast, medium, slow)
    pub issues: Vec<String>,
}

impl SystemHealthReport {
    pub fn new() -> Self {
        Self {
            network: Network::Testnet3,
            blockchain_connectivity: false,
            fee_estimation_available: false,
            rgb_features_enabled: false,
            current_block_height: None,
            current_fee_rates: None,
            issues: Vec::new(),
        }
    }

    /// Check if system is healthy for Bitcoin anchoring operations
    pub fn is_healthy(&self) -> bool {
        self.blockchain_connectivity && self.fee_estimation_available && self.issues.is_empty()
    }

    /// Get a summary of system status
    pub fn summary(&self) -> String {
        if self.is_healthy() {
            format!(
                "✅ System healthy on {:?} at block {} with fee rates: {:.1}/{:.1}/{:.1} sat/vB",
                self.network,
                self.current_block_height.unwrap_or(0),
                self.current_fee_rates.map(|(f, _m, _s)| f).unwrap_or(0.0),
                self.current_fee_rates.map(|(_f, m, _s)| m).unwrap_or(0.0),
                self.current_fee_rates.map(|(_f, _m, s)| s).unwrap_or(0.0),
            )
        } else {
            format!(
                "⚠️  System issues detected on {:?}: {} problems",
                self.network,
                self.issues.len()
            )
        }
    }
}

/// Main Bitcoin anchoring service for F1r3fly
pub struct F1r3flyBitcoinAnchor {
    config: AnchorConfig,
    psbt_constructor: AnchorPsbt,
    recovery_service: RecoveryService,
}

impl F1r3flyBitcoinAnchor {
    /// Create new F1r3fly Bitcoin anchor service
    pub fn new(config: AnchorConfig) -> AnchorResult<Self> {
        let psbt_constructor = AnchorPsbt::new(config.network);
        let recovery_service = RecoveryService::new();
        
        Ok(Self {
            config,
            psbt_constructor,
            recovery_service,
        })
    }

    /// Create new F1r3fly Bitcoin anchor service with custom recovery configuration
    pub fn with_recovery_config(config: AnchorConfig, recovery_service: RecoveryService) -> AnchorResult<Self> {
        let psbt_constructor = AnchorPsbt::new(config.network);
        
        Ok(Self {
            config,
            psbt_constructor,
            recovery_service,
        })
    }

    /// Create new F1r3fly Bitcoin anchor service with Esplora client
    pub fn with_esplora(config: AnchorConfig, esplora_client: EsploraClient) -> AnchorResult<Self> {
        let psbt_constructor = AnchorPsbt::with_esplora(config.network, esplora_client);
        let recovery_service = RecoveryService::new();
        
        Ok(Self {
            config,
            psbt_constructor,
            recovery_service,
        })
    }

    /// Create new F1r3fly Bitcoin anchor service with Esplora client and custom recovery
    pub fn with_esplora_and_recovery(
        config: AnchorConfig, 
        esplora_client: EsploraClient,
        recovery_service: RecoveryService
    ) -> AnchorResult<Self> {
        let psbt_constructor = AnchorPsbt::with_esplora(config.network, esplora_client);
        
        Ok(Self {
            config,
            psbt_constructor,
            recovery_service,
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
        self.psbt_constructor.build_psbt_transaction_from_address(
            state, 
            source_address, 
            change_address, 
            target_fee_rate
        ).await
    }
    
    /// **RGB COMPLIANCE**: Create RGB MPC commitment
    /// 
    /// Creates an RGB Multi-Protocol Commitment for F1r3fly state, providing
    /// compliance with RGB standards and enabling efficient multi-protocol usage.
    pub fn create_rgb_mpc_commitment(
        &self,
        state: &F1r3flyStateCommitment,
    ) -> AnchorResult<F1r3flyMpc> {
        if !self.config.rgb_enabled {
            return Err(AnchorError::Configuration(
                "RGB features must be enabled for MPC commitments".to_string()
            ));
        }
        
        F1r3flyMpc::with_f1r3fly_only(state)
    }
    
    /// Get the configured Bitcoin network
    pub fn network(&self) -> Network {
        self.config.network
    }
    
    /// Check if RGB features are enabled
    pub fn rgb_enabled(&self) -> bool {
        self.config.rgb_enabled
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
        self.psbt_constructor.create_commitment_psbt_for_signing(
            state, 
            source_address, 
            change_address, 
            target_fee_rate
        ).await
    }

    /// **PRODUCTION BROADCAST**: Broadcast a signed PSBT transaction
    /// 
    /// After external signing (e.g., with hardware wallet), this method broadcasts
    /// the signed PSBT to the Bitcoin network via Esplora API.
    
    pub async fn broadcast_signed_psbt(
        &self,
        signed_psbt: &PsbtTransaction,
    ) -> AnchorResult<BroadcastResult> {
        self.psbt_constructor.broadcast_signed_psbt(signed_psbt).await
    }

    /// **MONITORING**: Check the status of a broadcast transaction
    
    pub async fn check_transaction_status(
        &self,
        psbt: &PsbtTransaction,
    ) -> AnchorResult<TransactionStatus> {
        self.psbt_constructor.check_transaction_status(psbt).await
    }

    /// **MONITORING**: Wait for transaction confirmation with timeout
    /// 
    /// Polls the transaction status until it reaches the desired confirmations.
    /// Useful for automated workflows that need to wait for settlement.
    
    pub async fn wait_for_confirmation(
        &self,
        psbt: &PsbtTransaction,
        min_confirmations: u32,
        timeout_seconds: u64,
    ) -> AnchorResult<TransactionStatus> {
        self.psbt_constructor.wait_for_confirmation(psbt, min_confirmations, timeout_seconds).await
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
        let psbt = self.create_commitment_psbt_for_signing(
            state, 
            source_address, 
            change_address, 
            target_fee_rate
        ).await?;

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

    /// **COMPLETE WORKFLOW**: Broadcast and monitor signed PSBT
    /// 
    /// Completes the workflow started by `create_commitment_workflow()`:
    /// 1. Broadcasts the signed PSBT
    /// 2. Monitors for confirmation 
    /// 3. Returns final status
    
    pub async fn broadcast_and_monitor(
        &self,
        signed_psbt: &PsbtTransaction,
        min_confirmations: u32,
        timeout_seconds: u64,
    ) -> AnchorResult<(BroadcastResult, TransactionStatus)> {
        // Broadcast the transaction
        let broadcast_result = self.broadcast_signed_psbt(signed_psbt).await?;
        
        // Wait for confirmation
        let final_status = self.wait_for_confirmation(signed_psbt, min_confirmations, timeout_seconds).await?;
        
        Ok((broadcast_result, final_status))
    }

    /// **ERROR HANDLING**: Generate comprehensive error report with recovery suggestions
    pub fn analyze_error(&self, error: &AnchorError) -> ErrorReport {
        self.recovery_service.generate_error_report(error)
    }

    /// **RESILIENT EXECUTION**: Execute operation with automatic recovery
    
    pub async fn execute_with_recovery<F, T, Fut>(&self, operation: F, operation_name: &str) -> AnchorResult<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = AnchorResult<T>> + Send,
        T: Send,
    {
        self.recovery_service.execute_with_recovery(operation, operation_name).await
    }

    /// **RESILIENT WORKFLOW**: Create commitment with automatic error recovery
    
    pub async fn create_commitment_workflow_with_recovery(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>,
    ) -> AnchorResult<(PsbtTransaction, String, Option<ErrorReport>)> {
        let operation = || async {
            self.create_commitment_workflow(state, source_address, change_address, target_fee_rate).await
        };

        match self.execute_with_recovery(operation, "create_commitment_workflow").await {
            Ok((psbt, instructions)) => Ok((psbt, instructions, None)),
            Err(error) => {
                let error_report = self.analyze_error(&error);
                
                // Check if automatic recovery is possible
                if error_report.retryable && !error_report.automated_actions().is_empty() {
                    // Attempt one automatic recovery
                    match self.recovery_service.attempt_automatic_recovery(&error, || async {
                        self.create_commitment_workflow(state, source_address, change_address, target_fee_rate).await
                    }).await {
                        Ok(Some((psbt, instructions))) => Ok((psbt, instructions, Some(error_report))),
                        Ok(None) => Err(error), // Recovery failed
                        Err(recovery_error) => Err(recovery_error), // Recovery caused new error
                    }
                } else {
                    Err(error)
                }
            }
        }
    }

    /// **DIAGNOSTICS**: Check system health and network connectivity
    
    pub async fn diagnose_system_health(&self) -> SystemHealthReport {
        let mut report = SystemHealthReport::new();

        // Test basic connectivity
        if let Some(client) = self.psbt_constructor.esplora_client.as_ref() {
            match client.get_block_height().await {
                Ok(height) => {
                    report.blockchain_connectivity = true;
                    report.current_block_height = Some(height);
                }
                Err(e) => {
                    report.blockchain_connectivity = false;
                    report.issues.push(format!("Blockchain connectivity failed: {}", e));
                }
            }

            // Test fee estimation
            match client.get_fee_estimates().await {
                Ok(estimates) => {
                    report.fee_estimation_available = true;
                    report.current_fee_rates = Some((estimates.fast, estimates.medium, estimates.slow));
                    
                    // Check for network congestion
                    if estimates.fast > 50.0 {
                        report.issues.push(format!("High network congestion: {} sat/vB for fast confirmation", estimates.fast));
                    }
                }
                Err(e) => {
                    report.fee_estimation_available = false;
                    report.issues.push(format!("Fee estimation failed: {}", e));
                }
            }
        } else {
            report.issues.push("No blockchain client configured".to_string());
        }

        // Check configuration
        if self.config.rgb_enabled {
            report.rgb_features_enabled = true;
        }

        report.network = self.config.network;
        report
    }
}
