use clap::{ArgAction, Parser, Subcommand};
use node_cli::f1r3fly_api::F1r3flyApi;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use secp256k1::{Secp256k1, SecretKey};
use hex;
use rand::rngs::OsRng;
use reqwest;
use serde_json;
use rholang::rust::interpreter::util::rev_address::RevAddress;
use crypto::rust::public_key::PublicKey;

/// Command-line interface for interacting with F1r3fly nodes
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy Rholang code to the F1r3fly network
    Deploy(DeployArgs),
    
    /// Propose a block to the F1r3fly network
    Propose(ProposeArgs),
    
    /// Deploy Rholang code and propose a block in one operation
    FullDeploy(DeployArgs),
    
    /// Check if a block is finalized
    IsFinalized(IsFinalizedArgs),
    
    /// Execute Rholang code without committing to the blockchain (exploratory deployment)
    ExploratoryDeploy(ExploratoryDeployArgs),

    /// Generate a public key from a private key
    GeneratePublicKey(GeneratePublicKeyArgs),
    
    /// Generate a new secp256k1 private/public key pair
    GenerateKeyPair(GenerateKeyPairArgs),

    /// Generate a REV address from a public key
    GenerateRevAddress(GenerateRevAddressArgs),

    /// Get node status and peer information
    Status(HttpArgs),

    /// Get recent blocks or specific block information
    Blocks(BlocksArgs),

    /// Get current validator bonds from PoS contract
    Bonds(HttpArgs),

    /// Get active validators from PoS contract
    ActiveValidators(HttpArgs),

    /// Check wallet balance for a specific address
    WalletBalance(WalletBalanceArgs),

    /// Check if a validator is bonded
    BondStatus(BondStatusArgs),

    /// Get node metrics
    Metrics(HttpArgs),

    /// Bond a new validator to the network (dynamic validator addition)
    BondValidator(BondValidatorArgs),

    /// Check network health across multiple nodes
    NetworkHealth(NetworkHealthArgs),

    /// Get the last finalized block
    LastFinalizedBlock(HttpArgs),
}

/// Arguments for deploy and full-deploy commands
#[derive(Parser)]
struct DeployArgs {
    /// Path to the Rholang file to deploy
    #[arg(short, long)]
    file: PathBuf,

    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    port: u16,

    /// Use bigger phlo limit
    #[arg(short, long, default_value_t = false)]
    bigger_phlo: bool,
}

/// Arguments for propose command
#[derive(Parser)]
struct ProposeArgs {
    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    port: u16,
}

/// Arguments for is-finalized command
#[derive(Parser)]
struct IsFinalizedArgs {
    /// Block hash to check
    #[arg(short, long)]
    block_hash: String,

    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    port: u16,
    
    /// Maximum number of retry attempts
    #[arg(short, long, default_value_t = 12)]
    max_attempts: u32,
    
    /// Delay between retries in seconds
    #[arg(short, long, default_value_t = 5)]
    retry_delay: u64,
}

/// Arguments for exploratory-deploy command
#[derive(Parser)]
struct ExploratoryDeployArgs {
    /// Path to the Rholang file to execute
    #[arg(short, long)]
    file: PathBuf,

    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    port: u16,
    
    /// Block hash to use as reference (optional)
    #[arg(short, long)]
    block_hash: Option<String>,
    
    /// Use pre-state hash instead of post-state hash
    #[arg(short, long, default_value_t = false)]
    use_pre_state: bool,
}

/// Arguments for generate-public-key command
#[derive(Parser)]
struct GeneratePublicKeyArgs {
    /// Private key in hex format
    #[arg(
        short,
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    private_key: String,

    /// Output public key in compressed format (shorter)
    #[arg(short, long, default_value_t = false)]
    compressed: bool,
}

/// Arguments for generate-key-pair command
#[derive(Parser)]
struct GenerateKeyPairArgs {
    /// Output public key in compressed format (shorter)
    #[arg(short, long, default_value_t = false)]
    compressed: bool,
    
    /// Save keys to files instead of displaying them
    #[arg(short, long, default_value_t = false)]
    save: bool,
    
    /// Output directory for saved keys (default: current directory)
    #[arg(short, long, default_value = ".")]
    output_dir: String,
}

/// Arguments for generate-rev-address command
#[derive(Parser)]
struct GenerateRevAddressArgs {
    /// Public key in hex format (uncompressed format preferred)
    #[arg(short, long, conflicts_with = "private_key")]
    public_key: Option<String>,
    
    /// Private key in hex format (will derive public key from this)
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1",
        conflicts_with = "public_key"
    )]
    private_key: Option<String>,
}

