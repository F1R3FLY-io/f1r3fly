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