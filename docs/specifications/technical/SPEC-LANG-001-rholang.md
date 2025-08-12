# Rholang Language Specification

## Specification ID
SPEC-LANG-001

## Version
1.0.0

## Overview
Rholang is a behaviorally typed, concurrent programming language based on the ρ-calculus. It is designed for writing smart contracts on the RNode platform.

## Language Fundamentals

### 1. Processes
Everything in Rholang is a process. Processes can:
- Send messages on channels
- Receive messages from channels
- Create new channels
- Run in parallel

### 2. Channels and Names
```rholang
// Creating a new channel
new myChannel in {
  // Channel is now available in this scope
}

// Unforgeable names
new unforgeableName in {
  // This name cannot be guessed or forged
}
```

### 3. Send and Receive
```rholang
// Send
channel!(data)

// Receive
for (x <- channel) {
  // Process x
}

// Persistent receive
contract channel(x) = {
  // Process x, remain listening
}
```

### 4. Pattern Matching
```rholang
for (@{x /\ y} <- channel) {
  // Match intersection of patterns
}

for (@[head, ...tail] <- channel) {
  // Match list patterns
}

for (@{"key": value, ...rest} <- channel) {
  // Match map patterns
}
```

## Core Syntax

### Process Grammar
```
P, Q ::= 0                     // Nil process
      | for (ptrn <- chan) { P }  // Receive
      | chan!(P)               // Send
      | P | Q                  // Parallel composition
      | new x in { P }         // New name
      | if (cond) { P } else { Q }  // Conditional
      | match X { ptrn => P }  // Pattern match
      | bundle+ { P }          // Read-only bundle
      | bundle- { P }          // Write-only bundle
      | bundle0 { P }          // Non-readable bundle
      | contract chan(ptrn) = { P }  // Persistent receive
```

### Name Grammar
```
N ::= @P                       // Quote process to name
    | x                        // Variable
    | Uri(string)              // URI literal
```

### Pattern Grammar
```
ptrn ::= _                     // Wildcard
      | x                      // Binding
      | @P                     // Process pattern
      | [ptrn, ...]           // List pattern
      | {key: ptrn, ...}      // Map pattern
      | ptrn /\ ptrn          // Logical AND
      | ptrn \/ ptrn          // Logical OR
```

## Built-in Types

### 1. Basic Types
- **Int**: Arbitrary precision integers
- **Bool**: Boolean values (true/false)
- **String**: UTF-8 encoded strings
- **ByteArray**: Raw byte sequences
- **Uri**: Uniform Resource Identifiers

### 2. Collections
- **List**: Ordered sequences `[1, 2, 3]`
- **Map**: Key-value pairs `{"a": 1, "b": 2}`
- **Set**: Unique elements `Set(1, 2, 3)`
- **Tuple**: Fixed-size sequences `(1, "hello", true)`

### 3. Processes as Values
- Processes can be quoted to create names: `@{ x!(42) }`
- Names can be unquoted to run processes: `*name`

## Standard Library

### 1. Registry
```rholang
new uriChan, insertArbitrary(`rho:registry:insertArbitrary`), 
    lookup(`rho:registry:lookup`) in {
  insertArbitrary!(bundle+{*uriChan}, *uriChan) |
  for(@uri <- uriChan) {
    // Use registered URI
  }
}
```

### 2. System Channels
- `stdout`: Print to standard output
- `stderr`: Print to standard error
- `rho:io:stdout`: Acknowledged stdout
- `rho:io:stderr`: Acknowledged stderr

### 3. Cryptographic Operations
```rholang
new hash(`rho:crypto:blake2b256Hash`),
    verify(`rho:crypto:secp256k1Verify`) in {
  hash!(data.toByteArray(), *result) |
  verify!(data, signature, pubKey, *verified)
}
```

## Execution Model

### 1. Concurrent Execution
- Processes separated by `|` run concurrently
- No guaranteed execution order
- Communication creates synchronization

### 2. Spatial Types
- Channels have spatial types based on usage
- Send/receive polarities tracked
- Type safety through behavioral typing

### 3. Cost Model
- Each operation has a phlogiston cost
- Costs are deterministic
- Resource limits enforced per deploy

## Security Features

### 1. Unforgeable Names
```rholang
new unforgeableName in {
  // This name is cryptographically unique
  // Cannot be guessed or recreated
}
```

### 2. Bundles for Capability Security
```rholang
bundle+ { process }  // Read-only capability
bundle- { process }  // Write-only capability
bundle0 { process }  // Opaque capability
```

### 3. Object Capabilities
- Access control through unforgeable names
- No ambient authority
- Principle of least privilege

## Examples

### Hello World
```rholang
new stdout(`rho:io:stdout`) in {
  stdout!("Hello, World!")
}
```

### Token Contract
```rholang
new MakeMint in {
  contract MakeMint(return) = {
    new mint, totalSupply in {
      totalSupply!(0) |
      contract mint(amount, recipient) = {
        for (@current <- totalSupply) {
          totalSupply!(current + amount) |
          recipient!(amount)
        }
      } |
      return!(*mint)
    }
  }
}
```

### Multi-signature Wallet
```rholang
contract MultiSigWallet(@threshold, @owners, return) = {
  new proposals, vote in {
    contract proposals(@proposalId, @action) = {
      new votes in {
        votes!(Set()) |
        contract vote(@owner, @approve) = {
          if (owners.contains(owner)) {
            for (@currentVotes <- votes) {
              if (approve) {
                votes!(currentVotes.add(owner)) |
                if (currentVotes.add(owner).size() >= threshold) {
                  // Execute action
                  @action
                }
              }
            }
          }
        }
      }
    } |
    return!(bundle+{*proposals}, bundle+{*vote})
  }
}
```

## Related Documents
- Rholang Tutorial
- RSpace Specification
- Cost Table Specification