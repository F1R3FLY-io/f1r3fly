//! Test Bitcoin testnet connectivity and API functionality
//!
//! This example demonstrates:
//! 1. Connecting to Bitcoin testnet via Esplora API
//! 2. Testing live blockchain data retrieval
//! 3. Verifying fee estimation and network status

use bitcoin_anchor::{EsploraClient, AnchorConfig, F1r3flyBitcoinAnchor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Bitcoin Testnet Connectivity...\n");

    // 1. Create testnet Esplora client
    let esplora_client = EsploraClient::testnet();
    println!("✅ Created Esplora testnet client: https://mempool.space/testnet/api");

    // 2. Test basic connectivity - get current block height
    print!("📡 Testing blockchain connectivity... ");
    match esplora_client.get_block_height().await {
        Ok(height) => {
            println!("✅ SUCCESS");
            println!("   Current testnet block height: {}", height);
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            return Err(e.into());
        }
    }

    // 3. Test fee estimation
    print!("💰 Testing fee estimation... ");
    match esplora_client.get_fee_estimates().await {
        Ok(estimates) => {
            println!("✅ SUCCESS");
            println!("   Fast (1 block): {:.1} sat/vB", estimates.fast);
            println!("   Medium (3 blocks): {:.1} sat/vB", estimates.medium);
            println!("   Slow (6 blocks): {:.1} sat/vB", estimates.slow);
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            return Err(e.into());
        }
    }

    // 4. Create F1r3fly anchor service with testnet
    print!("⚙️  Creating F1r3fly anchor service... ");
    let config = AnchorConfig::testnet().with_rgb();
    let anchor = F1r3flyBitcoinAnchor::with_esplora(config, esplora_client)?;
    println!("✅ SUCCESS");

    // 5. Test system health diagnostics
    print!("🏥 Running system health check... ");
    let health_report = anchor.diagnose_system_health().await;
    println!("✅ COMPLETE");
    println!("   {}", health_report.summary());
    
    if !health_report.is_healthy() {
        println!("⚠️  Issues detected:");
        for issue in &health_report.issues {
            println!("   - {}", issue);
        }
    }

    println!("\n🎉 Testnet connectivity test completed successfully!");
    println!("✅ Ready for Bitcoin testnet transaction creation");

    Ok(())
} 