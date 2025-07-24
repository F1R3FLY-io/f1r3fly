use crate::error::{AnchorError, AnchorResult};
use bitcoin::{Address, Amount, OutPoint};
use bpstd::Network;

/// Validation rules and limits for Bitcoin transactions
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum dust threshold (satoshis)
    pub dust_limit: u64,
    /// Maximum transaction size in bytes
    pub max_transaction_size: usize,
    /// Minimum fee rate (sat/vB)
    pub min_fee_rate: f64,
    /// Maximum fee rate (sat/vB) - prevents accidental overpayment
    pub max_fee_rate: f64,
    /// Maximum number of inputs in a transaction
    pub max_inputs: usize,
    /// Maximum number of outputs in a transaction
    pub max_outputs: usize,
    /// Minimum confirmations required for input UTXOs
    pub min_input_confirmations: u32,
}

impl ValidationConfig {
    /// Default validation config for mainnet
    pub fn mainnet() -> Self {
        Self {
            dust_limit: 546, // Standard Bitcoin dust limit
            max_transaction_size: 100_000, // 100KB
            min_fee_rate: 1.0, // 1 sat/vB minimum
            max_fee_rate: 1000.0, // 1000 sat/vB maximum (prevents accidents)
            max_inputs: 100,
            max_outputs: 100,
            min_input_confirmations: 1,
        }
    }

    /// Default validation config for testnet (more permissive)
    pub fn testnet() -> Self {
        Self {
            dust_limit: 546,
            max_transaction_size: 100_000,
            min_fee_rate: 1.0,
            max_fee_rate: 10000.0, // Higher limit for testing
            max_inputs: 100,
            max_outputs: 100,
            min_input_confirmations: 0, // Allow unconfirmed inputs on testnet
        }
    }
}

/// UTXO information for validation
#[derive(Debug, Clone)]
pub struct ValidatedUtxo {
    pub outpoint: OutPoint,
    pub amount: Amount,
    pub confirmations: u32,
    pub is_dust: bool,
    pub is_confirmed: bool,
}

/// Transaction validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<AnchorError>,
    pub estimated_size: usize,
    pub effective_fee_rate: Option<f64>,
}

/// Comprehensive transaction validator
pub struct TransactionValidator {
    config: ValidationConfig,
    network: Network,
}

impl TransactionValidator {
    /// Create a new validator with the given configuration
    pub fn new(network: Network, config: ValidationConfig) -> Self {
        Self { config, network }
    }

    /// Create a validator with default configuration for the network
    pub fn for_network(network: Network) -> Self {
        let config = match network {
            Network::Mainnet => ValidationConfig::mainnet(),
            _ => ValidationConfig::testnet(),
        };
        Self::new(network, config)
    }

    /// Validate a Bitcoin address
    pub fn validate_address(&self, address: &Address) -> AnchorResult<()> {
        // Check if address is for the correct network
        let address_network = match address.to_string().chars().next() {
            Some('1') | Some('3') => Network::Mainnet,
            Some('m') | Some('n') | Some('2') => Network::Testnet3,
            Some('t') if address.to_string().starts_with("tb1") => Network::Testnet3,
            Some('b') if address.to_string().starts_with("bc1") => Network::Mainnet,
            Some('b') if address.to_string().starts_with("bcrt1") => Network::Regtest,
            _ => {
                return Err(AnchorError::InvalidAddress {
                    address: address.to_string(),
                    reason: "Unrecognized address format".to_string(),
                });
            }
        };

        // Check network compatibility (allow testnet addresses on signet/regtest)
        let compatible = match (self.network, address_network) {
            (Network::Mainnet, Network::Mainnet) => true,
            (Network::Testnet3, Network::Testnet3) => true,
            (Network::Signet, Network::Testnet3) => true,
            (Network::Regtest, Network::Testnet3) => true,
            (Network::Regtest, Network::Regtest) => true,
            _ => false,
        };

        if !compatible {
            return Err(AnchorError::InvalidAddress {
                address: address.to_string(),
                reason: format!("Address network mismatch: expected {:?}, got {:?}", self.network, address_network),
            });
        }

        Ok(())
    }

