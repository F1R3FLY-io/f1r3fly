use crate::bitcoin::opret::{OpReturnCommitment, OpReturnCommitter};
use crate::error::AnchorResult;
use bitcoin::{Amount, OutPoint, Sequence, Transaction, TxIn, TxOut, Witness};

/// Bitcoin transaction builder for F1r3fly commitments
#[derive(Clone, Debug)]
pub struct CommitmentTransaction {
    pub transaction: Transaction,
    pub commitment_output_index: usize,
    pub total_fee: Amount,
}

impl CommitmentTransaction {
    /// Build Bitcoin transaction with OP_RETURN commitment
    pub fn build_with_opret_commitment(
        inputs: &[TxIn],
        outputs: &[TxOut],
        commitment: &OpReturnCommitment,
    ) -> AnchorResult<Self> {
        let mut tx = Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: inputs.to_vec(),
            output: outputs.to_vec(),
        };

        // Add the OP_RETURN commitment output
        let commitment_output = OpReturnCommitter::create_output(commitment)?;
        let commitment_output_index = tx.output.len();
        tx.output.push(commitment_output);

        // Calculate fee based on transaction size
        let total_fee = Self::calculate_fee(&tx)?;

        Ok(Self {
            transaction: tx,
            commitment_output_index,
            total_fee,
        })
    }

    /// Build transaction with just commitment (no other outputs)
    pub fn build_commitment_only(
        inputs: &[TxIn],
        change_output: TxOut,
        commitment: &OpReturnCommitment,
    ) -> AnchorResult<Self> {
        Self::build_with_opret_commitment(inputs, &[change_output], commitment)
    }

    /// Get transaction ID
    pub fn txid(&self) -> bitcoin::Txid {
        self.transaction.compute_txid()
    }

    /// Get transaction size in virtual bytes
    pub fn vsize(&self) -> usize {
        self.transaction.vsize()
    }

    /// Get transaction weight
    pub fn weight(&self) -> bitcoin::Weight {
        self.transaction.weight()
    }

    /// Get fee rate in sat/vbyte
    pub fn fee_rate(&self) -> f64 {
        let vsize = self.vsize();
        if vsize == 0 {
            0.0
        } else {
            self.total_fee.to_sat() as f64 / vsize as f64
        }
    }

    /// Calculate transaction fee
    /// Uses conservative fee rate of 10 sat/vbyte
    fn calculate_fee(tx: &Transaction) -> AnchorResult<Amount> {
        // Conservative fee rate for Bitcoin mainnet (10 sat/vbyte)
        // TODO: In production, this could be:
        // - Fetched from fee estimation APIs (mempool.space, blockstream, etc.)
        // - Configurable via AnchorConfig
        // - Dynamic based on mempool conditions
        const FEE_RATE_SAT_PER_VBYTE: u64 = 10;

        let vsize = tx.vsize() as u64;
        let fee_amount = vsize * FEE_RATE_SAT_PER_VBYTE;

        // Minimum fee check (Bitcoin relay fee is ~1000 sats)
        const MIN_FEE_SATS: u64 = 1000;
        let final_fee = fee_amount.max(MIN_FEE_SATS);

        Ok(Amount::from_sat(final_fee))
    }

    /// Get the commitment output
    pub fn commitment_output(&self) -> &TxOut {
        &self.transaction.output[self.commitment_output_index]
    }

    /// Verify that the transaction contains the expected commitment
    pub fn verify_commitment(&self, expected_commitment: &OpReturnCommitment) -> bool {
        let commitment_output = self.commitment_output();

        // Check that it's an OP_RETURN output
        if commitment_output.value != Amount::ZERO {
            return false;
        }

        let script_bytes = commitment_output.script_pubkey.as_bytes();
        if script_bytes.is_empty() || script_bytes[0] != bitcoin::opcodes::all::OP_RETURN.to_u8() {
            return false;
        }

        // Extract the data and compare with expected commitment
        if script_bytes.len() < 2 {
            return false;
        }

        let data_len = script_bytes[1] as usize;
        if script_bytes.len() < 2 + data_len {
            return false;
        }

        let embedded_data = &script_bytes[2..2 + data_len];
        embedded_data == expected_commitment.data()
    }
}

/// Helper for creating transaction inputs
#[derive(Clone, Debug)]
pub struct InputBuilder;

