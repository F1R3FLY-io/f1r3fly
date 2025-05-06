use clap::{Parser, Subcommand};
use node_cli::f1r3fly_api::F1r3flyApi;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Deploy(args) => deploy_command(args).await,
        Commands::Propose(args) => propose_command(args).await,
        Commands::FullDeploy(args) => full_deploy_command(args).await,
        Commands::IsFinalized(args) => is_finalized_command(args).await,
        Commands::ExploratoryDeploy(args) => exploratory_deploy_command(args).await,
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
