# F1r3fly Node CLI

A command-line interface for interacting with F1r3fly nodes.

### Prerequisites

- [Environment set up](../README.md#installation).
- [Running Node](../README.md#running)

## Building

```bash
cargo build
```

## Usage

The CLI provides the following commands for interacting with F1r3fly nodes:

### Deploy

Deploy Rholang code to a F1r3fly node.

```bash
# Using default values (localhost:40402 with default private key)
cargo run -- deploy -f ../rholang/examples/stdout.rho

# With custom parameters
cargo run -- deploy -f ../rholang/examples/stdout.rho --private-key YOUR_PRIVATE_KEY -H node.example.com -p 40402

# With bigger phlo limit
cargo run -- deploy -f ../rholang/examples/stdout.rho -b
```

### Propose

Propose a block to the F1r3fly network.

```bash
# Using default values
cargo run -- propose

# With custom parameters
cargo run -- propose --private-key YOUR_PRIVATE_KEY -H node.example.com -p 40402
```

### Full Deploy

Deploy Rholang code and propose a block in one operation.

```bash
# Using default values
cargo run -- full-deploy -f ../rholang/examples/stdout.rho

# With custom parameters
cargo run -- full-deploy -f ../rholang/examples/stdout.rho --private-key YOUR_PRIVATE_KEY -H node.example.com -p 40402

# With bigger phlo limit
cargo run -- full-deploy -f ../rholang/examples/stdout.rho -b
```

### Is Finalized

Check if a block is finalized, with automatic retries.

```bash
# Using default values (retry every 5 seconds, up to 12 times)
cargo run -- is-finalized -b BLOCK_HASH

# With custom parameters
cargo run -- is-finalized -b BLOCK_HASH --private-key YOUR_PRIVATE_KEY -H node.example.com -p 40402

# With custom retry settings
cargo run -- is-finalized -b BLOCK_HASH -m 20 -r 3  # Retry every 3 seconds, up to 20 times
```

### Exploratory Deploy

Execute Rholang code without committing it to the blockchain. This is useful for read-only operations or when working with nodes in read-only mode.

```bash
# Using default values (latest state)
cargo run -- exploratory-deploy -f ../rholang/examples/stdout.rho

# With custom parameters
cargo run -- exploratory-deploy -f ../rholang/examples/stdout.rho --private-key YOUR_PRIVATE_KEY -H node.example.com -p 40402

# Execute at a specific block
cargo run -- exploratory-deploy -f ../rholang/examples/stdout.rho --block-hash BLOCK_HASH

# Using pre-state hash instead of post-state hash
cargo run -- exploratory-deploy -f ../rholang/examples/stdout.rho --block-hash BLOCK_HASH --use-pre-state
```

### Generate Public Key

Generate a public key from a given private key.

```bash
# Using default private key
cargo run -- generate-public-key

# Provide your own private key
cargo run -- generate-public-key --private-key YOUR_PRIVATE_KEY

# Generate compressed format public key
cargo run -- generate-public-key --private-key YOUR_PRIVATE_KEY --compressed
```

### Generate Key Pair

Generate a new secp256k1 private/public key pair.

```bash
# Generate a new key pair and display on screen
cargo run -- generate-key-pair

# Generate a key pair with compressed public key
cargo run -- generate-key-pair --compressed

# Generate a key pair and save to files
cargo run -- generate-key-pair --save

# Save to a specific directory
cargo run -- generate-key-pair --save --output-dir /path/to/keys
```

### Generate REV Address

Generate a REV address from a public key. You can either provide a public key directly or use a private key (from which the public key will be derived).

```bash
# Using default private key
cargo run -- generate-rev-address

# Provide your own private key
cargo run -- generate-rev-address --private-key YOUR_PRIVATE_KEY

# Provide a public key directly
cargo run -- generate-rev-address --public-key YOUR_PUBLIC_KEY
```

Example output:
```
🔑 Public key: 04a936f4e0cda4688ec61fa17cf3cbaed6a450ac8e633490596587ce22b78fe6621861d4fa442f7c9f8070acb846f40d8844dca94fda398722d6a4664041a7b39b
🏦 REV address: 111129tt6TzcaUQ8QkEUSark5MzaN814bBq1atM7cDr2SVAdriuNS4
```

## Node Inspection Commands

The CLI provides several commands for inspecting and monitoring F1r3fly nodes using HTTP endpoints:

### Status

Get node status and peer information.

```bash
# Get status from default node (localhost:40403)
cargo run -- status

# Get status from custom node
cargo run -- status -H node.example.com -p 40403
```

### Blocks

Get recent blocks or specific block information.

```bash
# Get 5 recent blocks (default)
cargo run -- blocks

# Get 10 recent blocks
cargo run -- blocks -n 10

# Get specific block by hash
cargo run -- blocks --block-hash BLOCK_HASH_HERE

# Get blocks from custom node
cargo run -- blocks -H node.example.com -p 40403 -n 3
```

### Bonds

Get current validator bonds from the PoS contract.

```bash
# Get validator bonds (uses HTTP port for explore-deploy endpoint)
cargo run -- bonds

# Get bonds from custom node
cargo run -- bonds -H node.example.com -p 40403
```

### Active Validators

Get active validators from the PoS contract.

```bash
# Get active validators (uses HTTP port for explore-deploy endpoint)
cargo run -- active-validators

# Get active validators from custom node
cargo run -- active-validators -H node.example.com -p 40403
```

### Wallet Balance

Check wallet balance for a specific address.

```bash
# Check wallet balance for an address
cargo run -- wallet-balance --address "1111AtahZeefej4tvVR6ti9TJtv8yxLebT31SCEVDCKMNikBk5r3g"

# Check balance from custom node (uses gRPC port)
cargo run -- wallet-balance -a "1111AtahZeefej4tvVR6ti9TJtv8yxLebT31SCEVDCKMNikBk5r3g" -H node.example.com -p 40402
```

### Bond Status

Check if a validator is bonded by public key.

```bash
# Check bond status for a public key
cargo run -- bond-status --public-key "04ffc016579a68050d655d55df4e09f04605164543e257c8e6df10361e6068a5336588e9b355ea859c5ab4285a5ef0efdf62bc28b80320ce99e26bb1607b3ad93d"

# Check from custom node (uses HTTP port like other inspection commands)
cargo run -- bond-status -k "PUBLIC_KEY_HERE" -H node.example.com -p 40403
```

### Metrics

Get node metrics for monitoring.

```bash
# Get node metrics (filtered to show key metrics)
cargo run -- metrics

# Get metrics from custom node
cargo run -- metrics -H node.example.com -p 40403
```

### Last Finalized Block

Get the last finalized block from the node.

```bash
# Get last finalized block from default node (localhost:40403)
cargo run -- last-finalized-block

# Get last finalized block from custom node
cargo run -- last-finalized-block -H node.example.com -p 40403
```

## Dynamic Validator Addition Commands

The CLI provides commands for dynamically adding validators to a running F1r3fly network, based on the procedures outlined in the `add-validator-dynamically.md` guide.

### Bond Validator

Deploy a bonding transaction to add a new validator to the network.

```bash
# Bond a new validator with default stake (50 trillion REV)
cargo run -- bond-validator

# Bond with custom stake amount
cargo run -- bond-validator --stake 25000000000000

# Bond and also propose a block (auto-propose)
cargo run -- bond-validator --propose true

# Explicitly disable auto-propose (same as default)
cargo run -- bond-validator --propose false

# Bond using custom node and private key
cargo run -- bond-validator -H node.example.com -p 40402 --private-key YOUR_PRIVATE_KEY
```

### Network Health

Check the health and connectivity of multiple nodes in your F1r3fly shard.

```bash
# Check standard F1r3fly shard ports (bootstrap, validator1, validator2, observer)
cargo run -- network-health

# Check network health with custom additional ports (e.g., after adding validator3)
cargo run -- network-health --custom-ports "60503"

# Check only custom ports (disable standard ports)
cargo run -- network-health --standard-ports false --custom-ports "60503,70503"

# Check network health on different host
cargo run -- network-health -H node.example.com --custom-ports "60503"
```

## Using the Makefile

For convenience, a Makefile is provided to simplify common operations. The Makefile uses the example Rholang file located at `../rholang/examples/stdout.rho`.

```bash
# Build the CLI
make build

# Deploy the example Rholang file
make deploy

# Deploy with bigger phlo limit
make deploy-big

# Propose a block
make propose

# Deploy and propose in one operation
make full-deploy

# Full deploy with bigger phlo limit
make full-deploy-big

# Check if a block is finalized
make check-finalized BLOCK_HASH=your_block_hash_here

# Execute Rholang without committing to the blockchain (exploratory deploy)
make exploratory-deploy

# Execute Rholang at a specific block
make exploratory-deploy-at-block BLOCK_HASH=your_block_hash_here

# Show help information
make help

# Node inspection commands
make status
make blocks
make bonds
make active-validators
make wallet-balance ADDRESS=your_wallet_address_here
make bond-status PUBLIC_KEY=your_public_key_here
make metrics
make last-finalized-block

# Dynamic validator addition commands
make bond-validator
make bond-validator STAKE=25000000000000
make network-health
make network-health-custom CUSTOM_PORTS=60503,70503
```

## Command Line Options

### Deploy and Full-Deploy Commands

- `-f, --file <FILE>`: Path to the Rholang file to deploy (required)
- `--private-key <PRIVATE_KEY>`: Private key in hex format
- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number (default: 40402)
- `-b, --bigger-phlo`: Use bigger phlo limit

### Propose Command

- `--private-key <PRIVATE_KEY>`: Private key in hex format
- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number (default: 40402)

### Is-Finalized Command

- `-b, --block-hash <BLOCK_HASH>`: Block hash to check (required)
- `--private-key <PRIVATE_KEY>`: Private key in hex format
- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number (default: 40402)
- `-m, --max-attempts <MAX_ATTEMPTS>`: Maximum number of retry attempts (default: 12)
- `-r, --retry-delay <RETRY_DELAY>`: Delay between retries in seconds (default: 5)

### Exploratory-Deploy Command

- `-f, --file <FILE>`: Path to the Rholang file to execute (required)
- `--private-key <PRIVATE_KEY>`: Private key in hex format
- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number (default: 40402)
- `-b, --block-hash <BLOCK_HASH>`: Optional block hash to use as reference
- `-u, --use-pre-state`: Use pre-state hash instead of post-state hash

### Generate-Public-Key Command

- `--private-key <PRIVATE_KEY>`: Private key in hex format
- `-c, --compressed`: Output public key in compressed format (shorter)

### Generate-Key-Pair Command

- `-c, --compressed`: Output public key in compressed format (shorter)
- `-s, --save`: Save keys to files instead of displaying them
- `-o, --output-dir <DIR>`: Output directory for saved keys (default: current directory)

### Status Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)

### Blocks Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)
- `-n, --number <NUMBER>`: Number of recent blocks to fetch (default: 5)
- `-b, --block-hash <BLOCK_HASH>`: Specific block hash to fetch (optional)

### Bonds Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)

### Active-Validators Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)

### Wallet-Balance Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number (default: 40402)
- `-a, --address <ADDRESS>`: Wallet address to check balance for (required)

### Bond-Status Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)
- `-k, --public-key <PUBLIC_KEY>`: Public key to check bond status for (required)

### Metrics Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)

### Last-Finalized-Block Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: HTTP port number (default: 40403)

### Bond-Validator Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number for deploy (default: 40402)
- `-s, --stake <STAKE>`: Stake amount for the validator (default: 50000000000000)
- `--private-key <PRIVATE_KEY>`: Private key for signing the deploy (hex format)
- `--propose <PROPOSE>`: Also propose a block after bonding (default: false)

### Network-Health Command

- `-H, --host <HOST>`: Host address (default: "localhost")
- `-s, --standard-ports <STANDARD_PORTS>`: Check standard F1r3fly shard ports (default: true)
- `-c, --custom-ports <CUSTOM_PORTS>`: Additional custom ports to check (comma-separated)