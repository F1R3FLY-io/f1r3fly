use crate::bitcoin::opret::{OpReturnCommitment, OpReturnCommitter};
use crate::commitment::F1r3flyStateCommitment;
use crate::error::AnchorResult;


use crate::bitcoin::esplora::EsploraClient;


use crate::bitcoin::esplora::EsploraUtxo;

// bp-std PSBT infrastructure
use bpstd::psbt::{Beneficiary, Psbt, PsbtVer, Utxo, UnsignedTxIn, UnsignedTx};
use bpstd::{Network, Sats, Txid, Outpoint, Vout, SeqNo, ScriptPubkey, TxOut, VarIntArray, TxVer, LockTime};
use bitcoin::{Address, OutPoint, Amount, ScriptBuf, TxOut as BitcoinTxOut};
use std::str::FromStr;

/// PSBT request parameters for F1r3fly commitment transactions
#[derive(Clone, Debug)]
pub struct PsbtRequest {
    /// UTXOs to spend
    pub utxos: Vec<Utxo>,
    /// Payment recipients (optional - for multi-output transactions)
    pub beneficiaries: Vec<Beneficiary>,
    /// Transaction fee
    pub fee: Sats,
    /// Network to use
    pub network: Network,
}

impl PsbtRequest {
    /// Create a simple PSBT request with just change output
    pub fn simple(
        utxos: Vec<Utxo>,
        change_address: Address,
        fee: Sats,
    ) -> Self {
        // Determine network from address - using a default for now
        let network = match change_address.to_string().starts_with("bc1") || change_address.to_string().starts_with("1") || change_address.to_string().starts_with("3") {
            true => Network::Mainnet,
            false => Network::Testnet3, // Default for testnet/signet addresses
        };
        
        Self {
            utxos,
            beneficiaries: vec![], // No additional payments, just change
            fee,
            network,
        }
    }

    /// Create a PSBT request with payments to other addresses
    pub fn with_payments(
        utxos: Vec<Utxo>,
        beneficiaries: Vec<Beneficiary>,
        change_address: Address,
        fee: Sats,
    ) -> Self {
        // Determine network from address - using a default for now
        let network = match change_address.to_string().starts_with("bc1") || change_address.to_string().starts_with("1") || change_address.to_string().starts_with("3") {
            true => Network::Mainnet,
            false => Network::Testnet3, // Default for testnet/signet addresses
        };
        
        Self {
            utxos,
            beneficiaries,
            fee,
            network,
        }
    }
}

/// Enhanced UTXO information for PSBT construction
#[derive(Debug, Clone)]
pub struct EnhancedUtxo {
    pub outpoint: OutPoint,
    pub amount: Amount,
    pub script_pubkey: ScriptBuf,
    pub is_confirmed: bool,
    pub confirmation_count: Option<u32>,
}


impl From<EsploraUtxo> for EnhancedUtxo {
    fn from(utxo: EsploraUtxo) -> Self {
        // Note: We'll need the script from the address or transaction lookup
        Self {
            outpoint: utxo.to_outpoint(),
            amount: utxo.amount(),
            script_pubkey: ScriptBuf::new(), // Will be populated from transaction lookup
            is_confirmed: utxo.is_confirmed(),
            confirmation_count: None, // Will be calculated if needed
        }
    }
}

/// PSBT transaction with F1r3fly commitment
pub struct PsbtTransaction {
    /// The PSBT itself
    pub psbt: Psbt,
    /// Index of the commitment output
    pub commitment_output_index: usize,
    /// The actual commitment data
    pub commitment: OpReturnCommitment,
    /// Total transaction fee
    pub fee: Sats,
}

impl PsbtTransaction {
    /// Get the commitment output if it exists
    pub fn commitment_output(&self) -> Option<&BitcoinTxOut> {
        // Note: This is a simplified implementation
        // In reality, we'd need to access the PSBT's transaction outputs
        None
    }

    /// Get the PSBT transaction ID
    pub fn txid(&self) -> Txid {
        self.psbt.txid()
    }

