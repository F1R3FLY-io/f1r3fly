use crate::args::*;
use reqwest;
use serde_json;
use std::time::Instant;

pub async fn status_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Getting node status from {}:{}", args.host, args.port);

    let url = format!("http://{}:{}/status", args.host, args.port);
    let client = reqwest::Client::new();

    let start_time = Instant::now();

    match client.get(&url).send().await {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let status_text = response.text().await?;
                let status_json: serde_json::Value = serde_json::from_str(&status_text)?;

                println!("✅ Node status retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);
                println!("📊 Node Status:");
                println!("{}", serde_json::to_string_pretty(&status_json)?);
            } else {
                println!("❌ Failed to get node status: HTTP {}", response.status());
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn blocks_command(args: &BlocksArgs) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let client = reqwest::Client::new();

    if let Some(block_hash) = &args.block_hash {
        println!("🔍 Getting specific block: {}", block_hash);
        let url = format!("http://{}:{}/block/{}", args.host, args.port, block_hash);

        match client.get(&url).send().await {
            Ok(response) => {
                let duration = start_time.elapsed();
                if response.status().is_success() {
                    let block_text = response.text().await?;
                    let block_json: serde_json::Value = serde_json::from_str(&block_text)?;

                    println!("✅ Block retrieved successfully!");
                    println!("⏱️  Time taken: {:.2?}", duration);
                    println!("🧱 Block Details:");
                    println!("{}", serde_json::to_string_pretty(&block_json)?);
                } else {
                    println!("❌ Failed to get block: HTTP {}", response.status());
                    println!("Error: {}", response.text().await?);
                }
            }
            Err(e) => {
                println!("❌ Connection failed!");
                println!("Error: {}", e);
                return Err(e.into());
            }
        }
    } else {
        println!(
            "🔍 Getting {} recent blocks from {}:{}",
            args.number, args.host, args.port
        );
        let url = format!("http://{}:{}/blocks/{}", args.host, args.port, args.number);

        match client.get(&url).send().await {
            Ok(response) => {
                let duration = start_time.elapsed();
                if response.status().is_success() {
                    let blocks_text = response.text().await?;
                    let blocks_json: serde_json::Value = serde_json::from_str(&blocks_text)?;

                    println!("✅ Blocks retrieved successfully!");
                    println!("⏱️  Time taken: {:.2?}", duration);
                    println!("🧱 Recent Blocks:");
                    println!("{}", serde_json::to_string_pretty(&blocks_json)?);
                } else {
                    println!("❌ Failed to get blocks: HTTP {}", response.status());
                    println!("Error: {}", response.text().await?);
                }
            }
            Err(e) => {
                println!("❌ Connection failed!");
                println!("Error: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}

pub async fn bonds_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🔍 Getting validator bonds from {}:{}",
        args.host, args.port
    );

    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();

    let rholang_query = r#"new return, rl(`rho:registry:lookup`), poSCh in { rl!(`rho:rchain:pos`, *poSCh) | for(@(_, PoS) <- poSCh) { @PoS!("getBonds", *return) } }"#;

    let body = serde_json::json!({
        "term": rholang_query
    });

    let start_time = Instant::now();

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let bonds_text = response.text().await?;
                let bonds_json: serde_json::Value = serde_json::from_str(&bonds_text)?;

                println!("✅ Validator bonds retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);
                println!("🔗 Current Validator Bonds:");
                println!("{}", serde_json::to_string_pretty(&bonds_json)?);
            } else {
                println!("❌ Failed to get bonds: HTTP {}", response.status());
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn active_validators_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🔍 Getting active validators from {}:{}",
        args.host, args.port
    );

    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();

    let rholang_query = r#"new return, rl(`rho:registry:lookup`), poSCh in { rl!(`rho:rchain:pos`, *poSCh) | for(@(_, PoS) <- poSCh) { @PoS!("getActiveValidators", *return) } }"#;

    let body = serde_json::json!({
        "term": rholang_query
    });

    let start_time = Instant::now();

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let validators_text = response.text().await?;
                let validators_json: serde_json::Value = serde_json::from_str(&validators_text)?;

                println!("✅ Active validators retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);
                println!("👥 Active Validators:");
                println!("{}", serde_json::to_string_pretty(&validators_json)?);
            } else {
                println!(
                    "❌ Failed to get active validators: HTTP {}",
                    response.status()
                );
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn wallet_balance_command(
    args: &WalletBalanceArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking wallet balance for address: {}", args.address);

    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();

    let rholang_query = format!(
        r#"new return, rl(`rho:registry:lookup`), revVaultCh in {{ rl!(`rho:rchain:revVault`, *revVaultCh) | for(@(_, RevVault) <- revVaultCh) {{ @RevVault!("findOrCreate", "{}", *return) }} }}"#,
        args.address
    );

    let body = serde_json::json!({
        "term": rholang_query
    });

    let start_time = Instant::now();

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let balance_text = response.text().await?;
                let balance_json: serde_json::Value = serde_json::from_str(&balance_text)?;

                println!("✅ Wallet balance retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);
                println!("💰 Wallet Balance for {}:", args.address);
                println!("{}", serde_json::to_string_pretty(&balance_json)?);
            } else {
                println!(
                    "❌ Failed to get wallet balance: HTTP {}",
                    response.status()
                );
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn bond_status_command(args: &BondStatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🔍 Checking bond status for public key: {}",
        args.public_key
    );

    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();

    // Get all bonds first, then check if our public key is in there
    let rholang_query = r#"new return, rl(`rho:registry:lookup`), poSCh in { rl!(`rho:rchain:pos`, *poSCh) | for(@(_, PoS) <- poSCh) { @PoS!("getBonds", *return) } }"#;

    let body = serde_json::json!({
        "term": rholang_query
    });

    let start_time = Instant::now();

    match client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
    {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let bonds_text = response.text().await?;
                let bonds_json: serde_json::Value = serde_json::from_str(&bonds_text)?;

                println!("✅ Bond information retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);

                // Check if the public key exists in the bonds
                let is_bonded = check_if_key_is_bonded(&bonds_json, &args.public_key);

                if is_bonded {
                    println!("🔗 ✅ Validator is BONDED");
                    println!("📍 Public key: {}", args.public_key);
                } else {
                    println!("🔗 ❌ Validator is NOT BONDED");
                    println!("📍 Public key: {}", args.public_key);
                }

                println!("\n📊 Full bonds data:");
                println!("{}", serde_json::to_string_pretty(&bonds_json)?);
            } else {
                println!("❌ Failed to get bond status: HTTP {}", response.status());
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn check_if_key_is_bonded(bonds_json: &serde_json::Value, target_public_key: &str) -> bool {
    // Navigate through the JSON structure to find bonds
    // The structure is: block.bonds[].validator
    if let Some(block) = bonds_json.get("block") {
        if let Some(bonds_array) = block.get("bonds") {
            if let Some(bonds) = bonds_array.as_array() {
                // Check each bond entry
                for bond in bonds {
                    if let Some(validator) = bond.get("validator") {
                        if let Some(validator_key) = validator.as_str() {
                            if validator_key == target_public_key {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub async fn metrics_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Getting node metrics from {}:{}", args.host, args.port);

    let url = format!("http://{}:{}/metrics", args.host, args.port);
    let client = reqwest::Client::new();

    let start_time = Instant::now();

    match client.get(&url).send().await {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let metrics_text = response.text().await?;

                println!("✅ Node metrics retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);
                println!("📊 Node Metrics:");

                // Filter and display key metrics
                let lines: Vec<&str> = metrics_text
                    .lines()
                    .filter(|line| {
                        line.contains("peers")
                            || line.contains("blocks")
                            || line.contains("consensus")
                            || line.contains("casper")
                            || line.contains("rspace")
                    })
                    .collect();

                if lines.is_empty() {
                    println!("📊 All Metrics:");
                    println!("{}", metrics_text);
                } else {
                    println!("📊 Key Metrics (peers, blocks, consensus):");
                    for line in lines {
                        println!("{}", line);
                    }
                    println!("\n💡 Use --verbose flag (if implemented) to see all metrics");
                }
            } else {
                println!("❌ Failed to get metrics: HTTP {}", response.status());
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

pub async fn network_health_command(
    args: &NetworkHealthArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Checking F1r3fly network health");

    let mut ports_to_check = Vec::new();

    if args.standard_ports {
        // Standard F1r3fly shard ports from the documentation
        ports_to_check.extend_from_slice(&[
            (40403, "Bootstrap"),
            (50403, "Validator1"),
            (60403, "Validator2"),
            (7043, "Observer"),
        ]);
    }

    // Add custom ports if specified
    if let Some(custom_ports_str) = &args.custom_ports {
        for port_str in custom_ports_str.split(',') {
            if let Ok(port) = port_str.trim().parse::<u16>() {
                ports_to_check.push((port, "Custom"));
            }
        }
    }

    if ports_to_check.is_empty() {
        println!("❌ No ports specified to check");
        return Ok(());
    }

    let client = reqwest::Client::new();
    let mut healthy_nodes = 0;
    let mut total_nodes = 0;
    let mut all_peers = Vec::new();

    println!("🔍 Checking {} nodes...\n", ports_to_check.len());

    for (port, node_type) in ports_to_check {
        total_nodes += 1;
        let url = format!("http://{}:{}/status", args.host, port);

        print!("📊 {} ({}:{}): ", node_type, args.host, port);

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(status_text) => {
                            match serde_json::from_str::<serde_json::Value>(&status_text) {
                                Ok(status_json) => {
                                    healthy_nodes += 1;

                                    let peers = status_json
                                        .get("peers")
                                        .and_then(|p| p.as_u64())
                                        .unwrap_or(0);

                                    all_peers.push(peers);

                                    println!("✅ HEALTHY ({} peers)", peers);
                                }
                                Err(_) => println!("❌ Invalid JSON response"),
                            }
                        }
                        Err(_) => println!("❌ Failed to read response"),
                    }
                } else {
                    println!("❌ HTTP {}", response.status());
                }
            }
            Err(_) => println!("❌ Connection failed"),
        }
    }

    println!("\n📈 Network Health Summary:");
    println!("✅ Healthy nodes: {}/{}", healthy_nodes, total_nodes);

    if healthy_nodes > 0 {
        let avg_peers = all_peers.iter().sum::<u64>() as f64 / all_peers.len() as f64;
        let min_peers = all_peers.iter().min().unwrap_or(&0);
        let max_peers = all_peers.iter().max().unwrap_or(&0);

        println!(
            "👥 Peer connections: avg={:.1}, min={}, max={}",
            avg_peers, min_peers, max_peers
        );

        if healthy_nodes == total_nodes && *min_peers >= (total_nodes as u64 - 1) {
            println!("🎉 Network is FULLY CONNECTED and HEALTHY!");
        } else if healthy_nodes == total_nodes {
            println!("⚠️  All nodes healthy but some peer connections may be missing");
        } else {
            println!("⚠️  Some nodes are unhealthy - check individual node logs");
        }
    } else {
        println!("❌ No healthy nodes found - check if network is running");
    }

    println!("\n💡 Tips:");
    println!(
        "- For a {}-node network, each node should have {} peers",
        total_nodes,
        total_nodes - 1
    );
    println!("- Check bonds: cargo run -- bonds");
    println!("- Check active validators: cargo run -- active-validators");

    Ok(())
}

pub async fn last_finalized_block_command(
    args: &HttpArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🔍 Getting last finalized block from {}:{}",
        args.host, args.port
    );

    let url = format!(
        "http://{}:{}/api/last-finalized-block",
        args.host, args.port
    );
    let client = reqwest::Client::new();

    let start_time = Instant::now();

    match client.get(&url).send().await {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                let block_text = response.text().await?;
                let block_json: serde_json::Value = serde_json::from_str(&block_text)?;

                println!("✅ Last finalized block retrieved successfully!");
                println!("⏱️  Time taken: {:.2?}", duration);
                println!("🧱 Last Finalized Block:");
                println!("{}", serde_json::to_string_pretty(&block_json)?);
            } else {
                println!(
                    "❌ Failed to get last finalized block: HTTP {}",
                    response.status()
                );
                println!("Error: {}", response.text().await?);
            }
        }
        Err(e) => {
            println!("❌ Connection failed!");
            println!("Error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
