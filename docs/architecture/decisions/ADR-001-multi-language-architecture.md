# ADR-001: Multi-Language Architecture

## Status
Adopted

## Context
RNode requires both high-level concurrent programming capabilities and low-level system programming performance. The decision was needed on whether to use a single programming language or adopt a multi-language approach.

## Decision
We will use Scala as the primary language for node infrastructure and business logic, with Rust for performance-critical system components.

## Rationale

### Scala Advantages
- **Actor Model**: Built-in support for concurrent programming via Akka
- **Functional Programming**: Immutable data structures and functional abstractions
- **JVM Ecosystem**: Access to mature libraries and tooling
- **Type Safety**: Strong static typing with type inference
- **gRPC Integration**: Excellent support for protocol buffers and gRPC

### Rust Advantages  
- **Memory Safety**: Zero-cost abstractions without garbage collection
- **Performance**: System-level performance for critical paths
- **Concurrency**: Safe concurrent programming with ownership model
- **Cross-compilation**: Easy deployment across different platforms
- **WASM Support**: Future potential for browser-based components

### Language Responsibilities

#### Scala Components
- **Node**: Main node process and coordination
- **Casper**: Consensus algorithm implementation  
- **Comm**: P2P networking (high-level protocols)
- **Integration Tests**: End-to-end testing framework

#### Rust Components
- **Rholang**: Language interpreter and parser
- **RSpace**: High-performance tuple space storage
- **Crypto**: Cryptographic operations and utilities
- **Block Storage**: Low-level storage operations
- **Node CLI**: Command-line interface tools

## Consequences

### Positive
- **Performance**: Critical components optimized for speed and memory usage
- **Developer Experience**: Use the right tool for each job
- **Ecosystem Access**: Leverage both JVM and Rust ecosystems
- **Future Flexibility**: Can migrate components as needed

### Negative
- **Complexity**: Additional build system and integration complexity
- **Learning Curve**: Team needs expertise in both languages
- **Debugging**: Cross-language debugging can be challenging
- **Dependencies**: Managing dependencies across language boundaries

## Implementation Strategy

### 1. Language Boundaries
- Clean interfaces between Scala and Rust components
- Use Protocol Buffers for data exchange
- FFI (Foreign Function Interface) for direct calls when needed

### 2. Build System
- SBT for Scala components
- Cargo for Rust components
- Unified build scripts for full system compilation

### 3. Testing Strategy
- Unit tests in respective languages
- Integration tests primarily in Scala
- Performance benchmarks in Rust

### 4. Documentation
- Language-specific documentation in respective directories
- Unified API documentation for public interfaces
- Architecture diagrams showing component interactions

## Monitoring

### Success Metrics
- **Performance**: Rust components show measurable performance improvements
- **Reliability**: No increase in bugs at language boundaries
- **Development Speed**: No significant slowdown in development velocity
- **Team Satisfaction**: Developers can effectively work in both languages

### Risk Mitigation
- **Training**: Provide Rust training for Scala developers
- **Code Review**: Cross-language code review process
- **Integration Testing**: Comprehensive tests at language boundaries
- **Documentation**: Clear guidelines for cross-language interfaces

## Related Decisions
- ADR-002: Consensus Algorithm Choice
- ADR-003: Storage Architecture
- ADR-004: Network Protocol Design

## References
- [Scala Documentation](https://docs.scala-lang.org/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Protocol Buffers](https://developers.google.com/protocol-buffers)
- [gRPC Documentation](https://grpc.io/docs/)