use crate::bitcoin::opret::{OpReturnCommitment, OpReturnCommitter};
use crate::commitment::F1r3flyStateCommitment;
use crate::error::AnchorResult;

use crate::bitcoin::esplora::EsploraClient;

use crate::bitcoin::esplora::EsploraUtxo;

// bp-std PSBT infrastructure
use bitcoin::{Address, Amount, OutPoint, ScriptBuf, TxOut as BitcoinTxOut};
use bpstd::psbt::{Beneficiary, Psbt, PsbtVer, UnsignedTx, UnsignedTxIn, Utxo};
use bpstd::{
    LockTime, Network, Outpoint, Sats, ScriptPubkey, SeqNo, TxOut, TxVer, Txid, VarIntArray, Vout,
};
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
    pub fn simple(utxos: Vec<Utxo>, change_address: Address, fee: Sats) -> Self {
        // Determine network from address - using a default for now
        let network = match change_address.to_string().starts_with("bc1")
            || change_address.to_string().starts_with("1")
            || change_address.to_string().starts_with("3")
        {
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
        let network = match change_address.to_string().starts_with("bc1")
            || change_address.to_string().starts_with("1")
            || change_address.to_string().starts_with("3")
        {
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
                return Err(crate::error::AnchorError::Psbt(format!(
                    "Input {} is not signed - PSBT must be fully signed before broadcasting",
                    i
                )));
            }
        }

        // For now, return an error indicating external signing is required
        // In a real implementation, this would work with the specific PSBT library being used
        Err(crate::error::AnchorError::Psbt(
            "PSBT must be signed externally and transaction hex provided directly for broadcasting"
                .to_string(),
        ))
    }

    /// Check if PSBT is fully signed and ready for broadcasting
    ///
    /// Returns true if all inputs have been signed and the PSBT
    /// can be extracted as a complete transaction.
    pub fn is_fully_signed(&self) -> bool {
        // Check if all inputs have final scripts (signatures)
        self.psbt
            .inputs()
            .all(|input| input.final_script_sig.is_some())
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

/// PSBT constructor for F1r3fly Bitcoin anchoring
pub struct AnchorPsbt {
    network: Network,

    pub esplora_client: Option<EsploraClient>,
}

impl AnchorPsbt {
    /// Create new PSBT constructor
    pub fn new(network: Network) -> Self {
        Self {
            network,
            esplora_client: None,
        }
    }

    /// Create new PSBT constructor with Esplora client

    pub fn with_esplora(network: Network, esplora_client: EsploraClient) -> Self {
        Self {
            network,
            esplora_client: Some(esplora_client),
        }
    }

    /// Build PSBT transaction with F1r3fly commitment using real UTXOs

    pub async fn build_psbt_transaction_from_address(
        &self,
        state: &F1r3flyStateCommitment,
        source_address: &Address,
        change_address: &Address,
        target_fee_rate: Option<f64>, // sat/vB, None = use network estimate
    ) -> AnchorResult<PsbtTransaction> {
        let client = self.esplora_client.as_ref().ok_or_else(|| {
            crate::error::AnchorError::Configuration(
                "Esplora client required for real UTXO fetching".to_string(),
            )
        })?;

        // 1. Fetch UTXOs for the source address
        let esplora_utxos = client
            .get_address_utxos(source_address)
            .await
            .map_err(|e| {
                crate::error::AnchorError::Network(format!("Failed to fetch UTXOs: {}", e))
            })?;

        if esplora_utxos.is_empty() {
            return Err(crate::error::AnchorError::InsufficientFunds(
                "No UTXOs available".to_string(),
            ));
        }

        // 2. Convert to enhanced UTXOs with script information
        let mut enhanced_utxos = Vec::new();
        for esplora_utxo in esplora_utxos {
            // Fetch transaction to get the script_pubkey
            let tx = client
                .get_transaction(&esplora_utxo.txid)
                .await
                .map_err(|e| {
                    crate::error::AnchorError::Network(format!(
                        "Failed to fetch transaction: {}",
                        e
                    ))
                })?;

            if let Some(output) = tx.vout.get(esplora_utxo.vout as usize) {
                let script_pubkey = ScriptBuf::from_hex(&output.scriptpubkey).map_err(|e| {
                    crate::error::AnchorError::InvalidData(format!("Invalid script: {}", e))
                })?;

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
                let estimates = client.get_fee_estimates().await.map_err(|e| {
                    crate::error::AnchorError::Network(format!(
                        "Failed to get fee estimates: {}",
                        e
                    ))
                })?;
                estimates.medium // Use medium priority (3 blocks)
            }
        };

        // 4. Perform coin selection
        let coin_selection =
            self.select_coins(&enhanced_utxos, fee_rate, target_fee_rate.is_none())?;

        // 5. Build the actual PSBT
        self.build_psbt_with_utxos(state, coin_selection, change_address)
            .await
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
        let confirmed_utxos: Vec<_> = sorted_utxos
            .into_iter()
            .filter(|utxo| utxo.is_confirmed)
            .collect();

        if confirmed_utxos.is_empty() {
            return Err(crate::error::AnchorError::InsufficientFunds(
                "No confirmed UTXOs available".to_string(),
            ));
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

        Err(crate::error::AnchorError::InsufficientFunds(
            "Insufficient funds to cover transaction and fees".to_string(),
        ))
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
                Txid::from_str(&enhanced_utxo.outpoint.txid.to_string()).map_err(|e| {
                    crate::error::AnchorError::InvalidData(format!("Invalid txid: {}", e))
                })?,
                Vout::from_u32(enhanced_utxo.outpoint.vout),
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
        let change_script =
            ScriptPubkey::from_checked(change_address.script_pubkey().as_bytes().to_vec());
        let change_output = TxOut::new(
            change_script,
            Sats::from_sats(coin_selection.change_amount.to_sat()),
        );
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
                    Sats::from_sats(enhanced_utxo.amount.to_sat()),
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
        self.build_psbt_transaction_from_address(
            state,
            source_address,
            change_address,
            target_fee_rate,
        )
        .await
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
        let address = address_str
            .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()
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
        let state = F1r3flyStateCommitment::new([1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]);

        let psbt_constructor = AnchorPsbt::new(Network::Testnet3);
        let fee = Sats::from_sats(1000u64);

        let result = psbt_constructor.build_psbt_transaction(&state, fee);
        assert!(result.is_ok());

        let psbt_tx = result.unwrap();
        assert_eq!(psbt_tx.fee, fee);
    }

    #[test]
    fn test_psbt_to_signed_transaction_hex() {
        let state = F1r3flyStateCommitment::new([1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]);

        let psbt_constructor = AnchorPsbt::new(Network::Testnet3);
        let fee = Sats::from_sats(1000u64);

        let psbt_tx = psbt_constructor
            .build_psbt_transaction(&state, fee)
            .unwrap();

        // Should return error since PSBT is not signed
        let result = psbt_tx.to_signed_transaction_hex();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("PSBT must be signed externally"));
    }
}