/// Arguments for HTTP-based commands (status, bonds, metrics)
#[derive(Parser)]
struct HttpArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// HTTP port number (not gRPC port)
    #[arg(short, long, default_value_t = 40403)]
    port: u16,
}

/// Arguments for blocks command
#[derive(Parser)]
struct BlocksArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// HTTP port number (not gRPC port)
    #[arg(short, long, default_value_t = 40403)]
    port: u16,
    
    /// Number of recent blocks to fetch (default: 5)
    #[arg(short, long, default_value_t = 5)]
    number: u32,
    
    /// Specific block hash to fetch (optional)
    #[arg(short, long)]
    block_hash: Option<String>,
}

/// Arguments for wallet-balance command
#[derive(Parser)]
struct WalletBalanceArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// gRPC port number 
    #[arg(short, long, default_value_t = 40402)]
    port: u16,
    
    /// Wallet address to check balance for
    #[arg(short = 'a', long)]
    address: String,
}

/// Arguments for bond-status command
#[derive(Parser)]
struct BondStatusArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// HTTP port number (same as other inspection commands)
    #[arg(short, long, default_value_t = 40403)]
    port: u16,
    
    /// Public key to check bond status for
    #[arg(short = 'k', long)]
    public_key: String,
}

/// Arguments for bond-validator command
#[derive(Parser)]
struct BondValidatorArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,

    /// gRPC port number for deploy
    #[arg(short, long, default_value_t = 40402)]
    port: u16,
    
    /// Stake amount for the validator
    #[arg(short, long, default_value_t = 50000000000000)]
    stake: u64,
    
    /// Private key for signing the deploy (hex format)
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    private_key: String,
    
    /// Also propose a block after bonding
    #[arg(long, default_value_t = false, action = ArgAction::Set, value_parser = clap::value_parser!(bool))]
    propose: bool,
}

/// Arguments for network-health command
#[derive(Parser)]
struct NetworkHealthArgs {
    /// Check standard F1r3fly shard ports (bootstrap, validator1, validator2, observer)
    #[arg(short, long, default_value_t = true)]
    standard_ports: bool,
    
    /// Additional custom ports to check (comma-separated, e.g. "60503,70503")
    #[arg(short, long)]
    custom_ports: Option<String>,
    
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Deploy(args) => deploy_command(args).await,
        Commands::Propose(args) => propose_command(args).await,
        Commands::FullDeploy(args) => full_deploy_command(args).await,
        Commands::IsFinalized(args) => is_finalized_command(args).await,
        Commands::ExploratoryDeploy(args) => exploratory_deploy_command(args).await,
        Commands::GeneratePublicKey(args) => generate_public_key_command(args),
        Commands::GenerateKeyPair(args) => generate_key_pair_command(args),
        Commands::GenerateRevAddress(args) => generate_rev_address_command(args),
        Commands::Status(args) => status_command(args).await,
        Commands::Blocks(args) => blocks_command(args).await,
        Commands::Bonds(args) => bonds_command(args).await,
        Commands::ActiveValidators(args) => active_validators_command(args).await,
        Commands::WalletBalance(args) => wallet_balance_command(args).await,
        Commands::BondStatus(args) => bond_status_command(args).await,
        Commands::Metrics(args) => metrics_command(args).await,
        Commands::BondValidator(args) => bond_validator_command(args).await,
        Commands::NetworkHealth(args) => network_health_command(args).await,
        Commands::LastFinalizedBlock(args) => last_finalized_block_command(args).await,
    }
}

