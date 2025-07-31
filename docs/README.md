# RNode Documentation

## Overview

RNode is a decentralized blockchain platform developed by F1R3FLY.io that implements four distinct consensus mechanisms using the Rholang programming language. The platform provides concurrent smart contract execution with Byzantine Fault Tolerant operations across multiple shards for enhanced scalability and performance.

## System Architecture

### High-Level Operation with Shards

```mermaid
graph TB
    subgraph "RNode Network"
        subgraph "Shard 0 (Root Shard)"
            S0_V1[Validator 1]
            S0_V2[Validator 2]
            S0_V3[Validator 3]
            S0_V4[Validator 4]
            S0_Consensus[Multi-Consensus Engine]
            S0_RSpace[RSpace Storage]
        end
        
        subgraph "Shard 1"
            S1_V1[Validator 1]
            S1_V2[Validator 2]
            S1_V3[Validator 3]
            S1_Consensus[Multi-Consensus Engine]
            S1_RSpace[RSpace Storage]
        end
        
        subgraph "Shard N"
            SN_V1[Validator 1]
            SN_V2[Validator 2]
            SN_V3[Validator 3]
            SN_Consensus[Multi-Consensus Engine]
            SN_RSpace[RSpace Storage]
        end
    end
    
    subgraph "Consensus Mechanisms"
        CM[Cordial Miners<br/>Cooperative Mining]
        CBC[Casper CBC<br/>BFT Consensus]
        RGB[RGB PSSM<br/>Client-Side Validation]
        CS[Casanova<br/>Adaptive Consensus]
    end
    
    subgraph "External Systems"
        Bitcoin[Bitcoin Network<br/>L1 Anchoring]
        Clients[Client Applications<br/>Smart Contracts]
        Observers[Observer Nodes<br/>Read-Only Access]
    end
    
    subgraph "Smart Contract Execution"
        Deploy[Deploy Rholang Contract]
        Propose[Propose Block]
        Execute[Concurrent Execution]
        Finalize[Block Finalization]
    end
    
    %% Shard Communications
    S0_Consensus -.->|Cross-Shard Messages| S1_Consensus
    S0_Consensus -.->|Cross-Shard Messages| SN_Consensus
    S1_Consensus -.->|Cross-Shard Messages| SN_Consensus
    
    %% Consensus Selection
    S0_Consensus --> CM
    S0_Consensus --> CBC
    S0_Consensus --> RGB
    S0_Consensus --> CS
    
    S1_Consensus --> CM
    S1_Consensus --> CBC
    S1_Consensus --> RGB
    S1_Consensus --> CS
    
    SN_Consensus --> CM
    SN_Consensus --> CBC
    SN_Consensus --> RGB
    SN_Consensus --> CS
    
    %% Bitcoin Anchoring
    S0_RSpace -->|State Commitments| Bitcoin
    S1_RSpace -->|State Commitments| Bitcoin
    SN_RSpace -->|State Commitments| Bitcoin
    
    %% Client Interactions
    Clients -->|Deploy Contracts| Deploy
    Deploy -->|Assign to Shard| S0_Consensus
    Deploy -->|Assign to Shard| S1_Consensus
    Deploy -->|Assign to Shard| SN_Consensus
    
    %% Contract Execution Flow
    Deploy --> Propose
    Propose --> Execute
    Execute --> Finalize
    
    %% RSpace Integration
    Execute -->|State Updates| S0_RSpace
    Execute -->|State Updates| S1_RSpace
    Execute -->|State Updates| SN_RSpace
    
    %% Observer Access
    Observers -->|Read State| S0_RSpace
    Observers -->|Read State| S1_RSpace
    Observers -->|Read State| SN_RSpace
    
    %% Validator Participation
    S0_V1 --> S0_Consensus
    S0_V2 --> S0_Consensus
    S0_V3 --> S0_Consensus
    S0_V4 --> S0_Consensus
    
    S1_V1 --> S1_Consensus
    S1_V2 --> S1_Consensus
    S1_V3 --> S1_Consensus
    
    SN_V1 --> SN_Consensus
    SN_V2 --> SN_Consensus
    SN_V3 --> SN_Consensus
    
    classDef shard fill:#e1f5fe,stroke:#01579b,stroke-width:2px
    classDef consensus fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef external fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef execution fill:#fff3e0,stroke:#e65100,stroke-width:2px
    
    class S0_V1,S0_V2,S0_V3,S0_V4,S1_V1,S1_V2,S1_V3,SN_V1,SN_V2,SN_V3,S0_RSpace,S1_RSpace,SN_RSpace shard
    class CM,CBC,RGB,CS,S0_Consensus,S1_Consensus,SN_Consensus consensus
    class Bitcoin,Clients,Observers external
    class Deploy,Propose,Execute,Finalize execution
```

### Key Architecture Components

#### **Sharding Model**
- **Root Shard (Shard 0)**: Coordinates cross-shard operations and maintains global state
- **Child Shards**: Process transactions independently with periodic synchronization
- **Cross-Shard Communication**: Secure message passing between shards
- **State Finality**: Coordinated finalization across all shards

