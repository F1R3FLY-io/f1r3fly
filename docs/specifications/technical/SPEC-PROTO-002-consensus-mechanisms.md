# Multi-Consensus Protocol Specification

## Specification ID
SPEC-PROTO-002

## Version
1.0.0

## Overview
This specification defines the four consensus mechanisms implemented in RNode: Cordial Miners, Casper CBC, RGB Partially Synchronized State Machines, and Casanova.

## 1. Cordial Miners

### Protocol Overview
Cordial Miners implements a cooperative consensus mechanism that reduces energy consumption through collaborative block production.

### Block Production
```rholang
contract CordialMiner(@minerKey, @cooperativePool, return) = {
  new proposalChan, votesChan in {
    // Propose block collaboratively
    contract proposalChan(@blockCandidate) = {
      for (@pool <- cooperativePool) {
        // Share proposal with cooperative pool
        pool!("propose", blockCandidate, *minerKey) |
        
        // Wait for cooperative votes
        for (@votes <- votesChan) {
          if (votes.size() >= pool.threshold()) {
            // Create block with cooperative signatures
            return!(blockCandidate.withSignatures(votes))
          }
        }
      }
    }
  }
}
```

### Reward Distribution
- Block rewards shared among cooperative pool members
- Proportional distribution based on contribution
- Reduced individual mining costs through cooperation

### Energy Efficiency
- Target: 99% reduction in energy consumption vs. PoW
- Cooperative validation reduces redundant work
- Optimized for sustainable blockchain operations

## 2. Casper CBC (Correct by Construction)

### Protocol Overview
Casper CBC provides Byzantine Fault Tolerant consensus with mathematical safety proofs.

### Validator Set Management
```rholang
contract CasperValidator(@validatorKey, @stake, @bondingContract, return) = {
  new justificationsChan, blocksChan in {
    // Bond stake to become validator
    bondingContract!(validatorKey, stake) |
    
    // Listen for blocks to validate
    for (@block <- blocksChan) {
      // Create justification for block
      justificationsChan!(block.hash(), validatorKey.sign(block)) |
      
      // Check for finality
      for (@justifications <- justificationsChan) {
        if (justifications.size() >= stake.faultToleranceThreshold()) {
          return!(block.finalize(justifications))
        }
      }
    }
  }
}
```

### Safety Properties
- No two conflicting blocks can be finalized
- Accountable Byzantine fault tolerance
- Mathematical proof of safety under async conditions
- Slashing for equivocation detection

### Finality Mechanism
- Blocks finalized when impossible to revert
- Requires 2/3+ validator agreement
- Economic finality through bonded stake
- GHOST fork choice rule

## 3. RGB Partially Synchronized State Machines

### Protocol Overview
RGB PSSM implements client-side validation with Bitcoin anchoring for scalable smart contracts.

### State Machine Definition
```rholang
contract RGBStateMachine(@initialState, @transitionRules, return) = {
  new stateChan, commitmentsChan in {
    stateChan!(initialState) |
    
    contract stateChan(@currentState, @transition) = {
      // Validate transition client-side
      if (transitionRules.validate(currentState, transition)) {
        new newState in {
          newState!(transition.apply(currentState)) |
          
          // Create commitment for Bitcoin anchoring
          commitmentsChan!(newState.commitment()) |
          stateChan!(*newState)
        }
      }
    }
  }
}
```

### Bitcoin Anchoring
- State commitments anchored to Bitcoin blockchain
- OP_RETURN outputs for commitment proofs
- RGB protocol compatibility
- L1 security guarantees

### Client-Side Validation
- Transactions validated by interested parties
- Reduced global state requirements
- Enhanced privacy through selective disclosure
- Scalable to millions of transactions

### State Commitment Format
```
RGB_Commitment = Blake2b256(
  state_id || 
  prev_state_hash || 
  transition_proof || 
  timestamp
)
```

## 4. Casanova

### Protocol Overview
Casanova provides adaptive consensus that dynamically adjusts to network conditions for optimal performance.

### Adaptive Algorithm
```rholang
contract CasanovaConsensus(@networkMonitor, @adaptationRules, return) = {
  new consensusMode, performanceMetrics in {
    consensusMode!("fast_path") |
    
    // Monitor network conditions
    for (@metrics <- performanceMetrics) {
      for (@currentMode <- consensusMode) {
        new optimalMode in {
          adaptationRules!(metrics, *optimalMode) |
          
          for (@newMode <- optimalMode) {
            if (newMode != currentMode) {
              // Trigger consensus mode switch
              consensusMode!(newMode) |
              return!(newMode)
            } else {
              consensusMode!(currentMode)
            }
          }
        }
      }
    }
  }
}
```

### Consensus Modes
1. **Fast Path**: Optimistic consensus for low-contention scenarios
2. **Byzantine Path**: Full BFT when Byzantine behavior detected
3. **Partition Tolerance**: Graceful degradation during network partitions
4. **High Throughput**: Batched processing for high-load scenarios

### Performance Adaptation
- Real-time network condition monitoring
- Dynamic consensus parameter adjustment
- Automatic fallback mechanisms
- Load-based validator selection

## Consensus Selection Protocol

### Network Condition Assessment
```rholang
contract ConsensusSelector(@networkState, return) = {
  new selector in {
    match networkState {
      {"type": "public", "threat_level": "high"} => {
        return!("casper_cbc")
      }
      {"type": "private", "energy_priority": "high"} => {
        return!("cordial_miners")  
      }
      {"throughput": throughput, "privacy": "required"} => {
        if (throughput > 10000) {
          return!("rgb_pssm")
        }
      }
      {"adaptive": true, "performance": "critical"} => {
        return!("casanova")
      }
    }
  }
}
```

### Consensus Switching Protocol
1. **Proposal Phase**: Node proposes consensus change
2. **Validation Phase**: Network validates proposal
3. **Agreement Phase**: Validators reach consensus on switch
4. **Transition Phase**: Atomic switch to new consensus
5. **Confirmation Phase**: Verify successful transition

## Security Considerations

### Cross-Consensus Security
- State integrity during consensus transitions
- Protection against consensus-specific attacks
- Unified cryptographic foundations
- Audit trail for all consensus changes

### Consensus-Specific Threats
- **Cordial Miners**: Cooperative pool manipulation
- **Casper CBC**: Long-range attacks, nothing-at-stake
- **RGB PSSM**: State hiding attacks, commitment tampering
- **Casanova**: Adaptation manipulation, performance gaming

## Performance Specifications

| Metric | Cordial Miners | Casper CBC | RGB PSSM | Casanova |
|--------|----------------|------------|----------|----------|
| Throughput | 100 TPS | 1,000 TPS | 10,000 TPS | 5,000 TPS |
| Finality | 30 sec | 60 sec | 10 sec | 15 sec |
| Energy Use | Very Low | Low | Very Low | Medium |
| BFT Guarantee | No | Yes | Partial | Yes |

## Related Specifications
- SPEC-PROTO-001: Block Structure
- SPEC-LANG-001: Rholang Language
- SPEC-NET-001: Network Protocols