async fn exploratory_deploy_command(args: &ExploratoryDeployArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Rholang code from file
    println!("📄 Reading Rholang from: {}", args.file.display());
    let rholang_code = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file: {}", e))?;
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
            args.use_pre_state
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

async fn deploy_command(args: &DeployArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Rholang code from file
    println!("📄 Reading Rholang from: {}", args.file.display());
    let rholang_code = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file: {}", e))?;
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

async fn propose_command(args: &ProposeArgs) -> Result<(), Box<dyn std::error::Error>> {
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

async fn full_deploy_command(args: &DeployArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Read the Rholang code from file
    println!("📄 Reading Rholang from: {}", args.file.display());
    let rholang_code = fs::read_to_string(&args.file)
        .map_err(|e| format!("Failed to read file: {}", e))?;
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

async fn is_finalized_command(args: &IsFinalizedArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the F1r3fly API client
    println!(
        "🔌 Connecting to F1r3fly node at {}:{}",
        args.host, args.port
    );
    let f1r3fly_api = F1r3flyApi::new(&args.private_key, &args.host, args.port);

    // Check if the block is finalized
    println!("🔍 Checking if block is finalized: {}", args.block_hash);
    println!("⏱️  Will retry every {} seconds, up to {} times", args.retry_delay, args.max_attempts);
    let start_time = Instant::now();

    match f1r3fly_api
        .is_finalized(&args.block_hash, Some(args.max_attempts), Some(args.retry_delay))
        .await
    {
        Ok(is_finalized) => {
            let duration = start_time.elapsed();
            if is_finalized {
                println!("✅ Block is finalized!");
            } else {
                println!("❌ Block is not finalized after {} attempts", args.max_attempts);
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

fn generate_public_key_command(args: &GeneratePublicKeyArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Decode private key from hex
    let private_key_bytes = hex::decode(&args.private_key)
        .map_err(|e| format!("Failed to decode private key: {}", e))?;
    
    // Create a secret key from the decoded bytes
    let secret_key = SecretKey::from_slice(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    
    // Initialize secp256k1 context
    let secp = Secp256k1::new();
    
    // Derive public key from private key
    let public_key = secret_key.public_key(&secp);
    
    // Serialize public key in the requested format
    let public_key_hex = if args.compressed {
        hex::encode(public_key.serialize())
    } else {
        hex::encode(public_key.serialize_uncompressed())
    };
    
    // Print the public key
    println!("Public key ({}): {}", 
        if args.compressed { "compressed" } else { "uncompressed" }, 
        public_key_hex);
    
    Ok(())
}

fn generate_key_pair_command(args: &GenerateKeyPairArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize secp256k1 context with random number generator
    let secp = Secp256k1::new();
    let mut rng = OsRng::default();
    
    // Generate a new random private key
    let secret_key = SecretKey::new(&mut rng);
    
    // Get the corresponding public key
    let public_key = secret_key.public_key(&secp);
    
    // Get the keys in hex format
    let private_key_hex = hex::encode(secret_key.secret_bytes());
    let public_key_hex = if args.compressed {
        hex::encode(public_key.serialize())
    } else {
        hex::encode(public_key.serialize_uncompressed())
    };
    
    if args.save {
        // Create output directory if it doesn't exist
        let output_dir = std::path::Path::new(&args.output_dir);
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }
        
        // Create filenames
        let private_key_file = output_dir.join("private_key.hex");
        let public_key_file = output_dir.join("public_key.hex");
        
        // Write keys to files
        std::fs::write(&private_key_file, &private_key_hex)?;
        std::fs::write(&public_key_file, &public_key_hex)?;
        
        println!("Key pair generated and saved to:");
        println!("  Private key: {}", private_key_file.display());
        println!("  Public key: {}", public_key_file.display());
    } else {
        // Print keys to stdout
        println!("Private key: {}", private_key_hex);
        println!("Public key ({}): {}", 
            if args.compressed { "compressed" } else { "uncompressed" }, 
            public_key_hex);
    }
    
    Ok(())
}

fn generate_rev_address_command(args: &GenerateRevAddressArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Get the public key either from direct input or by deriving from private key
    let public_key_bytes = if let Some(public_key_hex) = &args.public_key {
        // Use provided public key
        hex::decode(public_key_hex)
            .map_err(|e| format!("Failed to decode public key: {}", e))?
    } else if let Some(private_key_hex) = &args.private_key {
        // Derive public key from private key
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|e| format!("Failed to decode private key: {}", e))?;
        
        let secret_key = SecretKey::from_slice(&private_key_bytes)
            .map_err(|e| format!("Invalid private key: {}", e))?;
        
        let secp = Secp256k1::new();
        let public_key = secret_key.public_key(&secp);
        
        // Use uncompressed format for REV address generation
        public_key.serialize_uncompressed().to_vec()
    } else {
        return Err("Either public_key or private_key must be provided".into());
    };
    
    // Create PublicKey struct for RevAddress
    let public_key = PublicKey {
        bytes: public_key_bytes,
    };
    
    // Generate REV address
    match RevAddress::from_public_key(&public_key) {
        Some(rev_address) => {
            println!("🔑 Public key: {}", hex::encode(&public_key.bytes));
            println!("🏦 REV address: {}", rev_address.to_base58());
        }
        None => {
            return Err("Failed to generate REV address from public key".into());
        }
    }
    
    Ok(())
}

async fn status_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
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

async fn blocks_command(args: &BlocksArgs) -> Result<(), Box<dyn std::error::Error>> {
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
        println!("🔍 Getting {} recent blocks from {}:{}", args.number, args.host, args.port);
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

async fn bonds_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Getting validator bonds from {}:{}", args.host, args.port);
    
    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();
    
    let rholang_query = r#"new return, rl(`rho:registry:lookup`), poSCh in { rl!(`rho:rchain:pos`, *poSCh) | for(@(_, PoS) <- poSCh) { @PoS!("getBonds", *return) } }"#;
    
    let body = serde_json::json!({
        "term": rholang_query
    });
    
    let start_time = Instant::now();
    
    match client.post(&url)
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

async fn active_validators_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Getting active validators from {}:{}", args.host, args.port);
    
    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();
    
    let rholang_query = r#"new return, rl(`rho:registry:lookup`), poSCh in { rl!(`rho:rchain:pos`, *poSCh) | for(@(_, PoS) <- poSCh) { @PoS!("getActiveValidators", *return) } }"#;
    
    let body = serde_json::json!({
        "term": rholang_query
    });
    
    let start_time = Instant::now();
    
    match client.post(&url)
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
                println!("❌ Failed to get active validators: HTTP {}", response.status());
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

async fn wallet_balance_command(args: &WalletBalanceArgs) -> Result<(), Box<dyn std::error::Error>> {
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
    
    match client.post(&url)
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
                println!("❌ Failed to get wallet balance: HTTP {}", response.status());
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

async fn bond_status_command(args: &BondStatusArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Checking bond status for public key: {}", args.public_key);
    
    let url = format!("http://{}:{}/api/explore-deploy", args.host, args.port);
    let client = reqwest::Client::new();
    
    // Get all bonds first, then check if our public key is in there
    let rholang_query = r#"new return, rl(`rho:registry:lookup`), poSCh in { rl!(`rho:rchain:pos`, *poSCh) | for(@(_, PoS) <- poSCh) { @PoS!("getBonds", *return) } }"#;
    
    let body = serde_json::json!({
        "term": rholang_query
    });
    
    let start_time = Instant::now();
    
    match client.post(&url)
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

async fn metrics_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
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
                let lines: Vec<&str> = metrics_text.lines()
                    .filter(|line| {
                        line.contains("peers") || 
                        line.contains("blocks") || 
                        line.contains("consensus") ||
                        line.contains("casper") ||
                        line.contains("rspace")
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

async fn bond_validator_command(args: &BondValidatorArgs) -> Result<(), Box<dyn std::error::Error>> {
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
                        
                        println!("\n📋 Next Steps:");
                        println!("1. Wait for block finalization");
                        println!("2. Verify bond status with bonds command: cargo run -- bonds");
                        println!("3. Start the new validator node");
                        println!("4. Check network health: cargo run -- network-health");
                    }
                    Err(e) => {
                        println!("❌ Block proposal failed!");
                        println!("Error: {}", e);
                        return Err(e);
                    }
                }
            } else {
                println!("\n📋 Next Steps:");
                println!("1. Propose a block: cargo run -- propose");
                println!("2. Wait for block finalization");
                println!("3. Verify bond status with bonds command: cargo run -- bonds");
                println!("4. Start the new validator node");
                println!("5. Check network health: cargo run -- network-health");
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

async fn network_health_command(args: &NetworkHealthArgs) -> Result<(), Box<dyn std::error::Error>> {
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
                                    
                                    let peers = status_json.get("peers")
                                        .and_then(|p| p.as_u64())
                                        .unwrap_or(0);
                                    
                                    all_peers.push(peers);
                                    
                                    println!("✅ HEALTHY ({} peers)", peers);
                                }
                                Err(_) => println!("❌ Invalid JSON response")
                            }
                        }
                        Err(_) => println!("❌ Failed to read response")
                    }
                } else {
                    println!("❌ HTTP {}", response.status());
                }
            }
            Err(_) => println!("❌ Connection failed")
        }
    }
    
    println!("\n📈 Network Health Summary:");
    println!("✅ Healthy nodes: {}/{}", healthy_nodes, total_nodes);
    
    if healthy_nodes > 0 {
        let avg_peers = all_peers.iter().sum::<u64>() as f64 / all_peers.len() as f64;
        let min_peers = all_peers.iter().min().unwrap_or(&0);
        let max_peers = all_peers.iter().max().unwrap_or(&0);
        
        println!("👥 Peer connections: avg={:.1}, min={}, max={}", avg_peers, min_peers, max_peers);
        
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
    println!("- For a {}-node network, each node should have {} peers", total_nodes, total_nodes - 1);
    println!("- Check bonds: cargo run -- bonds");
    println!("- Check active validators: cargo run -- active-validators");
    
    Ok(())
}

async fn last_finalized_block_command(args: &HttpArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Getting last finalized block from {}:{}", args.host, args.port);
    
    let url = format!("http://{}:{}/api/last-finalized-block", args.host, args.port);
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
                println!("❌ Failed to get last finalized block: HTTP {}", response.status());
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