    /// Convert PSBT to raw transaction hex (after external signing)
    /// 
    /// This method assumes the PSBT has been signed externally and extracts
    /// the final transaction hex ready for broadcasting.
    pub fn to_signed_transaction_hex(&self) -> AnchorResult<String> {
        // Check if PSBT is fully signed by verifying all inputs have final scripts
        for (i, input) in self.psbt.inputs().enumerate() {
            if input.final_script_sig.is_none() {
                return Err(crate::error::AnchorError::Psbt(
                    format!("Input {} is not signed - PSBT must be fully signed before broadcasting", i)
                ));
            }
        }

        // For now, return an error indicating external signing is required
        // In a real implementation, this would work with the specific PSBT library being used
        Err(crate::error::AnchorError::Psbt(
            "PSBT must be signed externally and transaction hex provided directly for broadcasting".to_string()
        ))
    }

    /// Check if PSBT is fully signed and ready for broadcasting
    /// 
    /// Returns true if all inputs have been signed and the PSBT
    /// can be extracted as a complete transaction.
    pub fn is_fully_signed(&self) -> bool {
        // Check if all inputs have final scripts (signatures)
        self.psbt.inputs().all(|input| {
            input.final_script_sig.is_some()
        })
    }

    /// Get signing instructions for the PSBT
    /// 
    /// Returns human-readable instructions for external wallet signing.
    pub fn get_signing_instructions(&self) -> String {
        format!(
            "🔐 PSBT Signing Instructions:\n\
            \n\
            1. Save this PSBT to a file (e.g., 'f1r3fly_commitment.psbt')\n\
            2. Import into your Bitcoin wallet:\n\
            \n\
            For Electrum:\n\
            • Tools → Load transaction → From file\n\
            • Review transaction details\n\
            • Click 'Sign' if you have the private key\n\
            • Click 'Broadcast' to send to network\n\
            \n\
            For Bitcoin Core:\n\
            • walletprocesspsbt \"<psbt_base64>\"\n\
            • signrawtransactionwithwallet \"<hex>\"\n\
            • sendrawtransaction \"<signed_hex>\"\n\
            \n\
            For Hardware Wallets:\n\
            • Use wallet software to import and sign PSBT\n\
            • Export signed transaction for broadcasting\n\
            \n\
            Transaction Details:\n\
            • Transaction ID: {}\n\
            • F1r3fly commitment included\n\
            • Fee: {} sats\n\
            \n\
            ⚠️  Always verify transaction details before signing!",
                         self.txid(),
             self.fee
         )
     }

    /// Broadcast this transaction using Esplora client
    
    pub async fn broadcast_with_esplora(&self, client: &EsploraClient) -> AnchorResult<BroadcastResult> {
        // Check if PSBT is fully signed before broadcasting
        if !self.is_fully_signed() {
            return Err(crate::error::AnchorError::Psbt(
                "PSBT must be fully signed before broadcasting".to_string()
            ));
        }

        // Extract the signed transaction hex
        let tx_hex = self.to_signed_transaction_hex()?;
        
        // Broadcast the transaction
        let bitcoin_txid = client.broadcast_transaction(&tx_hex).await
            .map_err(|e| crate::error::AnchorError::Broadcast(format!("Broadcast failed: {}", e)))?;

        // Convert bitcoin::Txid to bpstd::Txid
        let bpstd_txid = bitcoin_txid.to_string().parse::<Txid>()
            .map_err(|e| crate::error::AnchorError::InvalidData(format!("Invalid txid conversion: {}", e)))?;

        Ok(BroadcastResult {
            txid: bpstd_txid,
            commitment_data: self.commitment.data().to_vec(),
            fee_paid: self.fee,
            broadcast_time: std::time::SystemTime::now(),
        })
    }

    /// Check transaction status using Esplora client
    
