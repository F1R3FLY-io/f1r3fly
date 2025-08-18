# Block Structure Specification

## Specification ID
SPEC-PROTO-001

## Version
1.0.0

## Overview
This specification defines the structure and validation rules for blocks in the RNode blockchain.

## Block Structure

### Block Header
```protobuf
message BlockHeader {
  bytes parent_hash_list = 1;      // List of parent block hashes (multi-parent DAG)
  int64 timestamp = 2;             // Unix timestamp in milliseconds
  int64 version = 3;               // Protocol version
  int32 shard_id = 4;              // Shard identifier
  bytes extra_bytes = 5;           // Extension field for future use
}
```

### Block Body
```protobuf
message BlockBody {
  RChainState state = 1;           // Post-state hash after block execution
  repeated ProcessedDeploy deploys = 2;  // List of processed deployments
  repeated ProcessedSystemDeploy system_deploys = 3;  // System deployments
}
```

### Complete Block
```protobuf
message Block {
  bytes block_hash = 1;            // Blake2b256 hash of block content
  BlockHeader header = 2;          // Block header
  BlockBody body = 3;              // Block body
  repeated Justification justifications = 4;  // Validator justifications
  bytes sender = 5;                // Validator public key
  int32 seq_num = 6;              // Sequence number for validator
  bytes sig = 7;                   // Validator signature
  bytes sig_algorithm = 8;         // Signature algorithm used
  bytes shard_id = 9;             // Shard identifier
  int32 block_number = 10;        // Block height
  int64 pre_state_hash = 11;      // Pre-state hash
  int64 post_state_hash = 12;     // Post-state hash
  int64 bonds_cache = 13;         // Cached bond state
  int32 block_size = 14;          // Total block size in bytes
}
```

## Validation Rules

### 1. Block Hash Validation
- Block hash must be Blake2b256 of serialized block content
- Hash must include all fields except the hash itself
- Hash calculation must be deterministic

### 2. Signature Validation
- Signature must be valid for the sender's public key
- Supported algorithms: Secp256k1, Ed25519
- Signature covers block hash

### 3. Parent Validation
- All parent hashes must reference existing blocks
- Parents must be from compatible shards
- No circular dependencies allowed

### 4. Timestamp Validation
- Timestamp must be greater than all parent timestamps
- Timestamp cannot be more than 15 minutes in the future
- Timestamp precision: milliseconds

### 5. State Validation
- Pre-state hash must match post-state of parents
- Post-state hash must be result of applying deploys
- State transitions must be deterministic

## Block Creation Process

1. **Collect Deploys**: Validator collects pending deploys from mempool
2. **Select Parents**: Choose parent blocks based on fork choice rule
3. **Execute Deploys**: Run deploys in deterministic order
4. **Calculate State**: Compute post-state hash
5. **Create Block**: Assemble block structure
6. **Sign Block**: Add validator signature
7. **Calculate Hash**: Compute final block hash
8. **Broadcast**: Send block to network

## Size Limits

- Maximum block size: 10 MB
- Maximum deploys per block: 10,000
- Maximum parent references: 100
- Maximum justifications: 100

## Performance Considerations

- Block creation time: < 1 second
- Block validation time: < 500 ms
- Block propagation time: < 3 seconds
- Storage per block: ~50 KB average

## Security Considerations

- Blocks must be signed by bonded validators
- Invalid blocks result in slashing
- Equivocation detection via justifications
- Replay protection via nonces

## Examples

### Minimal Valid Block
```json
{
  "block_hash": "0x1234...abcd",
  "header": {
    "parent_hash_list": ["0xparent1", "0xparent2"],
    "timestamp": 1640000000000,
    "version": 1,
    "shard_id": 0
  },
  "body": {
    "state": {
      "post_state_hash": "0xstate123"
    },
    "deploys": []
  },
  "sender": "0xvalidator_pubkey",
  "seq_num": 42,
  "sig": "0xsignature",
  "block_number": 1000
}
```

## Related Specifications
- SPEC-PROTO-002: Transaction specification
- SPEC-PROTO-003: State specification
- SPEC-NET-001: Block propagation protocol