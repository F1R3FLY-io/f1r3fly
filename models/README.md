# f1r3fly-models

Common data model types for the F1r3fly blockchain, including protobuf definitions and Rust implementations.

## Features

**Data Models:**
- Protobuf-generated types for blockchain operations
- Par (parallel process) representations
- Expression and pattern matching types
- Connectives and logical operations
- Unforgeable names and cryptographic primitives

**Protocol Buffers:**
- Casper consensus protocol messages
- RhoLang type definitions  
- RSpace storage types
- Service APIs and communication protocols

**Utilities:**
- Sorted collections (ParMap, ParSet)
- Hash and equality implementations
- Type mappers and converters

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
f1r3fly-models = "0.1.0"
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## License

Licensed under the Apache License, Version 2.0.