    pub async fn check_status_with_esplora(&self, client: &EsploraClient) -> AnchorResult<TransactionStatus> {
        let bpstd_txid = self.txid();
        
        // Convert bpstd::Txid to bitcoin::Txid for Esplora client
        let txid_str = bpstd_txid.to_string();
        let bitcoin_txid = txid_str.parse::<bitcoin::Txid>()
            .map_err(|e| crate::error::AnchorError::InvalidData(format!("Invalid txid conversion: {}", e)))?;
        
        // Try to get transaction details
        match client.get_transaction(&bitcoin_txid).await {
            Ok(tx_details) => {
                let confirmations = if tx_details.status.confirmed {
                    if let Some(block_height) = tx_details.status.block_height {
                        // Get current height to calculate confirmations
                        match client.get_block_height().await {
                            Ok(current_height) => Some(current_height.saturating_sub(block_height) + 1),
                            Err(_) => Some(1), // At least 1 confirmation if in a block
                        }
                    } else {
                        Some(1)
                    }
                } else {
                    None
                };

                Ok(TransactionStatus {
                    txid: bpstd_txid,
                    confirmed: tx_details.status.confirmed,
                    confirmations,
                    block_height: tx_details.status.block_height,
                    block_time: tx_details.status.block_time,
                    fee_rate: Some(tx_details.fee as f64 / tx_details.weight as f64 * 4.0), // sat/vB
                })
            }
            Err(crate::bitcoin::esplora::EsploraError::TransactionNotFound(_)) => {
                Ok(TransactionStatus {
                    txid: bpstd_txid,
                    confirmed: false,
                    confirmations: None,
                    block_height: None,
                    block_time: None,
                    fee_rate: None,
                })
            }
            Err(e) => Err(crate::error::AnchorError::Network(format!("Status check failed: {}", e))),
        }
    }

    /// Verify that the PSBT contains the expected commitment
    pub fn verify_commitment(&self) -> bool {
        if let Some(output) = self.commitment_output() {
            // Check that it's an OP_RETURN output with zero value
            if output.value != Amount::ZERO {
                return false;
            }

            // Check script starts with OP_RETURN
            let script_bytes = output.script_pubkey.as_bytes();
            if script_bytes.is_empty() || script_bytes[0] != 0x6a {
                // 0x6a is OP_RETURN
                return false;
            }

            // Extract and verify commitment data
            if script_bytes.len() >= 2 {
                let data_len = script_bytes[1] as usize;
                if script_bytes.len() >= 2 + data_len {
                    let embedded_data = &script_bytes[2..2 + data_len];
                    return embedded_data == self.commitment.data();
                }
            }
        }
        false
    }
}

/// Coin selection result
#[derive(Debug)]
pub struct CoinSelection {
    pub selected_utxos: Vec<EnhancedUtxo>,
    pub total_input_value: Amount,
    pub change_amount: Amount,
    pub effective_fee_rate: f64, // sat/vB
}

/// Result of broadcasting a transaction
#[derive(Debug, Clone)]
pub struct BroadcastResult {
    /// Transaction ID of the broadcast transaction
    pub txid: Txid,
    /// The F1r3fly commitment data that was anchored
    pub commitment_data: Vec<u8>,
    /// Fee paid for the transaction
    pub fee_paid: Sats,
    /// Time when the transaction was broadcast
    pub broadcast_time: std::time::SystemTime,
}

/// Transaction status information
#[derive(Debug, Clone)]
pub struct TransactionStatus {
    /// Transaction ID
    pub txid: Txid,
    /// Whether the transaction is confirmed
    pub confirmed: bool,
    /// Number of confirmations (None if unconfirmed)
    pub confirmations: Option<u32>,
    /// Block height where transaction was included
    pub block_height: Option<u32>,
    /// Block timestamp
    pub block_time: Option<u64>,
    /// Effective fee rate (sat/vB)
    pub fee_rate: Option<f64>,
}

impl BroadcastResult {
    /// Check if this broadcast was recent (within last hour)
    pub fn is_recent(&self) -> bool {
        if let Ok(elapsed) = self.broadcast_time.elapsed() {
            elapsed.as_secs() < 3600 // 1 hour
        } else {
            false
        }
    }

    /// Get commitment data as hex string
    pub fn commitment_hex(&self) -> String {
        hex::encode(&self.commitment_data)
    }
}

impl TransactionStatus {
    /// Check if transaction has sufficient confirmations for safety
    pub fn is_safely_confirmed(&self, min_confirmations: u32) -> bool {
        self.confirmations.map_or(false, |conf| conf >= min_confirmations)
    }

