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

The CLI allows you to deploy Rholang code to a F1r3fly node.

```bash
# Using default values (localhost:40402 with default private key)
cargo run -- -f ../rholang/examples/stdout.rho

# With custom parameters
cargo run -- -f ../rholang/examples/stdout.rho --private-key YOUR_PRIVATE_KEY -H node.example.com -p 40402

# With bigger phlo price
cargo run -- -f ../rholang/examples/stdout.rho -b
```

### Command Line Options

- `-f, --file <FILE>`: Path to the Rholang file to deploy (required)
- `--private-key <PRIVATE_KEY>`: Private key in hex format
- `-H, --host <HOST>`: Host address (default: "localhost")
- `-p, --port <PORT>`: gRPC port number (default: 40402)
- `-b, --bigger-phlo`: Use bigger phlo price