//! Testnet Wallet Setup Helper
//!
//! This tool guides you through setting up a Bitcoin testnet wallet
//! for live F1r3fly transaction testing.
//!
//! WORKFLOW:
//! 1. Generate or import testnet address
//! 2. Fund address from testnet faucets  
//! 3. Verify funding with F1r3fly system
//! 4. Ready for live transaction testing
//!
//! REQUIREMENTS:
//! - Bitcoin Core, Electrum, or other wallet software
//! - Internet connection for testnet faucets
//! - ~0.001 BTC in testnet coins (free from faucets)

use bitcoin_anchor::{EsploraClient, RetryConfig};
use bitcoin::{Address, Network};
use std::str::FromStr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 F1r3fly Testnet Wallet Setup Helper");
    println!("=====================================\n");

    // Step 1: Wallet Software Options
    println!("📱 Step 1: Choose Your Wallet Software");
    println!("--------------------------------------");
    println!("You need Bitcoin wallet software that supports testnet. Options:");
    println!();
    println!("🥇 RECOMMENDED: Bitcoin Core");
    println!("   • Full node with complete testnet support");
    println!("   • Download: https://bitcoincore.org/en/download/");
    println!("   • Setup: Start with -testnet flag");
    println!("   • Commands: bitcoin-cli -testnet getnewaddress");
    println!();
    println!("🥈 ALTERNATIVE: Electrum");
    println!("   • Lightweight wallet with testnet support");
    println!("   • Download: https://electrum.org/#download");
    println!("   • Setup: electrum --testnet");
    println!("   • GUI: Wallet → Addresses → Receiving");
    println!();
    println!("🥉 OTHER OPTIONS:");
    println!("   • BlueWallet (mobile, testnet support)");
    println!("   • Sparrow Wallet (desktop, advanced features)");
    println!("   • Bitcoin CLI tools");
    println!();

    // Step 2: Address Generation Instructions
    println!("🏗️  Step 2: Generate Testnet Address");
    println!("------------------------------------");
    println!("Follow instructions for your chosen wallet:");
    println!();
    println!("📋 Bitcoin Core:");
    println!("   1. Start Bitcoin Core with: bitcoin-qt -testnet");
    println!("   2. Wait for sync (or use -prune=550 for faster setup)");
    println!("   3. Go to Receive tab");
    println!("   4. Click 'Create new receiving address'");
    println!("   5. Copy the address (starts with 'tb1' or 'n'/'m')");
    println!();
    println!("📋 Electrum:");
    println!("   1. Start Electrum with: electrum --testnet");
    println!("   2. Create new wallet or open existing");
    println!("   3. Go to Addresses tab");
    println!("   4. Right-click → Copy address");
    println!("   5. Use address starting with 'tb1'");
    println!();

    // Step 3: Address Validation
    println!("✅ Step 3: Validate Your Address");
    println!("--------------------------------");
    println!("Enter your testnet address to validate:");
    println!("(Address should start with 'tb1', 'n', 'm', or '2')");
    println!();
    
    // Read address from user input (simulation for now)
    println!("💡 FOR DEMO: Using example address");
    let example_address = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
    println!("   Example address: {}", example_address);
    
    match Address::from_str(example_address) {
        Ok(addr) => {
            match addr.require_network(Network::Testnet) {
                Ok(testnet_addr) => {
                    println!("   ✅ Address is valid for testnet");
                    println!("   📋 Address type: {:?}", testnet_addr.address_type());
                    println!("   🔗 Network: Bitcoin Testnet");
                }
                Err(e) => {
                    println!("   ❌ Address is not for testnet: {}", e);
                    return Err("Invalid network for address".into());
                }
            }
        }
        Err(e) => {
            println!("   ❌ Invalid address format: {}", e);
            return Err("Invalid address".into());
        }
    }
    println!();

    // Step 4: Funding Instructions
    println!("💰 Step 4: Fund Your Address");
    println!("-----------------------------");
    println!("Get free testnet Bitcoin from faucets:");
    println!();
    println!("🚰 RECOMMENDED FAUCETS:");
    println!("   1. https://bitcoinfaucet.uo1.net/");
    println!("      • Reliable, moderate amounts");
    println!("      • Usually 0.001-0.01 BTC per request");
    println!("      • 24-hour cooldown");
    println!();
    println!("   2. https://testnet-faucet.mempool.co/");
    println!("      • Mempool.space official faucet");
    println!("      • Smaller amounts, frequent requests");
    println!("      • Good for small tests");
    println!();
    println!("   3. https://coinfaucet.eu/en/btc-testnet/");
    println!("      • Alternative backup faucet");
    println!("      • Variable amounts available");
    println!();
    println!("📋 FUNDING PROCESS:");
    println!("   1. Visit faucet website");
    println!("   2. Enter your testnet address: {}", example_address);
    println!("   3. Complete captcha/verification");
    println!("   4. Wait for transaction confirmation (10-30 minutes)");
    println!("   5. Check balance in your wallet");
    println!();
    println!("🎯 TARGET AMOUNT: ~0.001 BTC (100,000 sats)");
    println!("   • Enough for multiple F1r3fly transactions");
    println!("   • Covers transaction fees (typically 1,000-5,000 sats)");
    println!("   • Leaves room for testing different scenarios");
    println!();

    // Step 5: Verification Setup
    println!("🔍 Step 5: Verify Funding");
    println!("-------------------------");
    println!("Check if your address has been funded:");
    println!();

    let retry_config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(300),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 1.5,
        request_timeout: Duration::from_secs(15),
    };

    let client = EsploraClient::testnet_with_retry(retry_config);
    let address = Address::from_str(example_address)?
        .require_network(Network::Testnet)?;

    println!("   🌐 Checking address balance...");
    match client.get_address_utxos(&address).await {
        Ok(utxos) => {
            if utxos.is_empty() {
                println!("   ⏳ No UTXOs found yet");
                println!("      • Address may not be funded yet");
                println!("      • Faucet transaction may still be confirming");
                println!("      • Try again in 10-30 minutes");
            } else {
                let total_sats: u64 = utxos.iter().map(|u| u.value).sum();
                println!("   ✅ Found {} UTXOs", utxos.len());
                println!("   💰 Total balance: {} sats ({:.8} BTC)", total_sats, total_sats as f64 / 100_000_000.0);
                
                for (i, utxo) in utxos.iter().enumerate() {
                    println!("      UTXO {}: {}:{} = {} sats ({})",
                        i + 1,
                        utxo.txid,
                        utxo.vout,
                        utxo.value,
                        if utxo.status.confirmed { "confirmed" } else { "unconfirmed" }
                    );
                }

                if total_sats >= 50_000 {
                    println!("   🎉 READY FOR F1R3FLY TRANSACTIONS!");
                    println!("      Your address has sufficient funds for testing");
                } else {
                    println!("   ⚠️  Low balance - consider getting more from faucets");
                    println!("      Recommended: At least 50,000 sats for reliable testing");
                }
            }
        }
        Err(e) => {
            println!("   ❌ Error checking address: {}", e);
            println!("      This might be a temporary network issue");
        }
    }
    println!();

    // Step 6: Next Steps
    println!("🚀 Step 6: Next Steps");
    println!("---------------------");
    println!("Once your address is funded:");
    println!();
    println!("1. ✅ Run verification again to confirm funding");
    println!("2. 🏗️  Create F1r3fly PSBT transaction");
    println!("3. 🔐 Sign transaction in your wallet");
    println!("4. 📡 Broadcast to testnet");
    println!("5. ⏰ Monitor for confirmation");
    println!();
    println!("📋 SAVE THESE DETAILS:");
    println!("   • Testnet address: {}", example_address);
    println!("   • Wallet software: [Your choice]");
    println!("   • Funding transaction: [Check wallet for TXID]");
    println!();
    println!("🔧 TROUBLESHOOTING:");
    println!("   • No funds after 1 hour? Try different faucet");
    println!("   • Wallet sync issues? Use -prune=550 flag");
    println!("   • Network errors? Check internet connection");
    println!();
    println!("✨ Ready to create your first F1r3fly Bitcoin transaction!");

    Ok(())
} 