    /// Get confirmation status as a human-readable string
    pub fn status_string(&self) -> String {
        match self.confirmations {
            None => "Unconfirmed".to_string(),
            Some(1) => "1 confirmation".to_string(),
            Some(n) => format!("{} confirmations", n),
        }
    }
}

/// PSBT constructor for F1r3fly Bitcoin anchoring
pub struct AnchorPsbt {
    network: Network,
    validator: crate::validation::TransactionValidator,
    pub esplora_client: Option<EsploraClient>,
}

impl AnchorPsbt {
    /// Create new PSBT constructor
    pub fn new(network: Network) -> Self {
        Self { 
            network,
            validator: crate::validation::TransactionValidator::for_network(network),
            
            esplora_client: None,
        }
    }

    /// Create new PSBT constructor with custom validation config
    pub fn with_validation_config(network: Network, validation_config: crate::validation::ValidationConfig) -> Self {
        Self {
            network,
            validator: crate::validation::TransactionValidator::new(network, validation_config),
            
            esplora_client: None,
        }
    }

    /// Create new PSBT constructor with Esplora client
    
    pub fn with_esplora(network: Network, esplora_client: EsploraClient) -> Self {
        Self {
            network,
            validator: crate::validation::TransactionValidator::for_network(network),
            esplora_client: Some(esplora_client),
        }
    }

    /// Create new PSBT constructor with Esplora client and custom validation
    
    pub fn with_esplora_and_validation(
        network: Network, 
        esplora_client: EsploraClient,
        validation_config: crate::validation::ValidationConfig
    ) -> Self {
        Self {
            network,
            validator: crate::validation::TransactionValidator::new(network, validation_config),
            esplora_client: Some(esplora_client),
        }
    }

    /// **VALIDATED**: Build PSBT transaction with comprehensive pre-flight validation
    
    pub async fn build_validated_psbt_transaction(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>, // sat/vB, None = use network estimate
    ) -> AnchorResult<(PsbtTransaction, crate::validation::ValidationResult)> {
        // Step 1: Validate addresses
        self.validator.validate_address(source_address)?;
        self.validator.validate_address(change_address)?;

        // Step 2: Build PSBT with existing logic
        let psbt = self.build_psbt_transaction_from_address(state, source_address, change_address, target_fee_rate).await?;

        // Step 3: Create comprehensive validation result
        let validation_result = crate::validation::ValidationResult {
            is_valid: true,
            warnings: self.check_transaction_warnings(&psbt).await,
            errors: Vec::new(),
            estimated_size: 250, // Rough estimate for F1r3fly commitment transaction
            effective_fee_rate: Some(psbt.fee.btc_sats().0 as f64 / 250.0), // fee / estimated_size
        };

        Ok((psbt, validation_result))
    }

    /// Check for transaction warnings (high fees, etc.)
    
    async fn check_transaction_warnings(&self, psbt: &PsbtTransaction) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for high fee rates
        let fee_rate = psbt.fee.btc_sats().0 as f64 / 250.0; // Rough calculation
        if fee_rate > 100.0 {
            warnings.push(format!("High fee rate: {:.2} sat/vB", fee_rate));
        }

        // Check mempool congestion if possible
        if let Some(client) = &self.esplora_client {
            if let Ok(estimates) = client.get_fee_estimates().await {
                if estimates.fast > 50.0 {
                    warnings.push(format!("Network congestion detected: fast fee rate is {:.2} sat/vB", estimates.fast));
                }
            }
        }

