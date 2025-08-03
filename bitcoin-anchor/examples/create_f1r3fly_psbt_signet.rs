use bitcoin::Address;
use bitcoin_anchor::bitcoin::{AnchorPsbt, EsploraClient};
use bitcoin_anchor::commitment::F1r3flyStateCommitment;
use bpstd::Network;
use std::env;
use std::fs;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 F1r3fly PSBT Creator (Signet)");
    println!("=================================");
    println!("Creates a PSBT for F1r3fly commitment transaction on Bitcoin Signet");
    println!();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --example create_f1r3fly_psbt_signet -- <signet_address>");
        eprintln!("Example: cargo run --example create_f1r3fly_psbt_signet -- tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx");
        std::process::exit(1);
    }

    let address_str = &args[1];
    println!("🌐 Network: Bitcoin Signet");
    println!("📍 Source Address: {}", address_str);

    // Parse and validate address
    let address = match Address::from_str(address_str) {
        Ok(addr) => {
            if !addr.is_valid_for_network(bitcoin::Network::Signet) {
                eprintln!("❌ Address is not valid for Bitcoin Signet");
                eprintln!("   Signet addresses should start with 'tb1' (bech32)");
                std::process::exit(1);
            }
            addr.assume_checked()
        }
        Err(e) => {
            eprintln!("❌ Invalid address format: {}", e);
            std::process::exit(1);
        }
    };

    println!("✅ Address validated for signet");
    println!();

    // Setup Esplora client for Signet
    let client = EsploraClient::signet();

    // Step 1: Fetch UTXOs
    println!("🔍 Fetching UTXOs from Signet Esplora...");
    let utxos = client.get_address_utxos(&address).await?;

    if utxos.is_empty() {
        eprintln!("❌ No UTXOs found for address {}", address_str);
        eprintln!("   Make sure the address is funded from a signet faucet:");
        eprintln!("   • https://signet.bc-2.jp/");
        eprintln!("   • https://alt.signetfaucet.com/");
        eprintln!("   • https://faucet.signet.bech32.de/");
        std::process::exit(1);
    }

    let total_input = utxos.iter().map(|utxo| utxo.value).sum::<u64>();
    println!(
        "✅ Found {} UTXO(s) totaling {} sats",
        utxos.len(),
        total_input
    );

    for (i, utxo) in utxos.iter().enumerate() {
        println!(
            "   {}. {}:{} = {} sats ({})",
            i + 1,
            &utxo.txid.to_string()[..8],
            utxo.vout,
            utxo.value,
            if utxo.status.confirmed {
                "✅ confirmed"
            } else {
                "⏳ unconfirmed"
            }
        );
    }
    println!();

    // Step 2: Create mock F1r3fly state commitment
    println!("🎭 Creating Mock F1r3fly State...");
    let state = create_mock_f1r3fly_state();
    println!("✅ F1r3fly state commitment created");
    println!("   LFB hash: {}", hex::encode(&state.lfb_hash));
    println!("   RSpace root: {}", hex::encode(&state.rspace_root));
    println!("   Block height: {}", state.block_height);
    println!();

    // Step 3: Create PSBT with AnchorPsbt
    println!("🔨 Creating PSBT...");
    let psbt_constructor = AnchorPsbt::with_esplora(Network::Signet, client);

    let fee_rate = Some(50.0); // 50 sats/vbyte - fast for signet
    // let fee_rate = None; // Use Esplora's fee estimation
    let change_address = address.clone(); // Send change back to same address

    let psbt_transaction = psbt_constructor
        .build_psbt_transaction_from_address(&state, &address, &change_address, fee_rate)
        .await?;

    println!("✅ PSBT created successfully");
    println!("   Transaction ID: {}", psbt_transaction.txid());
    println!("   Fee: {} sats", psbt_transaction.fee.btc_sats().0);
    println!(
        "   Commitment output index: {}",
        psbt_transaction.commitment_output_index
    );
    println!();

    // Step 4: Export PSBT to file
    println!("💾 Exporting PSBT...");
    let psbt_filename = format!("f1r3fly_transaction_signet_{}.psbt", &address_str[..8]); // First 8 chars of address

    let psbt_base64 = psbt_transaction.psbt.to_string();
    fs::write(&psbt_filename, &psbt_base64)?;

    println!("✅ PSBT exported to: {}", psbt_filename);
    println!("   📄 File size: {} bytes", psbt_base64.len());
    println!(
        "   🔗 Base64 preview: {}...",
        &psbt_base64[..60.min(psbt_base64.len())]
    );
    println!();

    // Step 5: Next steps guidance
    println!("🚀 Next Steps:");
    println!("===============");
    println!("1. 📤 Import PSBT into Bitcoin Core (signet mode):");
    println!(
        "   bitcoin-cli -signet walletprocesspsbt \"{}\"",
        psbt_base64
    );
    println!("   Or use your preferred signet wallet");
    println!();
    println!("2. ✍️  Sign the transaction:");
    println!("   • Verify outputs look correct");
    println!("   • Sign with your signet private key");
    println!("   • Get the signed PSBT hex");
    println!();
    println!("3. 📡 Broadcast to Signet:");
    println!("   # Finalize and broadcast with Bitcoin Core");
    println!("   bitcoin-cli -signet finalizepsbt \"<signed_psbt_hex>\"");
    println!("   bitcoin-cli -signet sendrawtransaction \"<final_tx_hex>\"");
    println!();
    println!("4. 🔍 Verify commitment:");
    println!("   cargo run --example verify_f1r3fly_commitment -- <txid>");
    println!();
    println!("📁 PSBT file ready: {}", psbt_filename);
    println!("🌐 Signet Explorer: https://mempool.space/signet");

    Ok(())
}

fn create_mock_f1r3fly_state() -> F1r3flyStateCommitment {
    // Create mock F1r3fly state commitment
    // In real implementation, this would be derived from actual Casper finalization

    // Mock Last Finalized Block hash (32 bytes)
    let lfb_hash = [
        0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81, 0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7, 0xf8,
        0x09, 0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x71, 0x82, 0x93, 0xa4, 0xb5, 0xc6, 0xd7,
        0xe8, 0xf9,
    ];

    // Mock RSpace root hash (32 bytes)
    let rspace_root = [
        0xf9, 0xe8, 0xd7, 0xc6, 0xb5, 0xa4, 0x93, 0x82, 0x71, 0x60, 0x5f, 0x4e, 0x3d, 0x2c, 0x1b,
        0x0a, 0x09, 0xf8, 0xe7, 0xd6, 0xc5, 0xb4, 0xa3, 0x92, 0x81, 0x70, 0x6f, 0x5e, 0x4d, 0x3c,
        0x2b, 0x1a,
    ];

    // Mock Casper block height
    let block_height = 12345;

    // Mock finalization timestamp (updated for signet testing)
    let timestamp = 1699123456; // Mock Unix timestamp

    // Mock validator set hash (32 bytes)
    let validator_set_hash = [
        0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
        0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
        0x88, 0x99,
    ];

    F1r3flyStateCommitment::new(
        lfb_hash,
        rspace_root,
        block_height,
        timestamp,
        validator_set_hash,
    )
}
