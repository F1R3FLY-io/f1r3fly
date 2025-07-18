use crate::args::*;
use crate::f1r3fly_api::F1r3flyApi;
use std::fs;
use std::time::Instant;

pub async fn exploratory_deploy_command(
    args: &ExploratoryDeployArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Rholang code from file
    println!("📄 Reading Rholang from: {}", args.file.display());
    let rholang_code =
        fs::read_to_string(&args.file).map_err(|e| format!("Failed to read file: {}", e))?;
    println!("📊 Code size: {} bytes", rholang_code.len());

    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    // Execute the exploratory deployment
    println!("🚀 Executing Rholang code (exploratory deploy)...");

    // Display block hash if provided
    if let Some(block_hash) = &args.block_hash {
        println!("🧱 Using block hash: {}", block_hash);
    }

    // Display state hash preference
    if args.use_pre_state {
        println!("🔍 Using pre-state hash");
    } else {
        println!("🔍 Using post-state hash");
    }

    let start_time = Instant::now();

    match f1r3fly_api
        .exploratory_deploy(
            &rholang_code,
            args.block_hash.as_deref(),
            args.use_pre_state,
        )
        .await
    {
        Ok((result, block_info)) => {
            let duration = start_time.elapsed();
            println!("✅ Execution successful!");
            println!("⏱️  Time taken: {:.2?}", duration);
            println!("🧱 {}", block_info);
            println!("📊 Result:");
            println!("{}", result);
        }
        Err(e) => {
            println!("❌ Execution failed!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn deploy_command(args: &DeployArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Rholang code from file
    println!("📄 Reading Rholang from: {}", args.file.display());
    let rholang_code =
        fs::read_to_string(&args.file).map_err(|e| format!("Failed to read file: {}", e))?;
    println!("📊 Code size: {} bytes", rholang_code.len());

    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    let phlo_limit = if args.bigger_phlo {
        "5,000,000,000"
    } else {
        "50,000"
    };
    println!("💰 Using phlo limit: {}", phlo_limit);

    // Deploy the Rholang code
    println!("🚀 Deploying Rholang code...");
    let start_time = Instant::now();

    match f1r3fly_api
        .deploy(&rholang_code, args.bigger_phlo, "rholang")
        .await
    {
        Ok(deploy_id) => {
            let duration = start_time.elapsed();
            println!("✅ Deployment successful!");
            println!("⏱️  Time taken: {:.2?}", duration);
            println!("🆔 Deploy ID: {}", deploy_id);
        }
        Err(e) => {
            println!("❌ Deployment failed!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn propose_command(args: &ProposeArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    // Propose a block
    println!("📦 Proposing a new block...");
    let start_time = Instant::now();

    match f1r3fly_api.propose().await {
        Ok(block_hash) => {
            let duration = start_time.elapsed();
            println!("✅ Block proposed successfully!");
            println!("⏱️  Time taken: {:.2?}", duration);
            println!("🧱 Block hash: {}", block_hash);
        }
        Err(e) => {
            println!("❌ Block proposal failed!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn full_deploy_command(args: &DeployArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Rholang code from file
    println!("📄 Reading Rholang from: {}", args.file.display());
    let rholang_code =
        fs::read_to_string(&args.file).map_err(|e| format!("Failed to read file: {}", e))?;
    println!("📊 Code size: {} bytes", rholang_code.len());

    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    let phlo_limit = if args.bigger_phlo {
        "5,000,000,000"
    } else {
        "50,000"
    };
    println!("💰 Using phlo limit: {}", phlo_limit);

    // Deploy and propose
    println!("🚀 Deploying Rholang code and proposing a block...");
    let start_time = Instant::now();

    match f1r3fly_api
        .full_deploy(&rholang_code, args.bigger_phlo, "rholang")
        .await
    {
        Ok(block_hash) => {
            let duration = start_time.elapsed();
            println!("✅ Deployment and block proposal successful!");
            println!("⏱️  Time taken: {:.2?}", duration);
            println!("🧱 Block hash: {}", block_hash);
        }
        Err(e) => {
            println!("❌ Operation failed!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn is_finalized_command(
    args: &IsFinalizedArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    // Check if the block is finalized
    println!("🔍 Checking if block is finalized: {}", args.block_hash);
    println!(
        "⏱️  Will retry every {} seconds, up to {} times",
        args.retry_delay, args.max_attempts
    );
    let start_time = Instant::now();

    match f1r3fly_api
        .is_finalized(
            &args.block_hash,
            Some(args.max_attempts),
            Some(args.retry_delay),
        )
        .await
    {
        Ok(is_finalized) => {
            let duration = start_time.elapsed();
            if is_finalized {
                println!("✅ Block is finalized!");
            } else {
                println!(
                    "❌ Block is not finalized after {} attempts",
                    args.max_attempts
                );
            }
            println!("⏱️  Time taken: {:.2?}", duration);
        }
        Err(e) => {
            println!("❌ Error checking block finalization!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn bond_validator_command(
    args: &BondValidatorArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Bonding new validator to the network");
    println!("💰 Stake amount: {} REV", args.stake);

    // Initialize the F1r3fly API client for deploying
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    // Create the bonding Rholang code
    let bonding_code = format!(
        r#"new rl(`rho:registry:lookup`), poSCh, retCh, stdout(`rho:io:stdout`) in {{
  stdout!("About to lookup PoS contract...") |
  rl!(`rho:rchain:pos`, *poSCh) |
  for(@(_, PoS) <- poSCh) {{
    stdout!("About to bond...") |
    new deployerId(`rho:rchain:deployerId`) in {{
      @PoS!("bond", *deployerId, {}, *retCh) |
      for (@(result, message) <- retCh) {{
        stdout!(("Bond result:", result, "Message:", message))
      }}
    }}
  }}
}}"#,
        args.stake
    );

    println!("🚀 Deploying bonding transaction...");
    let start_time = Instant::now();

    // Deploy the bonding code
    match f1r3fly_api.deploy(&bonding_code, true, "rholang").await {
        Ok(deploy_id) => {
            let duration = start_time.elapsed();
            println!("✅ Bonding deploy successful!");
            println!("⏱️  Deploy time: {:.2?}", duration);
            println!("🆔 Deploy ID: {}", deploy_id);

            if args.propose {
                println!("📦 Proposing block to include bonding transaction...");
                let propose_start = Instant::now();

                match f1r3fly_api.propose().await {
                    Ok(block_hash) => {
                        let propose_duration = propose_start.elapsed();
                        println!("✅ Block proposed successfully!");
                        println!("⏱️  Propose time: {:.2?}", propose_duration);
                        println!("🧱 Block hash: {}", block_hash);
                    }
                    Err(e) => {
                        println!("❌ Block proposal failed!");
                        println!("Error: {}", e);
                        return Err(e);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Bonding deploy failed!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

pub async fn transfer_command(args: &TransferArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("💸 Initiating REV transfer");
    
    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    // Generate from_address from private key
    println!("🔍 Deriving sender address from private key...");
    let from_address = derive_rev_address_from_private_key(&args.private_key)?;
    
    // Validate addresses format
    validate_rev_address(&from_address)?;
    validate_rev_address(&args.to_address)?;
    
    // Convert REV to dust (1 REV = 100,000,000 dust)
    let amount_dust = args.amount * 100_000_000;
    
    println!("📋 Transfer Details:");
    println!("   From: {}", from_address);
    println!("   To: {}", args.to_address);
    println!("   Amount: {} REV ({} dust)", args.amount, amount_dust);
    
    // Generate Rholang transfer contract
    let rholang_code = generate_transfer_contract(&from_address, &args.to_address, amount_dust);
    
    println!("🚀 Deploying transfer contract...");
    let start_time = std::time::Instant::now();
    
    match f1r3fly_api.deploy(&rholang_code, false, "rholang").await {
        Ok(deploy_id) => {
            let duration = start_time.elapsed();
            println!("✅ Transfer contract deployed successfully!");
            println!("⏱️  Time taken: {:.2?}", duration);
            println!("🆔 Deploy ID: {}", deploy_id);
            
            if args.propose {
                println!("📦 Proposing block to finalize transfer...");
                let propose_start = std::time::Instant::now();
                
                match f1r3fly_api.propose().await {
                    Ok(block_hash) => {
                        let propose_duration = propose_start.elapsed();
                        println!("✅ Block proposed successfully!");
                        println!("⏱️  Proposal time: {:.2?}", propose_duration);
                        println!("🧱 Block hash: {}", block_hash);
                        println!("🎉 Transfer completed successfully!");
                    }
                    Err(e) => {
                        println!("⚠️  Transfer deployed but block proposal failed!");
                        println!("Error: {}", e);
                        println!("💡 You can manually propose: cargo run -- propose");
                    }
                }
            } else {
                println!("💡 Transfer deployed. Run 'cargo run -- propose' to finalize.");
            }
        }
        Err(e) => {
            println!("❌ Transfer deployment failed!");
            println!("Error: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

fn derive_rev_address_from_private_key(private_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    use secp256k1::{PublicKey, Secp256k1, SecretKey};
    use sha3::{Digest, Keccak256};
    
    // Parse private key
    let secret_key = SecretKey::from_slice(&hex::decode(private_key)?)?;
    
    // Generate public key
    let secp = Secp256k1::new();
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_bytes = public_key.serialize_uncompressed();
    
    // Generate ETH address (last 20 bytes of keccak256 hash of public key without 0x04 prefix)
    let mut hasher = Keccak256::new();
    hasher.update(&public_key_bytes[1..]);
    let eth_address_bytes = &hasher.finalize()[12..];
    
    // Generate REV address from ETH address
    let rev_address = generate_rev_address_from_eth(eth_address_bytes);
    
    Ok(rev_address)
}

fn generate_rev_address_from_eth(eth_address: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    
    // REV address generation (simplified version)
    // In real implementation, this would use base58check encoding
    // For now, we'll create a basic implementation
    
    let mut hasher = Sha256::new();
    hasher.update(b"\x00\x00"); // Version bytes for REV
    hasher.update(eth_address);
    let hash1 = hasher.finalize();
    
    let mut hasher2 = Sha256::new();
    hasher2.update(&hash1);
    let hash2 = hasher2.finalize();
    
    // Create REV address with checksum
    let mut full_address = Vec::new();
    full_address.extend_from_slice(b"\x00\x00");
    full_address.extend_from_slice(eth_address);
    full_address.extend_from_slice(&hash2[0..4]);
    
    // Base58 encode (simplified - in practice would use proper base58check)
    format!("1111{}", hex::encode(&full_address[2..]))
}

fn validate_rev_address(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !address.starts_with("1111") {
        return Err("Invalid REV address format: must start with '1111'".into());
    }
    
    if address.len() < 40 {
        return Err("Invalid REV address format: too short".into());
    }
    
    Ok(())
}

fn generate_transfer_contract(from_address: &str, to_address: &str, amount_dust: u64) -> String {
    format!(
        r#"new retCh, 
    deployerId(`rho:rchain:deployerId`),
    stdout(`rho:io:stdout`),
    rl(`rho:registry:lookup`),
    revVaultCh
in {{
  rl!(`rho:rchain:revVault`, *revVaultCh) |
  for (@(_, RevVault) <- revVaultCh) {{
    @RevVault!("findOrCreate", "{}", *retCh) |
    for (@(true, fromVault) <- retCh) {{
      @fromVault!("transfer", "{}", {}, "{}", *retCh) |
      for (@(success, message) <- retCh) {{
        stdout!(("💸 Transfer result:", success, message, "from", "{}", "to", "{}", "amount", {}))
      }}
    }} |
    for (@(false, errorMsg) <- retCh) {{
      stdout!(("❌ Vault not found for:", "{}", "Error:", errorMsg))
    }}
  }}
}}"#,
        from_address, to_address, amount_dust, from_address, from_address, to_address, amount_dust, from_address
    )
}