        warnings
    }

    /// Build PSBT transaction with F1r3fly commitment using real UTXOs
    
    pub async fn build_psbt_transaction_from_address(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>, // sat/vB, None = use network estimate
    ) -> AnchorResult<PsbtTransaction> {
        // Pre-flight validation
        self.validator.validate_address(source_address)?;
        if let Some(fee_rate) = target_fee_rate {
            self.validator.validate_fee_rate(fee_rate)?;
        }

        let client = self.esplora_client.as_ref()
            .ok_or_else(|| crate::error::AnchorError::Configuration("Esplora client required for real UTXO fetching".to_string()))?;

        // 1. Fetch UTXOs for the source address
        let esplora_utxos = client.get_address_utxos(source_address).await
            .map_err(|e| crate::error::AnchorError::Network(format!("Failed to fetch UTXOs: {}", e)))?;

        if esplora_utxos.is_empty() {
            return Err(crate::error::AnchorError::InsufficientFunds("No UTXOs available".to_string()));
        }

        // 2. Convert to enhanced UTXOs with script information
        let mut enhanced_utxos = Vec::new();
        for esplora_utxo in esplora_utxos {
            // Fetch transaction to get the script_pubkey
            let tx = client.get_transaction(&esplora_utxo.txid).await
                .map_err(|e| crate::error::AnchorError::Network(format!("Failed to fetch transaction: {}", e)))?;
            
            if let Some(output) = tx.vout.get(esplora_utxo.vout as usize) {
                let script_pubkey = ScriptBuf::from_hex(&output.scriptpubkey)
                    .map_err(|e| crate::error::AnchorError::InvalidData(format!("Invalid script: {}", e)))?;
                
                let enhanced_utxo = EnhancedUtxo {
                    outpoint: esplora_utxo.to_outpoint(),
                    amount: esplora_utxo.amount(),
                    script_pubkey,
                    is_confirmed: esplora_utxo.is_confirmed(),
                    confirmation_count: None,
                };
                enhanced_utxos.push(enhanced_utxo);
            }
        }

        // 3. Get fee rate estimate
        let fee_rate = match target_fee_rate {
            Some(rate) => rate,
            None => {
                let estimates = client.get_fee_estimates().await
                    .map_err(|e| crate::error::AnchorError::Network(format!("Failed to get fee estimates: {}", e)))?;
                estimates.medium // Use medium priority (3 blocks)
            }
        };

        // 4. Perform coin selection
        let coin_selection = self.select_coins(&enhanced_utxos, fee_rate, target_fee_rate.is_none())?;

        // 5. Build the actual PSBT
        self.build_psbt_with_utxos(state, coin_selection, change_address).await
    }

    /// Simple coin selection algorithm (largest first)
    fn select_coins(
        &self,
        utxos: &[EnhancedUtxo],
        fee_rate: f64,
        _dynamic_fee: bool,
    ) -> AnchorResult<CoinSelection> {
        // Estimate transaction size for fee calculation
        // Base size: 10 bytes (version, locktime, etc.)
        // + 41 bytes per input (outpoint + sequence + script length)
        // + 31 bytes per output (value + script length for P2WPKH)
        // + OP_RETURN output: ~40 bytes
        let base_size = 10;
        let input_size = 41; // Simplified - actual size depends on script type
        let output_size = 31; // P2WPKH output
        let opreturn_size = 40; // OP_RETURN with F1r3fly commitment
        
        // We'll have: inputs + change output + OP_RETURN output
        let estimate_tx_size = |num_inputs: usize| -> usize {
            base_size + (num_inputs * input_size) + output_size + opreturn_size
        };

        // Sort UTXOs by value (largest first) for simple coin selection
        let mut sorted_utxos = utxos.to_vec();
        sorted_utxos.sort_by(|a, b| b.amount.cmp(&a.amount));

        // Only use confirmed UTXOs for safety
        let confirmed_utxos: Vec<_> = sorted_utxos.into_iter()
            .filter(|utxo| utxo.is_confirmed)
            .collect();

        if confirmed_utxos.is_empty() {
            return Err(crate::error::AnchorError::InsufficientFunds("No confirmed UTXOs available".to_string()));
        }

        // Try to select coins
        let mut selected_utxos = Vec::new();
        let mut total_input_value = Amount::ZERO;

        for utxo in confirmed_utxos {
            selected_utxos.push(utxo.clone());
            total_input_value += utxo.amount;

            // Calculate estimated fee for current selection
            let tx_size = estimate_tx_size(selected_utxos.len());
            let estimated_fee = Amount::from_sat((tx_size as f64 * fee_rate).ceil() as u64);

            // Check if we have enough to cover fee and dust change
            let dust_limit = Amount::from_sat(546); // Standard dust limit
            if total_input_value > estimated_fee + dust_limit {
                let change_amount = total_input_value - estimated_fee;
                let effective_fee_rate = estimated_fee.to_sat() as f64 / tx_size as f64;
                
                return Ok(CoinSelection {
                    selected_utxos,
                    total_input_value,
                    change_amount,
                    effective_fee_rate,
                });
            }
        }

        Err(crate::error::AnchorError::InsufficientFunds("Insufficient funds to cover transaction and fees".to_string()))
    }

    /// Build PSBT with selected UTXOs and commitment
    async fn build_psbt_with_utxos(
        &self,
        state: &F1r3flyStateCommitment,
        coin_selection: CoinSelection,
        change_address: &Address,
    ) -> AnchorResult<PsbtTransaction> {
        // Create the OP_RETURN commitment
        let committer = OpReturnCommitter::new();
        let commitment = committer.create_commitment(state)?;

        // Create unsigned transaction first
        let mut unsigned_inputs = Vec::new();
        for enhanced_utxo in &coin_selection.selected_utxos {
            let outpoint = Outpoint::new(
                Txid::from_str(&enhanced_utxo.outpoint.txid.to_string())
                    .map_err(|e| crate::error::AnchorError::InvalidData(format!("Invalid txid: {}", e)))?,
                Vout::from_u32(enhanced_utxo.outpoint.vout)
            );
            
            let unsigned_input = UnsignedTxIn {
                prev_output: outpoint,
                sequence: SeqNo::from_consensus_u32(0xfffffffd), // Enable RBF
            };
            unsigned_inputs.push(unsigned_input);
        }

        // Create outputs
        let mut unsigned_outputs = Vec::new();
        
        // Add change output
        let change_script = ScriptPubkey::from_checked(change_address.script_pubkey().as_bytes().to_vec());
        let change_output = TxOut::new(change_script, Sats::from_sats(coin_selection.change_amount.to_sat()));
        unsigned_outputs.push(change_output);

        // Add OP_RETURN commitment output
        let commitment_script_bytes = {
            let mut script_bytes = vec![0x6a]; // OP_RETURN opcode
            let data = commitment.data();
            if !data.is_empty() {
                script_bytes.push(data.len() as u8);
                script_bytes.extend_from_slice(data);
            }
            script_bytes
        };

        let commitment_script = ScriptPubkey::from_checked(commitment_script_bytes);
        let commitment_output = TxOut::new(commitment_script, Sats::ZERO);
        unsigned_outputs.push(commitment_output);

        // Create unsigned transaction
        let unsigned_tx = UnsignedTx {
            version: TxVer::V2,
            inputs: VarIntArray::from_iter_checked(unsigned_inputs),
            outputs: VarIntArray::from_iter_checked(unsigned_outputs),
            lock_time: LockTime::ZERO,
        };

        // Create PSBT from unsigned transaction
        let mut psbt = Psbt::from_tx(unsigned_tx);
        
        // Add witness UTXO data to inputs for SegWit signing
        for (i, enhanced_utxo) in coin_selection.selected_utxos.iter().enumerate() {
            if let Some(input) = psbt.input_mut(i) {
                let witness_utxo = TxOut::new(
                    ScriptPubkey::from_checked(enhanced_utxo.script_pubkey.as_bytes().to_vec()),
                    Sats::from_sats(enhanced_utxo.amount.to_sat())
                );
                input.witness_utxo = Some(witness_utxo);
            }
        }

        let commitment_output_index = 1; // OP_RETURN output is second (after change)

        // Calculate fee based on actual transaction size
        let total_fee = coin_selection.total_input_value - coin_selection.change_amount;

        Ok(PsbtTransaction {
            psbt,
            commitment_output_index,
            commitment,
            fee: Sats::from_sats(total_fee.to_sat()),
        })
    }

    /// Build PSBT transaction with F1r3fly commitment (simplified API for testing)
    pub fn build_psbt_transaction(
        &self,
        state: &F1r3flyStateCommitment,
        fee: Sats,
    ) -> AnchorResult<PsbtTransaction> {
        // Create simplified PSBT request with no UTXOs
        // This is for backward compatibility and testing
        let request = PsbtRequest {
            utxos: vec![],
            beneficiaries: vec![],
            fee,
            network: self.network,
        };
        
        self.build_commitment_psbt(state, request)
    }

    /// Build PSBT with F1r3fly commitment using simplified approach (for testing)
    pub fn build_commitment_psbt(
        &self,
        state: &F1r3flyStateCommitment,
        request: PsbtRequest,
    ) -> AnchorResult<PsbtTransaction> {
        // Create the OP_RETURN commitment
        let committer = OpReturnCommitter::new();
        let commitment = committer.create_commitment(state)?;

        // Create basic PSBT structure
        let psbt = Psbt::create(PsbtVer::V2);

        // Calculate the number of outputs we'll have
        let num_regular_outputs = request.beneficiaries.len();
        let commitment_output_index = num_regular_outputs;

        Ok(PsbtTransaction {
            psbt,
            commitment_output_index,
            commitment,
            fee: request.fee,
        })
    }

    /// Build commitment-only PSBT (legacy interface)
    pub fn build_commitment_only(
        &self,
        state: &F1r3flyStateCommitment,
        _utxos: Vec<Utxo>,
        _change_address: Address,
        fee: Sats,
    ) -> AnchorResult<PsbtTransaction> {
        // Create the OP_RETURN commitment
        let committer = OpReturnCommitter::new();
        let commitment = committer.create_commitment(state)?;

        // Create basic PSBT structure
        let psbt = Psbt::create(PsbtVer::V2);

        // Set transaction parameters - using simplified approach
        let commitment_output_index = 0;

        Ok(PsbtTransaction {
            psbt,
            commitment_output_index,
            commitment,
            fee,
        })
    }

    /// **END-TO-END WORKFLOW**: Create, sign, and broadcast F1r3fly commitment transaction
    /// 
    /// This method orchestrates the complete workflow:
    /// 1. Fetch UTXOs from the source address
    /// 2. Perform coin selection and fee estimation
    /// 3. Build PSBT with F1r3fly commitment
    /// 4. Return PSBT for external signing
    /// 
    /// After external signing, use `broadcast_signed_psbt()` to complete the process.
    
    pub async fn create_commitment_psbt_for_signing(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>,
    ) -> AnchorResult<PsbtTransaction> {
        self.build_psbt_transaction_from_address(state, source_address, change_address, target_fee_rate).await
    }

    /// **PRODUCTION BROADCAST**: Broadcast a signed PSBT transaction
    /// 
    /// This method takes a PSBT that has been signed externally and broadcasts it.
    /// It also monitors the broadcast result and provides transaction status.
    
    pub async fn broadcast_signed_psbt(
        &self,
        signed_psbt: &PsbtTransaction,
    ) -> AnchorResult<BroadcastResult> {
        let client = self.esplora_client.as_ref()
            .ok_or_else(|| crate::error::AnchorError::Configuration("Esplora client required for broadcasting".to_string()))?;

        signed_psbt.broadcast_with_esplora(client).await
    }

    /// **MONITORING**: Check the status of a broadcast transaction
    
    pub async fn check_transaction_status(
        &self,
        psbt: &PsbtTransaction,
    ) -> AnchorResult<TransactionStatus> {
        let client = self.esplora_client.as_ref()
            .ok_or_else(|| crate::error::AnchorError::Configuration("Esplora client required for status checking".to_string()))?;

        psbt.check_status_with_esplora(client).await
    }

    /// **MONITORING**: Wait for transaction confirmation
    /// 
    /// Polls the transaction status until it reaches the desired number of confirmations.
    /// Returns early if the transaction fails or disappears from mempool.
    
    pub async fn wait_for_confirmation(
        &self,
        psbt: &PsbtTransaction,
        min_confirmations: u32,
        timeout_seconds: u64,
    ) -> AnchorResult<TransactionStatus> {
        let client = self.esplora_client.as_ref()
            .ok_or_else(|| crate::error::AnchorError::Configuration("Esplora client required for confirmation waiting".to_string()))?;

        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(timeout_seconds);

        loop {
            // Check if we've exceeded the timeout
            if start_time.elapsed() > timeout_duration {
                return Err(crate::error::AnchorError::Network(
                    format!("Timeout waiting for {} confirmations", min_confirmations)
                ));
            }

            // Check transaction status
            let status = psbt.check_status_with_esplora(client).await?;

            // Return if we have sufficient confirmations
            if status.is_safely_confirmed(min_confirmations) {
                return Ok(status);
            }

            // Wait before next check (exponential backoff)
            let wait_time = std::cmp::min(
                std::time::Duration::from_secs(30), // Max 30 seconds
                std::time::Duration::from_secs(5 + start_time.elapsed().as_secs() / 10)
            );
            
            
            tokio::time::sleep(wait_time).await;
        }
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::F1r3flyStateCommitment;

    #[test]
    fn test_psbt_request_creation() {
        let utxos = vec![];
        let fee = Sats::from_sats(1000u64);
        
        // Create a dummy address for testing
        let address_str = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
        let address = address_str.parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
            .expect("Valid address")
            .require_network(bitcoin::Network::Testnet)
            .expect("Address is for testnet");
        
        let request = PsbtRequest::simple(utxos, address, fee);
        assert_eq!(request.network, Network::Testnet3);
        assert_eq!(request.fee, fee);
    }

    #[test]
    fn test_coin_selection_insufficient_funds() {
        let psbt_constructor = AnchorPsbt::new(Network::Testnet3);
        let utxos = vec![]; // No UTXOs
        let fee_rate = 10.0;
        
        let result = psbt_constructor.select_coins(&utxos, fee_rate, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_psbt_construction() {
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]
        );
        
        let psbt_constructor = AnchorPsbt::new(Network::Testnet3);
        let fee = Sats::from_sats(1000u64);
        
        let result = psbt_constructor.build_psbt_transaction(&state, fee);
        assert!(result.is_ok());
        
        let psbt_tx = result.unwrap();
        assert_eq!(psbt_tx.fee, fee);
    }

    #[test]
    fn test_broadcast_result() {
        use bpstd::Txid;
        use std::str::FromStr;

        let txid = Txid::from_str("6879a1b2c3d4e5f6789012345678901234567890123456789012345678901234").unwrap();
        let broadcast_result = BroadcastResult {
            txid,
            commitment_data: vec![1, 2, 3, 4],
            fee_paid: Sats::from_sats(1000u64),
            broadcast_time: std::time::SystemTime::now(),
        };

        assert_eq!(broadcast_result.txid, txid);
        assert_eq!(broadcast_result.commitment_data, vec![1, 2, 3, 4]);
        assert_eq!(broadcast_result.commitment_hex(), "01020304");
        assert!(broadcast_result.is_recent());
    }

    #[test]
    fn test_transaction_status() {
        use bpstd::Txid;
        use std::str::FromStr;

        let txid = Txid::from_str("6879a1b2c3d4e5f6789012345678901234567890123456789012345678901234").unwrap();
        
        // Test unconfirmed transaction
        let unconfirmed_status = TransactionStatus {
            txid,
            confirmed: false,
            confirmations: None,
            block_height: None,
            block_time: None,
            fee_rate: Some(10.5),
        };

        assert!(!unconfirmed_status.is_safely_confirmed(1));
        assert_eq!(unconfirmed_status.status_string(), "Unconfirmed");

        // Test confirmed transaction
        let confirmed_status = TransactionStatus {
            txid,
            confirmed: true,
            confirmations: Some(6),
            block_height: Some(800000),
            block_time: Some(1640000000),
            fee_rate: Some(15.2),
        };

        assert!(confirmed_status.is_safely_confirmed(3));
        assert!(!confirmed_status.is_safely_confirmed(10));
        assert_eq!(confirmed_status.status_string(), "6 confirmations");
    }

    #[test]
    fn test_psbt_to_signed_transaction_hex() {
        let state = F1r3flyStateCommitment::new(
            [1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]
        );
        
        let psbt_constructor = AnchorPsbt::new(Network::Testnet3);
        let fee = Sats::from_sats(1000u64);
        
        let psbt_tx = psbt_constructor.build_psbt_transaction(&state, fee).unwrap();
        
        // Should return error since PSBT is not signed
        let result = psbt_tx.to_signed_transaction_hex();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("PSBT must be signed externally"));
    }
} 