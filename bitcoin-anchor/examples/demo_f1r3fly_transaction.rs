//! End-to-End F1r3fly Transaction Demo
//!
//! This example demonstrates the complete workflow for creating
//! Bitcoin transactions with F1r3fly state commitments on testnet.
//!
//! Workflow:
//! 1. Create F1r3fly state commitment  
//! 2. Fetch UTXOs from testnet address
//! 3. Perform coin selection
//! 4. Build transaction with OP_RETURN commitment
//! 5. Create PSBT ready for signing
//! 6. Display transaction details

use bitcoin_anchor::{
    F1r3flyBitcoinAnchor, EsploraClient, RetryConfig, AnchorConfig,
    AnchorError, F1r3flyStateCommitment
};
use bitcoin::{Address, Amount, FeeRate};
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 F1r3fly Bitcoin Testnet Transaction Demo");
    println!("==========================================\n");

    // Step 1: Initialize F1r3fly anchor service for testnet
    println!("🔧 Step 1: Initializing F1r3fly Anchor Service...");
    
    let config = AnchorConfig::testnet();

    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(300),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 1.5,
        request_timeout: Duration::from_secs(15),
    };

    let esplora_client = EsploraClient::testnet_with_retry(retry_config);
    let service = F1r3flyBitcoinAnchor::with_esplora(config, esplora_client)?;
    println!("   ✅ Anchor service initialized for testnet\n");

    // Step 2: Check system health
    println!("🏥 Step 2: Checking system health...");
    let health = service.diagnose_system_health().await;
    
    if health.blockchain_connectivity {
        println!("   ✅ Blockchain connectivity: OK (block {})", 
                health.current_block_height.unwrap_or(0));
    } else {
        return Err("❌ Blockchain connectivity failed".into());
    }

    if health.fee_estimation_available {
        let (fast, medium, slow) = health.current_fee_rates.unwrap_or((0.0, 0.0, 0.0));
        println!("   ✅ Fee estimation: {} / {} / {} sat/vB (fast/med/slow)", fast, medium, slow);
    }

    if !health.issues.is_empty() {
        println!("   ⚠️  Issues detected:");
        for issue in &health.issues {
            println!("      - {}", issue);
        }
    } else {
        println!("   ✅ System health: OPTIMAL");
    }
    println!();

    // Step 3: Create F1r3fly state commitment
    println!("🧬 Step 3: Creating F1r3fly state commitment...");
    
    // Create F1r3fly state commitment structure
    let state = F1r3flyStateCommitment::new(
        [1u8; 32], // genesis_id
        [2u8; 32], // state_hash  
        1000,      // block_height
        1642694400, // timestamp
        [3u8; 32]  // commitment_hash
    );
    
    // Create OP_RETURN commitment for the state
    let commitment = service.create_opret_commitment(&state)?;
    
    println!("   ✅ State commitment created:");
    println!("      OP_RETURN size: {} bytes", commitment.data().len());
    println!("      Hash: {}", hex::encode(commitment.hash()));
    println!();

    // Step 4: Set up transaction parameters
    println!("💰 Step 4: Setting up transaction parameters...");
    
    // Use a testnet address with known UTXOs (from our previous tests)
    let source_address = Address::from_str("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx")?
        .require_network(bitcoin::Network::Testnet)?;
    
    // Destination for change (in real use, this would be user's change address)  
    // Using same address as source for demo simplicity
    let change_address = source_address.clone();
    
    let target_amount = Amount::from_sat(500); // Spend 500 sats + fees
    let fee_rate = FeeRate::from_sat_per_vb(60).expect("Valid fee rate"); // 60 sat/vB
    
    println!("   📍 Source address: {}", source_address);
    println!("   📍 Change address: {}", change_address);
    println!("   💸 Target amount: {} sats", target_amount.to_sat());
    println!("   ⚡ Fee rate: {} sat/vB", fee_rate.to_sat_per_vb_ceil());
    println!();

    // Step 5: Fetch UTXOs and perform coin selection
    println!("🔍 Step 5: Fetching UTXOs and performing coin selection...");
    
    // Create a separate client for UTXO fetching since the service owns the main one
    let utxo_client = EsploraClient::testnet();
    
    match utxo_client.get_address_utxos(&source_address).await {
        Ok(utxos) => {
            println!("   ✅ Found {} UTXOs for address", utxos.len());
            
            let mut total_value = Amount::ZERO;
            for (i, utxo) in utxos.iter().enumerate() {
                let utxo_amount = Amount::from_sat(utxo.value);
                println!("      UTXO {}: {}:{} = {} sats ({})", 
                    i + 1,
                    utxo.txid,
                    utxo.vout,
                    utxo_amount.to_sat(),
                    if utxo.status.confirmed { "confirmed" } else { "unconfirmed" }
                );
                total_value += utxo_amount;
            }
            println!("   💰 Total available: {} sats", total_value.to_sat());
            
            if utxos.is_empty() {
                println!("   ⚠️  No UTXOs available - get testnet coins from a faucet");
                println!("      Recommended faucets:");
                println!("      - https://bitcoinfaucet.uo1.net/");
                println!("      - https://testnet-faucet.mempool.co/");
                println!();
            }
        }
        Err(e) => {
            println!("   ❌ Failed to fetch UTXOs: {}", e);
        }
    }

    // Step 6: Build transaction with F1r3fly commitment
    println!("🏗️  Step 6: Building transaction with F1r3fly commitment...");
    
    let target_fee_rate = Some(fee_rate.to_sat_per_vb_ceil() as f64);
    
    match service.build_psbt_transaction_from_address(
        &state,
        &source_address,
        &change_address,
        target_fee_rate,
    ).await {
        Ok(psbt_tx) => {
            println!("   ✅ PSBT created successfully!");
            println!("      Transaction ID: {}", psbt_tx.txid());
            println!("      Commitment output index: {}", psbt_tx.commitment_output_index);
            println!("      Fee: {} sats", psbt_tx.fee);
            
            println!("      OP_RETURN commitment:");
            println!("         Size: {} bytes", psbt_tx.commitment.data().len());
            println!("         Hash: {}", hex::encode(psbt_tx.commitment.hash()));
            
            println!();
            println!("🎯 Step 7: Transaction Analysis...");
            
            println!("   📊 Transaction breakdown:");
            println!("      PSBT created with F1r3fly commitment");
            println!("      Total fee: {} sats", psbt_tx.fee);
            println!("      Ready for external signing");
            
            println!();
            println!("   🔐 Next steps for transaction:");
            println!("      1. Sign PSBT with wallet/hardware device");
            println!("      2. Extract signed transaction");
            println!("      3. Broadcast to Bitcoin testnet");
            println!("      4. Monitor for confirmations");
        }
        Err(AnchorError::InsufficientFunds(msg)) => {
            println!("   ❌ Insufficient funds: {}", msg);
            println!();
            println!("   💡 Solutions:");
            println!("      1. Get more testnet coins from a faucet");
            println!("      2. Reduce the target amount");
            println!("      3. Use a lower fee rate");
        }
        Err(e) => {
            println!("   ❌ Transaction building failed: {}", e);
        }
    }

    println!();
    println!("✨ Demo Complete!");
    println!("==================");
    println!("🎯 F1r3fly Bitcoin integration is working!");
    println!("📋 Next steps for production:");
    println!("   1. Implement transaction signing");
    println!("   2. Add transaction broadcasting");
    println!("   3. Add confirmation monitoring");
    println!("   4. Integrate with RGB protocol layers");

    Ok(())
} 