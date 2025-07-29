# f1r3fly-rholang

F1r3fly Rholang programming language - a concurrent, message-passing smart contract language built on the ρ-calculus.

## Overview

Rholang is a concurrent programming language with a focus on message-passing, formally modeled by the ρ-calculus (a reflective, higher-order extension of the π-calculus). It's designed for implementing protocols and smart contracts on the F1r3fly blockchain.

This is a Rust implementation of the Rholang language, providing high-performance execution and seamless integration with other F1r3fly components.

## Features

**Language Features:**
- Concurrent execution with message-passing
- Pattern matching and unification  
- Smart contract development
- Protocol implementation
- Tree-sitter parsing

**Implementation:**
- High-performance Rust interpreter
- Integration with F1r3fly ecosystem
- Extensive example library
- Cross-compilation support

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
f1r3fly-rholang = "0.1.0"
```

## Building

```bash
# Release build
cargo build --release

# Development build  
cargo build --profile dev
```

## Testing

```bash
# Run all tests
cargo test

# Run tests in release mode
cargo test --release

# Run specific test
cargo test --test <test_name>
```

## Examples

The `examples/` directory contains numerous Rholang code examples demonstrating language features and smart contract patterns.

## License

Licensed under the Apache License, Version 2.0.