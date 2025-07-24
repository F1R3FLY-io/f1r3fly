//! Testnet Address Monitoring Tool
//!
//! This tool monitors a Bitcoin testnet address for funding progress
//! and provides guidance on using testnet faucets.
//!
//! USAGE:
//! cargo run --example monitor_testnet_address -- tb1q...your_address_here
//!
//! WORKFLOW:
//! 1. Provide your testnet address as argument
//! 2. Tool checks current balance and provides faucet guidance
//! 3. Monitors for incoming transactions
//! 4. Reports when funding targets are reached

use bitcoin_anchor::EsploraClient;
use bitcoin::{Address, Network};
use std::env;
use std::str::FromStr;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 F1r3fly Regtest Address Monitor");
    println!("===================================\n");

    // Step 1: Get address from command line arguments
    let args: Vec<String> = env::args().collect();
    
    let target_address = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("❌ Error: No regtest address provided");
        println!();
        println!("📋 USAGE:");
        println!("   cargo run --example monitor_regtest_address -- <your_regtest_address>");
        println!();
        println!("📝 EXAMPLES:");
        println!("   cargo run --example monitor_regtest_address -- bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx");
        println!("   cargo run --example monitor_regtest_address -- bcrt1q76a3ee8zr78443xf9v64lcfpanvqwp2apmclrz");
        println!();
        println!("💡 Need a regtest address? Use:");
        println!("   bitcoin-cli -regtest getnewaddress 'f1r3fly_demo' 'bech32'");
        println!();
        return Err("No address provided".into());
    };

    println!("📍 Monitoring Address");
    println!("--------------------");
    println!("🎯 Target address: {}", target_address);
    
    // Validate the address
    let address = match Address::from_str(&target_address) {
        Ok(addr) => {
            match addr.require_network(Network::Regtest) {
                Ok(regtest_addr) => {
                    println!("✅ Address validated for Bitcoin regtest");
                    println!("📋 Address type: {:?}", regtest_addr.address_type());
                    regtest_addr
                }
                Err(e) => {
                    println!("❌ Error: Address is not for regtest network");
                    println!("   Details: {}", e);
                    println!();
                    println!("💡 Make sure you're using a regtest address that starts with:");
                    println!("   • bcrt1... (bech32)");
                    return Err("Invalid network for address".into());
                }
            }
        }
        Err(e) => {
            println!("❌ Error: Invalid address format");
            println!("   Details: {}", e);
            println!();
            println!("💡 Please provide a valid Bitcoin testnet address");
            return Err("Invalid address format".into());
        }
    };
    
    println!("🔗 Network: Bitcoin Regtest");
    println!();

    // Step 2: Setup blockchain client
    println!("🔍 Current Balance Check");
    println!("------------------------");

    let client = EsploraClient::new("http://localhost:3002");

    let (mut current_sats, mut utxo_count) = check_balance(&client, &address).await?;
    
    let target_sats = 100_000u64; // 0.001 BTC target
    let minimum_sats = 50_000u64; // 0.0005 BTC minimum
    
    println!("💰 Current balance: {} sats ({:.8} BTC)", current_sats, current_sats as f64 / 100_000_000.0);
    println!("🎯 Target balance: {} sats ({:.8} BTC)", target_sats, target_sats as f64 / 100_000_000.0);
    println!("📦 Current UTXOs: {}", utxo_count);
    
    // Show detailed UTXO information
    if utxo_count > 0 {
        println!();
        println!("📋 UTXO Details:");
        match client.get_address_utxos(&address).await {
            Ok(utxos) => {
                for (i, utxo) in utxos.iter().enumerate() {
                    println!("   {}. {}:{} = {} sats ({})",
                        i + 1,
                        &utxo.txid.to_string()[..8],
                        utxo.vout,
                        utxo.value,
                        if utxo.status.confirmed { "✅ confirmed" } else { "⏳ unconfirmed" }
                    );
                    
                    // Add block explorer link for unconfirmed transactions
                    if !utxo.status.confirmed {
                        println!("      🔍 Track on block explorer:");
                        println!("      • http://localhost:5000/tx/{}", utxo.txid);
                    }
                }
            }
            Err(e) => {
                println!("   ⚠️  Could not fetch UTXO details: {}", e);
            }
        }
    }
    println!();
    
    // Step 3: Status assessment with confirmation check
    let (confirmed_sats, unconfirmed_sats) = match client.get_address_utxos(&address).await {
        Ok(utxos) => {
            let confirmed: u64 = utxos.iter()
                .filter(|utxo| utxo.status.confirmed)
                .map(|utxo| utxo.value)
                .sum();
            let unconfirmed: u64 = utxos.iter()
                .filter(|utxo| !utxo.status.confirmed)
                .map(|utxo| utxo.value)
                .sum();
            (confirmed, unconfirmed)
        }
        Err(_) => (0, current_sats), // Fallback: assume all unconfirmed
    };

    // Check confirmed funds first (required for PSBT creation)
    if confirmed_sats >= target_sats {
        println!("🎉 FULLY FUNDED & CONFIRMED!");
        println!("   ✅ {} sats confirmed (target: {} sats)", confirmed_sats, target_sats);
        println!("   ✅ Ready to proceed with F1r3fly PSBT creation");
        println!();
        println!("🚀 Next Steps:");
        println!("   1. Create F1r3fly PSBT transaction");
        println!("   2. Sign transaction in your wallet");
        println!("   3. Broadcast to testnet");
        println!("   4. Monitor for confirmation");
        return Ok(());
    } else if confirmed_sats >= minimum_sats {
        println!("⚠️  PARTIALLY CONFIRMED");
        println!("   ✅ {} sats confirmed (minimum: {} sats)", confirmed_sats, minimum_sats);
        println!("   📊 Recommended: {} sats for optimal fees", target_sats);
        if unconfirmed_sats > 0 {
            println!("   ⏳ {} sats pending confirmation", unconfirmed_sats);
        }
        println!("   ✅ Can proceed with F1r3fly PSBT creation");
        println!();
        println!("🚀 Next Steps:");
        println!("   1. Create F1r3fly PSBT transaction");
        println!("   2. Sign transaction in your wallet");
        println!("   3. Broadcast to testnet");
        println!("   4. Monitor for confirmation");
        return Ok(());
    } else if current_sats >= minimum_sats && unconfirmed_sats > 0 {
        let needed_confirmed = minimum_sats.saturating_sub(confirmed_sats);
        
        if current_sats >= target_sats {
            println!("💰 FULLY FUNDED - WAITING FOR CONFIRMATIONS");
            println!("   ✅ Total balance: {} sats (exceeds target!)", current_sats);
        } else {
            println!("⏳ FUNDED ABOVE MINIMUM - WAITING FOR CONFIRMATIONS");
            println!("   💰 Total balance: {} sats", current_sats);
        }
        
        println!("   ✅ Confirmed: {} sats", confirmed_sats);
        println!("   ⏳ Unconfirmed: {} sats", unconfirmed_sats);
        println!();
        println!("❌ CANNOT PROCEED YET - Need {} more confirmed sats", needed_confirmed);
        println!("   F1r3fly PSBT requires minimum {} confirmed sats", minimum_sats);
        println!("   Currently waiting for {} sats to confirm", unconfirmed_sats);
        println!();
        println!("⏰ SOLUTION: Wait for confirmations (10-30 minutes)");
        println!("   • Use block explorer links above to track progress");
        println!("   • Once {} more sats confirm, you can proceed!", needed_confirmed);
        println!("   • Re-run this tool to check status");
        return Ok(());
    } else {
        println!("❌ NEEDS MORE FUNDING");
        println!("   💰 Current total: {} sats", current_sats);
        if confirmed_sats > 0 {
            println!("   ✅ {} sats confirmed", confirmed_sats);
        }
        if unconfirmed_sats > 0 {
            println!("   ⏳ {} sats unconfirmed", unconfirmed_sats);
        }
        println!("   🎯 Need: {} sats total", target_sats);
        println!("   📊 Still need: {} more sats", target_sats.saturating_sub(current_sats));
    }
    println!();

    // Step 4: Faucet instructions (only if funding needed)
    println!("🚰 Testnet Faucet Instructions");
    println!("------------------------------");
    println!("Get free testnet Bitcoin from these faucets:");
    println!();

    let needed_sats = target_sats.saturating_sub(current_sats);
    let needed_btc = needed_sats as f64 / 100_000_000.0;
    
    println!("💡 AMOUNT NEEDED: {} sats ({:.8} BTC)", needed_sats, needed_btc);
    println!();

    println!("🥇 PRIMARY FAUCET: https://bitcoinfaucet.uo1.net/");
    println!("   📋 Steps:");
    println!("   1. Visit: https://bitcoinfaucet.uo1.net/");
    println!("   2. Enter address: {}", target_address);
    println!("   3. Complete captcha");
    println!("   4. Click 'Send testnet bitcoins'");
    println!("   5. Wait for confirmation (10-30 minutes)");
    println!("   💰 Amount: Usually 0.001-0.01 BTC per request");
    println!("   ⏰ Cooldown: 24 hours between requests");
    println!();

    println!("🥈 BACKUP FAUCET: https://coinfaucet.eu/en/btc-testnet/");
    println!("   📋 Steps:");
    println!("   1. Visit: https://coinfaucet.eu/en/btc-testnet/");
    println!("   2. Enter address: {}", target_address);
    println!("   3. Complete verification");
    println!("   4. Request coins");
    println!("   💰 Amount: Variable amounts");
    println!("   ⏰ Cooldown: Varies");
    println!();

    println!("⚡ PRO TIP: Try both faucets for faster funding!");
    println!();

    // Step 5: Start monitoring
    println!("⏰ Live Monitoring");
    println!("-----------------");
    println!("🔄 Starting automatic monitoring...");
    println!("   Checking every 10 seconds for new transactions");
    println!("   Press Ctrl+C to stop and exit");
    println!();

    // Step 6: Monitoring loop
    let mut check_count = 0;
    let max_checks = 180; // Check for 30 minutes (every 10 seconds)
    let initial_sats = current_sats;
    let initial_utxos = utxo_count;
    
    loop {
        check_count += 1;
        
        println!("🔄 Check #{}: Polling blockchain...", check_count);
        
        let (new_sats, new_utxo_count) = check_balance(&client, &address).await?;
        
        if new_sats > current_sats {
            let gained_sats = new_sats - current_sats;
            println!("🎉 NEW FUNDS DETECTED!");
            println!("   ⬆️  Balance increased by {} sats", gained_sats);
            println!("   💰 New balance: {} sats ({:.8} BTC)", new_sats, new_sats as f64 / 100_000_000.0);
            println!("   📈 UTXOs: {} (was {})", new_utxo_count, utxo_count);
            
            if new_sats >= target_sats {
                println!("🎊 TARGET REACHED!");
                println!("   Your address is now fully funded for F1r3fly testing");
                break;
            } else if new_sats >= minimum_sats && current_sats < minimum_sats {
                println!("✅ MINIMUM REACHED!");
                println!("   You have enough for basic testing");
                println!("   Continue monitoring for target amount or proceed with testing");
            }
            
            // Update current values for next iteration
            current_sats = new_sats;
            utxo_count = new_utxo_count;
            
            println!();
        } else if new_sats == current_sats && new_utxo_count > utxo_count {
            println!("🔄 UTXO UPDATE: New unconfirmed transaction detected");
            utxo_count = new_utxo_count;
        } else {
            println!("   ⏳ No new funds yet... (Balance: {} sats)", new_sats);
        }
        
        if check_count >= max_checks {
            println!("⏰ Monitoring timeout reached (30 minutes)");
            println!("   Continue checking manually or run this script again");
            break;
        }
        
        if new_sats >= target_sats {
            break;
        }
        
        // Wait 10 seconds before next check
        sleep(Duration::from_secs(10)).await;
    }

    println!();
    println!("📊 Final Status Report");
    println!("======================");
    
    let (final_sats, final_utxo_count) = check_balance(&client, &address).await?;
    let total_gained = final_sats.saturating_sub(initial_sats);
    
    println!("💰 Final balance: {} sats ({:.8} BTC)", final_sats, final_sats as f64 / 100_000_000.0);
    println!("📦 Total UTXOs: {}", final_utxo_count);
    
    if total_gained > 0 {
        println!("📈 Total gained during monitoring: {} sats", total_gained);
        println!("🎯 UTXOs added: {}", final_utxo_count.saturating_sub(initial_utxos));
    }
    
    if final_sats >= target_sats {
        println!("🎉 FUNDING COMPLETE!");
        println!("   ✅ Ready for F1r3fly transaction testing");
        println!("   🚀 Proceed to next step: Create PSBT transaction");
    } else if final_sats >= minimum_sats {
        println!("⚠️  PARTIALLY FUNDED");
        println!("   ✅ Minimum requirements met");
        println!("   💡 Can proceed with testing, but consider getting more funds");
    } else {
        println!("❌ STILL NEEDS FUNDING");
        println!("   💡 Try different faucets or wait longer for confirmations");
        println!("   🔄 Run this script again to continue monitoring");
    }
    
    println!();
    println!("🔗 Block Explorer:");
    println!("   • Check address: http://localhost:5000/address/{}", target_address);
    println!();
    println!("🔄 To monitor again, run:");
    println!("   cargo run --example monitor_regtest_address -- {}", target_address);

    Ok(())
}

async fn check_balance(client: &EsploraClient, address: &Address) -> Result<(u64, usize), Box<dyn std::error::Error>> {
    match client.get_address_utxos(address).await {
        Ok(utxos) => {
            let total_sats: u64 = utxos.iter().map(|u| u.value).sum();
            Ok((total_sats, utxos.len()))
        }
        Err(e) => {
            println!("   ⚠️  Network error: {}", e);
            Ok((0, 0)) // Return zero on error, will retry
        }
    }
} 