#### **Multi-Consensus Engine**
Each shard can independently select and switch between four consensus mechanisms:
- **Cordial Miners**: Energy-efficient cooperative mining
- **Casper CBC**: Byzantine Fault Tolerant with mathematical safety proofs
- **RGB PSSM**: Client-side validation with Bitcoin L1 anchoring
- **Casanova**: Adaptive consensus optimizing for current network conditions

#### **RSpace Tuple Space**
- **Concurrent Storage**: Each shard maintains its own RSpace instance
- **Cross-Shard Queries**: Ability to query state across multiple shards
- **State Synchronization**: Coordinated state updates and consistency guarantees
- **Bitcoin Anchoring**: Periodic state commitments to Bitcoin for additional security

## Documentation Structure

### =Ë Requirements
- **[Requirements Overview](requirements/)** - Business and user requirements
  - **[User Stories](requirements/user-stories/)** - Feature requirements from user perspective
  - **[Business Requirements](requirements/business-requirements/)** - Business logic and constraints
  - **[Acceptance Criteria](requirements/acceptance-criteria/)** - Definition of done for features

### =Ð Specifications  
- **[Specifications Overview](specifications/)** - Technical specifications and design documents
  - **[Technical Specifications](specifications/technical/)** - API specs, data schemas, algorithms
  - **[Visual Design](specifications/visual-design/)** - UI/UX mockups and style guides
  - **[Integration Specifications](specifications/integration/)** - Third-party service integrations

### <× Architecture
- **[Architecture Overview](architecture/)** - System design and architectural decisions
  - **[Architecture Decision Records](architecture/decisions/)** - ADRs documenting key decisions
  - **[System Diagrams](architecture/diagrams/)** - Component diagrams and data flows
  - **[Design Patterns](architecture/patterns/)** - Established patterns and conventions

### =Ö API Documentation
- **[API Overview](api/)** - Complete API reference and examples
  - **[Node Operations](api/node-operations.md)** - Core node functionality
  - **[Smart Contracts](api/smart-contracts.md)** - Contract deployment and execution
  - **[Consensus APIs](api/consensus-apis.md)** - Multi-consensus management
  - **[Network APIs](api/network-apis.md)** - P2P networking operations

###  Current Status
- **[Project Status](ToDos.md)** - Live project status, active tasks, and priorities

## Quick Navigation

### For New Developers
1. **[Getting Started](../README.md#installation)** - Environment setup
2. **[Developer Guide](../DEVELOPER.md)** - Development workflow
3. **[API Documentation](api/)** - API reference and examples
4. **[Architecture Overview](architecture/)** - System design understanding

### For Node Operators
1. **[Node Operations](requirements/user-stories/US-CORE-001-node-operator.md)** - Operator requirements
2. **[Deployment Guide](../README.md#running)** - Node deployment
3. **[Configuration](../README.md#configuration)** - Node configuration options
4. **[Troubleshooting](../README.md#troubleshooting)** - Common issues and solutions

### For Smart Contract Developers
1. **[Contract Development](requirements/user-stories/US-SMART-001-developer-deploy.md)** - Developer workflow
2. **[Rholang Language](specifications/technical/SPEC-LANG-001-rholang.md)** - Language specification
3. **[CLI Tools](../node-cli/README.md)** - Command-line interface
4. **[Examples](../rholang/examples/)** - Contract examples

### For Researchers & Contributors
1. **[Multi-Consensus Architecture](architecture/decisions/ADR-002-multi-consensus-architecture.md)** - Consensus design
2. **[Contributing Guide](../CONTRIBUTING.md)** - Contribution workflow
3. **[Technical Specifications](specifications/technical/)** - Detailed technical specs
4. **[Current Development Status](ToDos.md)** - Active development tasks

## Development Resources

### Key Technologies
- **Languages**: Scala (node), Rust (performance), Rholang (contracts)
- **Consensus**: Cordial Miners, Casper CBC, RGB PSSM, Casanova
- **Storage**: RSpace tuple space with LMDB backend
- **Networking**: gRPC with TLS 1.3, P2P with Kademlia DHT
- **Environment**: Nix/Direnv for reproducible development

### External Resources
- **[F1R3FLY Discord](https://discord.gg/NN59aFdAHM)** - Community support and discussions
- **[Bitcoin Anchoring](../bitcoin-anchor/README.md)** - Layer 1 security integration
- **[RNodeFS](https://github.com/F1R3FLY-io/rnodefs#rnodefs)** - File system built on RNode
- **[Docker Hub](https://hub.docker.com/r/f1r3flyindustries/rnode-rust-node)** - Container images

## Contributing to Documentation

This documentation follows F1R3FLY.io's documentation-first development methodology. When contributing:

1. **Requirements First**: Start with user stories and business requirements
2. **Specifications**: Create detailed technical specifications
3. **Architecture**: Document architectural decisions with ADRs
4. **Implementation**: Build according to documented specifications
5. **Update Status**: Keep `ToDos.md` current with progress

For specific contribution guidelines, see our **[Contributing Guide](../CONTRIBUTING.md)**.