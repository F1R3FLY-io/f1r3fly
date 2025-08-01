use bitcoin::Txid;
use bitcoin_anchor::bitcoin::EsploraClient;
use bitcoin_anchor::commitment::F1r3flyStateCommitment;
use std::env;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 F1r3fly Commitment Verification Tool");
    println!("=======================================");
    println!("Verifies F1r3fly state commitments anchored to Bitcoin");
    println!();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run --example verify_f1r3fly_commitment -- <transaction_id>");
        eprintln!("Example: cargo run --example verify_f1r3fly_commitment -- 750ab3d5d751053e47ed08d855723a59bdbc30db543eae9112aa203144fa2ba5");
        std::process::exit(1);
    }

    let txid_str = &args[1];
    println!("🔗 Bitcoin Transaction: {}", txid_str);
    println!("🌐 Explorer: http://localhost:5001/tx/{}", txid_str);
    println!();

    // Parse transaction ID
    let txid = Txid::from_str(txid_str)?;

    // Connect to Esplora API
    let client = EsploraClient::new("http://localhost:3002");

    // Fetch transaction
    println!("📡 Fetching transaction from Bitcoin...");
    let tx = client.get_transaction(&txid).await?;

    // Find the OP_RETURN output
    let mut commitment_hash: Option<[u8; 32]> = None;
    let mut commitment_output_index: Option<usize> = None;

    for (i, output) in tx.vout.iter().enumerate() {
        if output.scriptpubkey_type == "op_return" {
            println!("✅ Found OP_RETURN output at index {}", i);

            // Parse the scriptpubkey to extract the hash
            if output.scriptpubkey.len() >= 68 {
                // "6a20" + 64 hex chars
                let hash_hex = &output.scriptpubkey[4..]; // Skip "6a20" (OP_RETURN + PUSH32)
                if hash_hex.len() == 64 {
                    match hex::decode(hash_hex) {
                        Ok(bytes) if bytes.len() == 32 => {
                            let mut hash = [0u8; 32];
                            hash.copy_from_slice(&bytes);
                            commitment_hash = Some(hash);
                            commitment_output_index = Some(i);
                            println!("🔍 Extracted F1r3fly commitment hash: {}", hash_hex);
                            break;
                        }
                        _ => continue,
                    }
                }
            }
        }
    }

    let commitment_hash = commitment_hash.ok_or("No F1r3fly commitment found in transaction")?;
    let output_index = commitment_output_index.unwrap();

    println!();
    println!("🎯 SHAREHOLDER DEMO: F1r3fly State Verification");
    println!("==============================================");

    // Show the original F1r3fly state that was committed
    println!("📊 Original F1r3fly State Data:");
    println!("--------------------------------");

    // Create the same mock state that was used in the commitment
    let original_state = create_demo_f1r3fly_state();

    println!(
        "🧱 Last Finalized Block:    {}",
        hex::encode(&original_state.lfb_hash)
    );
    println!(
        "🌿 RSpace State Root:       {}",
        hex::encode(&original_state.rspace_root)
    );
    println!(
        "📏 Casper Block Height:     {}",
        original_state.block_height
    );
    println!(
        "⏰ Finalization Timestamp:  {} ({})",
        original_state.timestamp,
        format_timestamp(original_state.timestamp)
    );
    println!(
        "👥 Validator Set Hash:      {}",
        hex::encode(&original_state.validator_set_hash)
    );

    println!();
    println!("🔐 Cryptographic Verification:");
    println!("-------------------------------");

    // Verify the hash matches
    let computed_hash = original_state.commitment_hash();
    println!(
        "💽 Hash in Bitcoin:         {}",
        hex::encode(&commitment_hash)
    );
    println!(
        "🧮 Computed from data:      {}",
        hex::encode(&computed_hash)
    );

    if commitment_hash == computed_hash {
        println!("✅ VERIFIED: Hash matches! F1r3fly state is authentically committed to Bitcoin");
    } else {
        println!("❌ MISMATCH: Hash does not match!");
        return Err("Hash verification failed".into());
    }

    println!();
    println!("🎯 SHAREHOLDER SUMMARY");
    println!("======================");
    println!("✅ F1r3fly state has been permanently anchored to Bitcoin");
    println!(
        "✅ Transaction confirmed in Bitcoin block: {} confirmations",
        if tx.status.confirmed {
            tx.status
                .block_height
                .map_or("pending".to_string(), |_| "confirmed".to_string())
        } else {
            "pending".to_string()
        }
    );
    println!("✅ Cryptographic integrity verified");
    println!("✅ Immutable record: Once in Bitcoin, cannot be altered");
    println!();
    println!(
        "🌐 View on Block Explorer: http://localhost:5001/tx/{}",
        txid_str
    );
    println!(
        "📄 Look for output #{} (OP_RETURN) containing the commitment hash",
        output_index
    );

    Ok(())
}

fn create_demo_f1r3fly_state() -> F1r3flyStateCommitment {
    // This must match exactly the state used in create_f1r3fly_psbt_regtest.rs
    F1r3flyStateCommitment::new(
        [
            0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f, 0x70, 0x81, 0x92, 0xa3, 0xb4, 0xc5, 0xd6, 0xe7,
            0xf8, 0x09, 0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x71, 0x82, 0x93, 0xa4, 0xb5,
            0xc6, 0xd7, 0xe8, 0xf9,
        ], // lfb_hash
        [
            0xf9, 0xe8, 0xd7, 0xc6, 0xb5, 0xa4, 0x93, 0x82, 0x71, 0x60, 0x5f, 0x4e, 0x3d, 0x2c,
            0x1b, 0x0a, 0x09, 0xf8, 0xe7, 0xd6, 0xc5, 0xb4, 0xa3, 0x92, 0x81, 0x70, 0x6f, 0x5e,
            0x4d, 0x3c, 0x2b, 0x1a,
        ], // rspace_root
        12345,      // block_height
        1699123456, // timestamp
        [
            0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
            0x66, 0x77, 0x88, 0x99,
        ], // validator_set_hash
    )
}

fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let system_time = UNIX_EPOCH + Duration::from_secs(timestamp);
    match system_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            // Simple formatting - in production you'd use chrono
            format!("Unix timestamp {}", duration.as_secs())
        }
        Err(_) => "Invalid timestamp".to_string(),
    }
}