impl InputBuilder {
    /// Create a transaction input from UTXO
    pub fn from_utxo(txid: bitcoin::Txid, vout: u32) -> TxIn {
        TxIn {
            previous_output: OutPoint { txid, vout },
            script_sig: bitcoin::Script::new().into(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::default(),
        }
    }

    /// Create multiple inputs from UTXOs
    pub fn from_utxos(utxos: &[(bitcoin::Txid, u32)]) -> Vec<TxIn> {
        utxos
            .iter()
            .map(|(txid, vout)| Self::from_utxo(*txid, *vout))
            .collect()
    }
}

/// Helper for creating transaction outputs
#[derive(Clone, Debug)]
pub struct OutputBuilder;

impl OutputBuilder {
    /// Create a change output
    pub fn change_output(address: &bitcoin::Address, amount: Amount) -> AnchorResult<TxOut> {
        Ok(TxOut {
            value: amount,
            script_pubkey: address.script_pubkey(),
        })
    }

    /// Create a payment output
    pub fn payment_output(address: &bitcoin::Address, amount: Amount) -> AnchorResult<TxOut> {
        Ok(TxOut {
            value: amount,
            script_pubkey: address.script_pubkey(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::F1r3flyStateCommitment;
    use bitcoin::hashes::Hash;

    fn create_dummy_input() -> TxIn {
        InputBuilder::from_utxo(bitcoin::Txid::from_byte_array([1u8; 32]), 0)
    }

    fn create_dummy_output() -> TxOut {
        TxOut {
            value: Amount::from_sat(50000),
            script_pubkey: bitcoin::Script::new().into(),
        }
    }

    #[test]
    fn test_commitment_transaction_creation() {
        let state = F1r3flyStateCommitment::new([1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]);

        let commitment = OpReturnCommitter::create_commitment(&state).unwrap();

        let inputs = vec![create_dummy_input()];
        let outputs = vec![create_dummy_output()];

        let tx = CommitmentTransaction::build_with_opret_commitment(&inputs, &outputs, &commitment)
            .unwrap();

        // Should have original outputs plus commitment output
        assert_eq!(tx.transaction.input.len(), 1);
        assert_eq!(tx.transaction.output.len(), 2); // 1 original + 1 commitment

        // Commitment output should be last
        assert_eq!(tx.commitment_output_index, 1);

        // Should verify the commitment
        assert!(tx.verify_commitment(&commitment));
    }

    #[test]
    fn test_commitment_only_transaction() {
        let state = F1r3flyStateCommitment::new([1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]);

        let commitment = OpReturnCommitter::create_commitment(&state).unwrap();

        let inputs = vec![create_dummy_input()];
        let change_output = create_dummy_output();

        let tx = CommitmentTransaction::build_commitment_only(&inputs, change_output, &commitment)
            .unwrap();

        // Should have change output plus commitment output
        assert_eq!(tx.transaction.output.len(), 2);
        assert!(tx.verify_commitment(&commitment));
    }

    #[test]
    fn test_transaction_properties() {
        let state = F1r3flyStateCommitment::new([1u8; 32], [2u8; 32], 12345, 1234567890, [3u8; 32]);

        let commitment = OpReturnCommitter::create_commitment(&state).unwrap();
        let inputs = vec![create_dummy_input()];
        let outputs = vec![create_dummy_output()];

        let tx = CommitmentTransaction::build_with_opret_commitment(&inputs, &outputs, &commitment)
            .unwrap();

        // Should have valid transaction properties
        assert!(tx.vsize() > 0);
        assert!(tx.weight().to_wu() > 0);
        assert!(tx.fee_rate() > 0.0);

        // Should have valid txid
        let txid = tx.txid();
        assert_ne!(txid, bitcoin::Txid::from_byte_array([0u8; 32]));
    }

    #[test]
    fn test_input_output_builders() {
        let txid = bitcoin::Txid::from_byte_array([1u8; 32]);
        let input = InputBuilder::from_utxo(txid, 0);

        assert_eq!(input.previous_output.txid, txid);
        assert_eq!(input.previous_output.vout, 0);

        let utxos = vec![(txid, 0), (txid, 1)];
        let inputs = InputBuilder::from_utxos(&utxos);
        assert_eq!(inputs.len(), 2);
    }
}