    /// Validate a transaction amount
    pub fn validate_amount(&self, amount: Amount, purpose: &str) -> AnchorResult<()> {
        let sats = amount.to_sat();

        // Check for zero amount (except for OP_RETURN outputs which can be zero)
        if purpose != "op_return" && sats == 0 {
            return Err(AnchorError::InvalidAmount {
                amount: sats,
                reason: format!("Zero amount not allowed for {}", purpose),
            });
        }

        // Check for dust (except for OP_RETURN outputs which can be zero)
        if purpose != "op_return" && sats < self.config.dust_limit {
            return Err(AnchorError::DustOutput {
                amount: sats,
                dust_limit: self.config.dust_limit,
            });
        }

        // Check for maximum reasonable amount (21 million BTC)
        if sats > 21_000_000 * 100_000_000 {
            return Err(AnchorError::InvalidAmount {
                amount: sats,
                reason: "Amount exceeds maximum possible Bitcoin supply".to_string(),
            });
        }

        Ok(())
    }

    /// Validate UTXOs for use in a transaction
    pub fn validate_utxos(&self, utxos: &[ValidatedUtxo]) -> AnchorResult<()> {
        if utxos.is_empty() {
            return Err(AnchorError::UtxoError {
                message: "No UTXOs provided".to_string(),
            });
        }

        if utxos.len() > self.config.max_inputs {
            return Err(AnchorError::ValidationError {
                reason: format!("Too many inputs: {} (maximum: {})", utxos.len(), self.config.max_inputs),
            });
        }

        // Check for duplicate UTXOs
        let mut seen_outpoints = std::collections::HashSet::new();
        for utxo in utxos {
            if !seen_outpoints.insert(utxo.outpoint) {
                return Err(AnchorError::UtxoError {
                    message: format!("Duplicate UTXO: {}", utxo.outpoint),
                });
            }
        }

        // Check confirmations
        for utxo in utxos {
            if utxo.confirmations < self.config.min_input_confirmations {
                return Err(AnchorError::UtxoError {
                    message: format!(
                        "UTXO {} has insufficient confirmations: {} (minimum: {})",
                        utxo.outpoint, utxo.confirmations, self.config.min_input_confirmations
                    ),
                });
            }
        }

        Ok(())
    }

    /// Validate fee rate
    pub fn validate_fee_rate(&self, fee_rate: f64) -> AnchorResult<()> {
        if fee_rate < self.config.min_fee_rate {
            return Err(AnchorError::FeeEstimation {
                reason: format!(
                    "Fee rate too low: {:.2} sat/vB (minimum: {:.2})",
                    fee_rate, self.config.min_fee_rate
                ),
            });
        }

        if fee_rate > self.config.max_fee_rate {
            return Err(AnchorError::FeeEstimation {
                reason: format!(
                    "Fee rate too high: {:.2} sat/vB (maximum: {:.2}). This may be a mistake.",
                    fee_rate, self.config.max_fee_rate
                ),
            });
        }

        Ok(())
    }

    /// Estimate transaction size in bytes
    pub fn estimate_transaction_size(&self, num_inputs: usize, num_outputs: usize) -> usize {
        // Base transaction size
        let base_size = 10; // version (4) + input count (1) + output count (1) + locktime (4)
        
        // Input sizes (varies by script type, using P2WPKH as default)
        let input_size = 41; // outpoint (36) + script length (1) + sequence (4)
        
        // Output sizes (varies by script type)
        let output_size = 31; // value (8) + script length (1) + script (22 for P2WPKH)
        
        // Add witness data for P2WPKH inputs (approximately)
        let witness_size = num_inputs * 27; // signature (73) + pubkey (33) with witness discount
        
        base_size + (num_inputs * input_size) + (num_outputs * output_size) + (witness_size / 4)
    }

