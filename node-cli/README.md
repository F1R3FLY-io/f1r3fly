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