# ADR-002: Multi-Consensus Architecture

## Status
Adopted

## Context
RNode needs to support different consensus mechanisms for varying network conditions, use cases, and performance requirements. A single consensus approach cannot optimally serve all scenarios in a diverse blockchain ecosystem.

## Decision
We will implement four distinct consensus mechanisms using Rholang: Cordial Miners, Casper CBC, RGB Partially Synchronized State Machines, and Casanova.

## Rationale

### 1. Cordial Miners
**Use Case**: Energy-efficient, cooperative mining environments
- **Advantages**: Reduced energy consumption, collaborative approach
- **Optimal For**: Environmentally conscious deployments, consortium chains
- **Implementation**: Cooperative block production with shared rewards

### 2. Casper CBC (Correct by Construction)
**Use Case**: High-security, Byzantine Fault Tolerant networks
- **Advantages**: Mathematical safety proofs, asynchronous operation
- **Optimal For**: Public networks with untrusted validators
- **Implementation**: Proof-of-stake with economic finality

### 3. RGB Partially Synchronized State Machines
**Use Case**: Privacy-focused, scalable smart contracts
- **Advantages**: Client-side validation, Bitcoin anchoring, privacy preservation
- **Optimal For**: High-throughput applications with privacy requirements
- **Implementation**: Off-chain state transitions with on-chain commitments

### 4. Casanova
**Use Case**: High-performance, adaptive consensus
- **Advantages**: Dynamic adaptation, optimized throughput
- **Optimal For**: Enterprise applications with varying load patterns
- **Implementation**: Adaptive consensus with intelligent switching

## Architecture Design

### Pluggable Consensus Framework
```
┌─────────────────────────────────────────────────────────┐
│                 Consensus Abstraction Layer            │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────┐   │
│  │   Cordial   │ │   Casper    │ │  RGB Partially  │   │
│  │   Miners    │ │    CBC      │ │  Synchronized   │   │
│  │             │ │             │ │ State Machines  │   │
│  └─────────────┘ └─────────────┘ └─────────────────┘   │
│                    ┌─────────────┐                      │
│                    │  Casanova   │                      │
│                    │             │                      │
│                    └─────────────┘                      │
├─────────────────────────────────────────────────────────┤
│              Rholang Implementation Layer               │
└─────────────────────────────────────────────────────────┘
```

### Consensus Selection Criteria
1. **Network Type**: Public vs. Private vs. Consortium
2. **Security Requirements**: Byzantine tolerance vs. Crash tolerance
3. **Performance Needs**: Throughput vs. Latency optimization
4. **Privacy Requirements**: Public vs. Private state
5. **Energy Constraints**: Power consumption considerations

## Implementation Strategy

### 1. Common Abstractions
- **Consensus Interface**: Unified API for all consensus mechanisms
- **Block Structure**: Common block format across consensus types
- **State Management**: Shared state transition framework
- **Network Layer**: Common P2P communication protocols

### 2. Rholang Implementation
- Each consensus mechanism implemented as Rholang contracts
- Shared Rholang libraries for common functionality
- Consensus-specific optimizations in Rholang
- Runtime consensus switching through contract calls

### 3. Configuration Management
```rholang
// Consensus configuration contract
contract ConsensusConfig(return) = {
  new config in {
    config!({
      "default": "casper_cbc",
      "available": ["cordial_miners", "casper_cbc", "rgb_pssm", "casanova"],
      "switching_rules": {
        "network_load": {"high": "casanova", "low": "cordial_miners"},
        "security_level": {"high": "casper_cbc", "medium": "rgb_pssm"}
      }
    }) |
    return!(*config)
  }
}
```

## Consensus Switching Protocol

### 1. Trigger Conditions
- Network performance degradation
- Security threat detection
- Administrative configuration changes
- Scheduled consensus upgrades

### 2. Switching Process
1. **Consensus Proposal**: Validators propose consensus change
2. **Agreement Phase**: Network reaches agreement on new consensus
3. **Transition Block**: Special block marks consensus transition
4. **State Migration**: Transfer state to new consensus mechanism
5. **Activation**: New consensus becomes active

### 3. Safety Guarantees
- No state loss during transitions
- Maintain Byzantine fault tolerance throughout switch
- Rollback capability for failed transitions
- Consensus history preservation

## Performance Characteristics

| Consensus | Throughput | Latency | Energy | Privacy | BFT |
|-----------|------------|---------|---------|---------|-----|
| Cordial Miners | Medium | Low | Very Low | Low | No |
| Casper CBC | Medium | Medium | Low | Low | Yes |
| RGB PSSM | High | Low | Very Low | High | Partial |
| Casanova | Very High | Very Low | Medium | Medium | Yes |

## Consequences

### Positive
- **Flexibility**: Optimal consensus for each scenario
- **Resilience**: Fallback mechanisms for network issues
- **Innovation**: Platform for consensus research and development
- **Adaptability**: Dynamic response to changing network conditions

### Negative
- **Complexity**: Increased system complexity and testing burden
- **Security Surface**: Multiple consensus implementations to secure
- **Development Overhead**: Maintenance of four different mechanisms
- **Transition Risks**: Potential issues during consensus switching

## Monitoring and Success Metrics

### Performance Metrics
- Transaction throughput per consensus type
- Block finality times
- Network resource utilization
- Consensus switching frequency and success rate

### Security Metrics
- Byzantine fault tolerance verification
- Consensus attack resistance
- State consistency across transitions
- Security audit results for each mechanism

## Related Decisions
- ADR-001: Multi-Language Architecture
- ADR-003: RSpace Storage Model
- ADR-004: Rholang Language Design

## References
- [Casper CBC Specification](https://github.com/ethereum/cbc-casper)
- [RGB Protocol Documentation](https://rgb-org.github.io/)
- [Cordial Miners Research](https://cordialminers.org/)
- [ρ-calculus Foundations](https://en.wikipedia.org/wiki/Rho_calculus)