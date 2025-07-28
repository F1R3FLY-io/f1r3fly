# f1r3fly-rspace-plus-plus

High-performance F1r3fly Tuple Space implementation in Rust.

## Features

**Core Functionality:**
- Tuple space storage and retrieval
- Pattern matching and unification
- Persistent storage with LMDB backend
- Concurrent access with DashMap
- Serialization with bincode and protobuf

**Performance:**
- Optimized for high-throughput blockchain operations
- Memory-efficient data structures
- Async/await support with Tokio
- Parallel processing with Rayon

**Testing:**
- Property-based testing with proptest
- Comprehensive test suite
- Cross-compilation support

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
f1r3fly-rspace-plus-plus = "0.1.0"
```

## Building

```bash
cargo build --release
```

## Testing

```bash
# Run all tests
cargo test

# Run with release optimizations
cargo test --release

# Run specific test
cargo test --test <test_name>
```

## Cross-compilation

Uses Cross.toml for cross-platform builds.

## License

Licensed under the Apache License, Version 2.0.