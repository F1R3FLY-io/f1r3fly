# RNode Architecture Documentation

This directory contains architectural designs and decisions for the RNode platform.

## Structure

- **[decisions/](decisions/)** - Architecture Decision Records (ADRs)
- **[diagrams/](diagrams/)** - System component diagrams and data flows
- **[patterns/](patterns/)** - Established patterns and conventions

## Architecture Overview

RNode is a distributed blockchain platform implementing a concurrent smart contract execution environment. The architecture is designed around several key principles:

### Core Architectural Principles

1. **Concurrency by Design**: Built on the ρ-calculus for true concurrent execution
2. **Multi-Consensus Architecture**: Four consensus mechanisms for different scenarios
3. **Multi-language Implementation**: Scala for node infrastructure, Rust for performance-critical components
4. **Modular Design**: Loosely coupled components with clear interfaces
5. **Resource Accounting**: Precise cost accounting for all operations

## System Components

### 1. Core Node Infrastructure
- **Node**: Main node process and lifecycle management
- **Casper**: Multi-consensus mechanisms (Cordial Miners, Casper CBC, RGB PSSM, Casanova)
- **Comm**: P2P networking and communication protocols
- **Block Storage**: Persistent storage for blockchain data

### 2. Smart Contract Platform
- **Rholang**: Concurrent programming language implementation
- **RSpace**: Tuple space storage for contract state
- **Interpreter**: Rholang contract execution engine
- **Cost Accounting**: Resource usage tracking and limits

### 3. Supporting Infrastructure
- **Crypto**: Cryptographic primitives and utilities
- **Models**: Shared data structures and serialization
- **Node CLI**: Command-line interface for node interaction
- **Bitcoin Anchor**: Layer 1 anchoring to Bitcoin blockchain

### 4. External Integrations
- **Docker**: Containerized deployment
- **Nix**: Reproducible development environments
- **Testing**: Comprehensive test suites and frameworks

## Key Architectural Decisions

### ADR-001: Multi-Language Architecture
- **Decision**: Use Scala for node infrastructure, Rust for performance components
- **Rationale**: Leverage Scala's concurrency model while using Rust for system programming
- **Status**: Adopted

### ADR-002: Multi-Consensus Architecture
- **Decision**: Implement four consensus mechanisms: Cordial Miners, Casper CBC, RGB Partially Synchronized State Machines, and Casanova
- **Rationale**: Different consensus models optimize for different use cases and network conditions
- **Status**: Adopted

### ADR-003: RSpace Tuple Space Storage
- **Decision**: Use tuple space model for smart contract state storage
- **Rationale**: Enables concurrent execution and natural parallelism
- **Status**: Adopted

### ADR-004: Rholang Language Design
- **Decision**: Create domain-specific language based on ρ-calculus
- **Rationale**: Express concurrent computation naturally and safely
- **Status**: Adopted

## Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         RNode                               │
├─────────────────────────────────────────────────────────────┤
│  P2P Network Layer (Comm)                                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │  Block Sync     │ │  Peer Discovery │ │  Message      │  │
│  │                 │ │                 │ │  Propagation  │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Consensus Layer (Casper)                                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │  Block Creation │ │   Finalization  │ │   Validation  │  │
│  │                 │ │                 │ │               │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Execution Layer (Rholang/RSpace)                          │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │   Interpreter   │ │   Tuple Space   │ │ Cost Account  │  │
│  │                 │ │                 │ │               │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  Storage Layer                                              │
│  ┌─────────────────┐ ┌─────────────────┐ ┌───────────────┐  │
│  │  Block Storage  │ │   RSpace Store  │ │  DAG Storage  │  │
│  │                 │ │                 │ │               │  │
│  └─────────────────┘ └─────────────────┘ └───────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Component Dependencies

```
Node
├── Casper (Consensus)
│   ├── Block Storage
│   ├── Models
│   └── Crypto
├── Comm (Networking)
│   ├── Crypto
│   └── Models
├── Rholang (Smart Contracts)
│   ├── RSpace
│   ├── Models
│   └── Crypto
└── Node CLI
    ├── Crypto
    └── Models
```

## Performance Characteristics

- **Throughput**: Target 1000+ TPS with concurrent execution
- **Latency**: Block finality within 30-60 seconds
- **Storage**: Efficient state pruning and garbage collection
- **Network**: Optimized block propagation and validation

## Security Architecture

1. **Cryptographic Security**: Ed25519 and Secp256k1 signatures
2. **Network Security**: TLS 1.3 for all communications
3. **Economic Security**: Validator bonding and slashing
4. **Contract Security**: Capability-based security model

## Related Documents
- System requirements documentation
- Technical specifications
- Deployment guides
- Security analysis