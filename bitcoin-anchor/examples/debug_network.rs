//! Network diagnostic tool to debug timeout issues
//!
//! This example tests different network configurations to identify
//! the source of timeout issues with the Esplora API

use bitcoin_anchor::{EsploraClient, RetryConfig};
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Network Diagnostics for Bitcoin Testnet...\n");

    // Test 1: Basic connectivity with different timeouts
    println!("📡 Test 1: Testing different timeout configurations");
    
    let timeout_configs = vec![
        ("Conservative (30s)", Duration::from_secs(30)),
        ("Moderate (15s)", Duration::from_secs(15)),
        ("Aggressive (10s)", Duration::from_secs(10)),
        ("Very Aggressive (5s)", Duration::from_secs(5)),
    ];

    for (name, timeout) in timeout_configs {
        print!("   {} timeout... ", name);
        let start = Instant::now();
        
        let retry_config = RetryConfig {
            max_attempts: 1, // Single attempt for pure timeout testing
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 1.0,
            request_timeout: timeout,
        };
        
        let client = EsploraClient::testnet_with_retry(retry_config);
        
        match client.get_block_height().await {
            Ok(height) => {
                let elapsed = start.elapsed();
                println!("✅ SUCCESS in {:?} (block {})", elapsed, height);
            }
            Err(e) => {
                let elapsed = start.elapsed();
                println!("❌ FAILED in {:?} ({})", elapsed, e);
            }
        }
    }

    println!("\n🌐 Test 2: Testing different API endpoints");
    
    // Test 2: Try different endpoints
    let endpoints = vec![
        ("Mempool.space Testnet", "https://mempool.space/testnet/api"),
    ];

    let standard_config = RetryConfig {
        max_attempts: 1,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 1.0,
        request_timeout: Duration::from_secs(15),
    };

    for (name, url) in endpoints {
        print!("   {} ... ", name);
        let start = Instant::now();
        
        let client = EsploraClient::with_retry_config(url, standard_config.clone());
        
        match client.get_block_height().await {
            Ok(height) => {
                let elapsed = start.elapsed();
                println!("✅ SUCCESS in {:?} (block {})", elapsed, height);
            }
            Err(e) => {
                let elapsed = start.elapsed();
                println!("❌ FAILED in {:?} ({})", elapsed, e);
            }
        }
    }

    println!("\n⚡ Test 3: Testing fee estimation endpoints");
    
    let client = EsploraClient::testnet_with_retry(standard_config);
    
    print!("   Fee estimates... ");
    let start = Instant::now();
    match client.get_fee_estimates().await {
        Ok(estimates) => {
            let elapsed = start.elapsed();
            println!("✅ SUCCESS in {:?}", elapsed);
            println!("      Fast: {:.1} sat/vB", estimates.fast);
            println!("      Medium: {:.1} sat/vB", estimates.medium);
            println!("      Slow: {:.1} sat/vB", estimates.slow);
            
            if estimates.fast > 50.0 {
                println!("      ⚠️  High congestion detected (this causes 'ISSUES DETECTED')");
            }
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("❌ FAILED in {:?} ({})", elapsed, e);
        }
    }

    println!("\n🏥 Test 4: Understanding health check thresholds");
    println!("   Current fee threshold for 'ISSUES DETECTED': > 50.0 sat/vB");
    println!("   Recommendation: This threshold may be too low for testnet");
    println!("   Testnet often has higher fees than mainnet due to limited use");

    println!("\n📋 Summary:");
    println!("   • Network timeouts may be due to:");
    println!("     - Aggressive timeout settings (< 15s)");
    println!("     - Blockstream API rate limiting");
    println!("     - Network connectivity issues");
    println!("     - DNS resolution delays");
    println!("   • 'ISSUES DETECTED' is caused by high fee rates (> 50 sat/vB)");
    println!("   • This is normal for Bitcoin testnet during busy periods");

    Ok(())
} 