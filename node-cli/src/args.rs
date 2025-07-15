use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

/// Command-line interface for interacting with F1r3fly nodes
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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
pub struct DeployArgs {
    /// Path to the Rholang file to deploy
    #[arg(short, long)]
    pub file: PathBuf,

    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    pub private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    pub port: u16,

    /// Use bigger phlo limit
    #[arg(short, long, default_value_t = false)]
    pub bigger_phlo: bool,
}

/// Arguments for propose command
#[derive(Parser)]
pub struct ProposeArgs {
    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    pub private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    pub port: u16,
}

/// Arguments for is-finalized command
#[derive(Parser)]
pub struct IsFinalizedArgs {
    /// Block hash to check
    #[arg(short, long)]
    pub block_hash: String,

    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    pub private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    pub port: u16,

    /// Maximum number of retry attempts
    #[arg(short, long, default_value_t = 12)]
    pub max_attempts: u32,

    /// Delay between retries in seconds
    #[arg(short, long, default_value_t = 5)]
    pub retry_delay: u64,
}

/// Arguments for exploratory-deploy command
#[derive(Parser)]
pub struct ExploratoryDeployArgs {
    /// Path to the Rholang file to execute
    #[arg(short, long)]
    pub file: PathBuf,

    /// Private key in hex format
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    pub private_key: String,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// gRPC port number
    #[arg(short, long, default_value_t = 40402)]
    pub port: u16,

    /// Block hash to use as reference (optional)
    #[arg(short, long)]
    pub block_hash: Option<String>,

    /// Use pre-state hash instead of post-state hash
    #[arg(short, long, default_value_t = false)]
    pub use_pre_state: bool,
}

/// Arguments for generate-public-key command
#[derive(Parser)]
pub struct GeneratePublicKeyArgs {
    /// Private key in hex format
    #[arg(
        short,
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    pub private_key: String,

    /// Output public key in compressed format (shorter)
    #[arg(short, long, default_value_t = false)]
    pub compressed: bool,
}

/// Arguments for generate-key-pair command
#[derive(Parser)]
pub struct GenerateKeyPairArgs {
    /// Output public key in compressed format (shorter)
    #[arg(short, long, default_value_t = false)]
    pub compressed: bool,

    /// Save keys to files instead of displaying them
    #[arg(short, long, default_value_t = false)]
    pub save: bool,

    /// Output directory for saved keys (default: current directory)
    #[arg(short, long, default_value = ".")]
    pub output_dir: String,
}

/// Arguments for generate-rev-address command
#[derive(Parser)]
pub struct GenerateRevAddressArgs {
    /// Public key in hex format (uncompressed format preferred)
    #[arg(short, long, conflicts_with = "private_key")]
    pub public_key: Option<String>,

    /// Private key in hex format (will derive public key from this)
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1",
        conflicts_with = "public_key"
    )]
    pub private_key: Option<String>,
}

/// Arguments for HTTP-based commands (status, bonds, metrics)
#[derive(Parser)]
pub struct HttpArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// HTTP port number (not gRPC port)
    #[arg(short, long, default_value_t = 40453)]
    pub port: u16,
}

/// Arguments for blocks command
#[derive(Parser)]
pub struct BlocksArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// HTTP port number (not gRPC port)
    #[arg(short, long, default_value_t = 40453)]
    pub port: u16,

    /// Number of recent blocks to fetch (default: 5)
    #[arg(short, long, default_value_t = 5)]
    pub number: u32,

    /// Specific block hash to fetch (optional)
    #[arg(short, long)]
    pub block_hash: Option<String>,
}

/// Arguments for wallet-balance command
#[derive(Parser)]
pub struct WalletBalanceArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// gRPC port number (requires read-only node)
    #[arg(short, long, default_value_t = 40452)]
    pub port: u16,

    /// Wallet address to check balance for
    #[arg(short = 'a', long)]
    pub address: String,
}

/// Arguments for bond-status command
#[derive(Parser)]
pub struct BondStatusArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// HTTP port number (same as other inspection commands)
    #[arg(short, long, default_value_t = 40403)]
    pub port: u16,

    /// Public key to check bond status for
    #[arg(short = 'k', long)]
    pub public_key: String,
}

/// Arguments for bond-validator command
#[derive(Parser)]
pub struct BondValidatorArgs {
    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,

    /// gRPC port number for deploy
    #[arg(short, long, default_value_t = 40402)]
    pub port: u16,

    /// Stake amount for the validator
    #[arg(short, long, default_value_t = 50000000000000)]
    pub stake: u64,

    /// Private key for signing the deploy (hex format)
    #[arg(
        long,
        default_value = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
    )]
    pub private_key: String,

    /// Also propose a block after bonding
    #[arg(long, default_value_t = false, action = ArgAction::Set, value_parser = clap::value_parser!(bool))]
    pub propose: bool,
}

/// Arguments for network-health command
#[derive(Parser)]
pub struct NetworkHealthArgs {
    /// Check standard F1r3fly shard ports (bootstrap, validator1, validator2, observer)
    #[arg(short, long, default_value_t = true)]
    pub standard_ports: bool,

    /// Additional custom ports to check (comma-separated, e.g. "60503,70503")
    #[arg(short, long)]
    pub custom_ports: Option<String>,

    /// Host address
    #[arg(short = 'H', long, default_value = "localhost")]
    pub host: String,
}