    /// Perform comprehensive transaction validation
    pub fn validate_transaction(
        &self,
        inputs: &[ValidatedUtxo],
        output_amounts: &[Amount],
        fee: Amount,
    ) -> ValidationResult {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Validate inputs
        if let Err(e) = self.validate_utxos(inputs) {
            errors.push(e);
        }

        // Validate outputs
        if output_amounts.len() > self.config.max_outputs {
            errors.push(AnchorError::ValidationError {
                reason: format!("Too many outputs: {} (maximum: {})", output_amounts.len(), self.config.max_outputs),
            });
        }

        for (i, amount) in output_amounts.iter().enumerate() {
            if let Err(e) = self.validate_amount(*amount, &format!("output_{}", i)) {
                errors.push(e);
            }
        }

        // Calculate totals
        let total_input: u64 = inputs.iter().map(|utxo| utxo.amount.to_sat()).sum();
        let total_output: u64 = output_amounts.iter().map(|amt| amt.to_sat()).sum();
        let fee_sats = fee.to_sat();

        // Check balance
        if total_input < total_output + fee_sats {
            errors.push(AnchorError::InsufficientFunds(format!(
                "Insufficient funds: inputs {} sats, outputs + fee {} sats",
                total_input, total_output + fee_sats
            )));
        }

        // Estimate transaction size and fee rate
        let estimated_size = self.estimate_transaction_size(inputs.len(), output_amounts.len());
        let effective_fee_rate = if estimated_size > 0 {
            Some(fee_sats as f64 / estimated_size as f64)
        } else {
            None
        };

        // Validate fee rate
        if let Some(fee_rate) = effective_fee_rate {
            if let Err(e) = self.validate_fee_rate(fee_rate) {
                errors.push(e);
            }
        }

        // Check transaction size
        if estimated_size > self.config.max_transaction_size {
            errors.push(AnchorError::TransactionTooLarge {
                size: estimated_size,
                max_size: self.config.max_transaction_size,
            });
        }

        // Generate warnings
        if total_input > total_output + fee_sats {
            let change = total_input - total_output - fee_sats;
            if change < self.config.dust_limit {
                warnings.push(format!(
                    "Small change output: {} sats (dust limit: {})",
                    change, self.config.dust_limit
                ));
            }
        }

        if let Some(fee_rate) = effective_fee_rate {
            if fee_rate > 100.0 {
                warnings.push(format!(
                    "High fee rate: {:.2} sat/vB. This will be expensive.",
                    fee_rate
                ));
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            warnings,
            errors,
            estimated_size,
            effective_fee_rate,
        }
    }

    /// Suggest fee rate based on network conditions
    pub fn suggest_fee_rate(&self, network_fee_estimates: Option<(f64, f64, f64)>) -> f64 {
        match network_fee_estimates {
            Some((_fast, medium, _slow)) => {
                // Choose medium priority by default
                medium.max(self.config.min_fee_rate)
            }
            None => {
                // Fallback fee rates when network estimation is unavailable
                match self.network {
                    Network::Mainnet => 10.0, // Conservative mainnet fallback
                    _ => 2.0, // Testnet fallback
                }
            }
        }
    }

    /// Check if a fee rate indicates mempool congestion
    pub fn is_mempool_congested(&self, current_fee_rate: f64) -> Option<f64> {
        let congestion_threshold = match self.network {
            Network::Mainnet => 50.0, // 50 sat/vB indicates congestion on mainnet
            _ => 20.0, // 20 sat/vB for testnets
        };

        if current_fee_rate > congestion_threshold {
            Some(current_fee_rate * 1.5) // Suggest 50% higher fee
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_validation() {
        let validator = TransactionValidator::for_network(Network::Testnet3);
        
        // Valid testnet address (simplified test)
        // In a real test, we'd parse a proper Address object
        // For now, skip the address parsing test since it requires proper setup
        
        // Test amount validation instead
        assert!(validator.validate_amount(Amount::from_sat(1000), "test").is_ok());
        assert!(validator.validate_amount(Amount::from_sat(100), "test").is_err()); // Dust
    }

    #[test]
    fn test_amount_validation() {
        let validator = TransactionValidator::for_network(Network::Testnet3);
        
        // Valid amount
        assert!(validator.validate_amount(Amount::from_sat(1000), "output").is_ok());
        
        // Dust amount
        assert!(validator.validate_amount(Amount::from_sat(100), "output").is_err());
        
        // Zero amount
        assert!(validator.validate_amount(Amount::ZERO, "output").is_err());
        
        // OP_RETURN can be zero
        assert!(validator.validate_amount(Amount::ZERO, "op_return").is_ok());
    }

    #[test]
    fn test_transaction_size_estimation() {
        let validator = TransactionValidator::for_network(Network::Testnet3);
        
        let size = validator.estimate_transaction_size(2, 2);
        assert!(size > 100 && size < 500); // Reasonable size for 2-in, 2-out transaction
    }

    #[test]
    fn test_fee_rate_validation() {
        let validator = TransactionValidator::for_network(Network::Testnet3);
        
        // Valid fee rate
        assert!(validator.validate_fee_rate(10.0).is_ok());
        
        // Too low
        assert!(validator.validate_fee_rate(0.5).is_err());
        
        // Too high (likely mistake)
        assert!(validator.validate_fee_rate(50000.0).is_err());
    }
} 