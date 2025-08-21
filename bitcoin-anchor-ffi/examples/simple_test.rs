//! Simple test for Bitcoin anchor integration with real signet
//!
//! This example tests the complete Bitcoin integration flow:
//! 1. Creates a minimal F1r3fly commitment (fake data for testing)
//! 2. Uses real signet address and private key (hardcoded in FFI)
//! 3. Creates, signs, and broadcasts a real Bitcoin transaction
//! 4. Returns TXID for manual tracking on signet explorer

use bitcoin_anchor_ffi::BitcoinAnchorHandle;
use bitcoin_anchor_ffi::{
    anchor_finalization, create_bitcoin_anchor, deallocate_memory, destroy_bitcoin_anchor,
};
use models::bitcoin_anchor::{
    BitcoinAnchorConfigProto, BitcoinAnchorResultProto, F1r3flyStateCommitmentProto,
};
use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create minimal test commitment (fake F1r3fly data for testing)
    let test_commitment = create_minimal_commitment();
    println!("📝 Created test F1r3fly commitment:");
    println!("   Block height: {}", test_commitment.block_height);
    println!("   Timestamp: {}", test_commitment.timestamp);
    println!();

    // 2. Create Bitcoin anchor configuration for signet
    let config = create_signet_config();
    println!("⚙️  Created signet configuration");

    // 3. Create anchor handle
    let anchor_handle = create_test_anchor(&config)?;
    println!("🔧 Created Bitcoin anchor handle");
    println!();

    // 4. Test the complete Bitcoin anchor flow
    println!("🚀 Starting Bitcoin anchor process...");
    println!("   - Fetching real UTXOs from signet");
    println!("   - Creating PSBT transaction");
    println!("   - Signing with private key");
    println!("   - Broadcasting to Bitcoin network");
    println!();

    let result = test_anchor_finalization(anchor_handle, &test_commitment)?;

    // 5. Clean up anchor handle
    destroy_bitcoin_anchor(anchor_handle);

    // 6. Report results for manual tracking
    if result.success {
        println!("🎉 SUCCESS! Bitcoin transaction created and broadcast!");
        println!();
        println!("📊 Transaction Details:");
        println!("   🔗 TXID: {}", result.transaction_id);
        println!("   💸 Fee: {} sats", result.fee_sats);
        println!();
        println!("🌐 Track your transaction:");
        println!(
            "   📍 Primary: https://signet.bitcoinexplorer.org/tx/{}",
            result.transaction_id
        );
        println!(
            "   📍 Alternative: https://mempool.space/signet/tx/{}",
            result.transaction_id
        );
        println!();
        println!("ℹ️  Additional Info:");
        println!("{}", result.debug_info);
        println!();
        println!("✅ Test completed successfully!");
        println!("   Your real Bitcoin transaction is now on the signet blockchain!");
    } else {
        println!("❌ FAILED: {}", result.error_message);
        println!();
        println!("🔍 Debug Info:");
        println!("{}", result.debug_info);

        return Err(format!("Bitcoin anchor test failed: {}", result.error_message).into());
    }

    Ok(())
}

/// Create a minimal F1r3fly state commitment for testing
fn create_minimal_commitment() -> F1r3flyStateCommitmentProto {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate fake but valid F1r3fly commitment data
    let fake_lfb_hash = [0x01; 32]; // Fake last finalized block hash
    let fake_rspace_root = [0x02; 32]; // Fake RSpace root hash
    let fake_validator_set_hash = [0x03; 32]; // Fake validator set hash

    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    F1r3flyStateCommitmentProto {
        lfb_hash: fake_lfb_hash.to_vec(),
        rspace_root: fake_rspace_root.to_vec(),
        block_height: 12345, // Fake block height
        timestamp: current_time,
        validator_set_hash: fake_validator_set_hash.to_vec(),
    }
}

/// Create Bitcoin signet configuration
fn create_signet_config() -> BitcoinAnchorConfigProto {
    BitcoinAnchorConfigProto {
        network: "signet".to_string(),
        enabled: true,
        esplora_url: "https://signet.bitcoinexplorer.org/api".to_string(),
        fee_rate: 10.0,      // 10 sat/vB for signet
        max_fee_sats: 10000, // Max 10,000 sats fee
    }
}

/// Create Bitcoin anchor handle for testing
fn create_test_anchor(
    config: &BitcoinAnchorConfigProto,
) -> Result<*mut BitcoinAnchorHandle, Box<dyn std::error::Error>> {
    let config_bytes = config.encode_to_vec();

    let handle = create_bitcoin_anchor(config_bytes.as_ptr(), config_bytes.len());

    if handle.is_null() {
        return Err("Failed to create Bitcoin anchor handle".into());
    }

    Ok(handle)
}

/// Test the anchor finalization with error handling
fn test_anchor_finalization(
    handle: *mut BitcoinAnchorHandle,
    commitment: &F1r3flyStateCommitmentProto,
) -> Result<BitcoinAnchorResultProto, Box<dyn std::error::Error>> {
    let commitment_bytes = commitment.encode_to_vec();

    let result_ptr = anchor_finalization(handle, commitment_bytes.as_ptr(), commitment_bytes.len());

    if result_ptr.is_null() {
        return Err("Anchor finalization returned null pointer".into());
    }

    // Read the length-prefixed result
    let result_len = unsafe { *(result_ptr as *const i32) };
    let result_data = unsafe {
        std::slice::from_raw_parts((result_ptr as *const u8).offset(4), result_len as usize)
    };

    // Parse the protobuf result
    let result = BitcoinAnchorResultProto::decode(result_data)?;

    // Clean up the allocated memory
    deallocate_memory(result_ptr, (result_len + 4) as usize);

    Ok(result)
}
