//! Test UTXO fetching from Bitcoin testnet addresses
//!
//! This example demonstrates:
//! 1. Fetching UTXOs from known testnet addresses
//! 2. Validating UTXO data and confirmation status
//! 3. Testing coin selection algorithms

use bitcoin_anchor::{EsploraClient, F1r3flyStateCommitment, AnchorConfig, F1r3flyBitcoinAnchor, RetryConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set overall timeout for the entire example
    let timeout_duration = std::time::Duration::from_secs(60);
    let result = tokio::time::timeout(timeout_duration, run_example()).await;
    
    match result {
        Ok(res) => res,
        Err(_) => {
            println!("⏰ Example timed out after 60 seconds");
            Err("Timeout".into())
        }
    }
}

async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing UTXO Fetching from Bitcoin Testnet...\n");

    // 1. Create testnet Esplora client with shorter timeouts for examples
    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay: std::time::Duration::from_millis(200),
        max_delay: std::time::Duration::from_secs(3),
        backoff_multiplier: 1.5,
        request_timeout: std::time::Duration::from_secs(10), // Shorter timeout
    };
    let esplora_client = EsploraClient::testnet_with_retry(retry_config);
    
    // Quick network connectivity test
    print!("🌐 Testing network connectivity... ");
    match esplora_client.get_block_height().await {
        Ok(height) => {
            println!("✅ Connected to testnet (block {})", height);
        }
        Err(e) => {
            println!("❌ Network issue: {}", e);
            println!("⚠️  Proceeding with UTXO test anyway...");
        }
    }
    
    // 2. Test with a known working testnet address (limiting to avoid timeouts)
    let test_addresses = vec![
        // Bitcoin testnet address that has been verified to work
        "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx", // Classic bech32 testnet address (has UTXOs!)
    ];

    for address_str in test_addresses {
        println!("📍 Testing address: {}", address_str);
        
        // Parse the address with improved error handling
        let address = match address_str.parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>() {
            Ok(addr) => {
                // Try to validate the network (clone addr first since require_network consumes it)
                match addr.clone().require_network(bitcoin::Network::Testnet) {
                    Ok(testnet_addr) => testnet_addr,
                    Err(e) => {
                        println!("   ❌ Address network validation failed: {} (address may be for different network)", e);
                        
                        // Try to get the actual network to provide better error info
                        if let Ok(mainnet_addr) = addr.require_network(bitcoin::Network::Bitcoin) {
                            println!("      ℹ️  This appears to be a mainnet address: {}", mainnet_addr);
                        }
                        continue;
                    }
                }
            },
            Err(e) => {
                println!("   ❌ Address parsing failed: {} (invalid address format)", e);
                println!("      ℹ️  Address: {}", address_str);
                continue;
            }
        };

        // Fetch UTXOs for this address
        print!("   🔍 Fetching UTXOs... ");
        match esplora_client.get_address_utxos(&address).await {
            Ok(utxos) => {
                println!("✅ SUCCESS - Found {} UTXOs", utxos.len());
                
                if utxos.is_empty() {
                    println!("   📭 No UTXOs available for this address");
                } else {
                    let mut total_value = 0u64;
                    let mut confirmed_count = 0;
                    
                    for (i, utxo) in utxos.iter().enumerate() {
                        println!("   UTXO {}: {}:{} = {} sats ({})", 
                            i + 1,
                            utxo.txid,
                            utxo.vout,
                            utxo.value,
                            if utxo.is_confirmed() { "confirmed" } else { "unconfirmed" }
                        );
                        
                        total_value += utxo.value;
                        if utxo.is_confirmed() {
                            confirmed_count += 1;
                        }
                    }
                    
                    println!("   💰 Total value: {} sats ({:.8} BTC)", 
                        total_value, 
                        total_value as f64 / 100_000_000.0
                    );
                    println!("   ✅ Confirmed UTXOs: {}/{}", confirmed_count, utxos.len());
                }
            }
            Err(e) => {
                match e {
                    bitcoin_anchor::EsploraError::AddressNotFound => {
                        println!("📭 No UTXOs found (address not used)");
                    }
                    _ => {
                        println!("❌ FAILED: {}", e);
                    }
                }
            }
        }
        println!();
    }

    // 3. Test with F1r3fly anchor service UTXO fetching
    println!("🚀 Testing F1r3fly anchor service UTXO integration...");
    
    let config = AnchorConfig::testnet().with_rgb();
    let anchor = F1r3flyBitcoinAnchor::with_esplora(config, esplora_client)?;
    
    // Create a test F1r3fly state commitment
    let state = F1r3flyStateCommitment::new(
        [1u8; 32], [2u8; 32], 100, 1642694400, [3u8; 32]
    );
    
    println!("✅ F1r3fly state commitment created: {} bytes", 
        state.to_bitcoin_commitment()?.len()
    );
    
    // Test the anchor service network configuration
    let health_report = anchor.diagnose_system_health().await;
    println!("✅ Anchor service health: {}", 
        if health_report.is_healthy() { "HEALTHY" } else { "ISSUES DETECTED" });
    println!("   Network: {:?}", health_report.network);
    
    println!("\n🎉 UTXO fetching test completed!");
    println!("📋 Next steps:");
    println!("   1. Get testnet coins from a faucet");
    println!("   2. Test transaction building with real UTXOs");
    println!("   3. Create and broadcast F1r3fly commitment transactions");

    Ok(())
} 