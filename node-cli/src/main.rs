use clap::Parser;
use node_cli::f1r3fly_api::F1r3flyApi;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Command-line interface for deploying Rholang code to F1r3fly nodes
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
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

    /// Use bigger phlo price
    #[arg(short, long, default_value_t = false)]
    bigger_phlo: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

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
