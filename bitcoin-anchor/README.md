# Bitcoin Anchor for F1r3fly

**Current Status: Steps 1-2 Complete**

Minimal crate structure focused on core commitment functionality for Bitcoin L1 anchoring of F1r3fly RSpace state.

## Architecture (Current)

```
f1r3fly/bitcoin-anchor/
├── Cargo.toml              # Minimal dependencies (thiserror, serde, blake2)
├── src/
│   ├── lib.rs              # Public API with basic structure
│   ├── error.rs            # Error types for Steps 1-2
│   ├── config/             # Configuration management
│   │   ├── mod.rs
│   │   └── anchor_config.rs # AnchorConfig, AnchorMethod, AnchorFrequency
│   └── commitment/         # Core commitment logic (Step 2)
│       ├── mod.rs
│       ├── f1r3fly_state.rs    # F1r3flyStateCommitment struct
│       └── serialization.rs   # Deterministic serialization
└── README.md
```

## What's Implemented

### ✅ Step 1: Basic Structure
- **Configuration**: `AnchorConfig` with method (Taproot/OpReturn/Auto) and frequency settings
- **Error Handling**: `AnchorError` enum with proper error types
- **Public API**: `F1r3flyBitcoinAnchor` struct with basic interface

### ✅ Step 2: State Commitment
- **`F1r3flyStateCommitment`**: Core structure containing:
  - Last Finalized Block hash (32 bytes)
  - RSpace state root (32 bytes)  
  - Block height (i64)
  - Timestamp (u64)
  - Validator set hash (32 bytes)
- **Bitcoin Commitment**: `to_bitcoin_commitment()` generates 32-byte Blake2b hash
- **Deterministic Serialization**: Fixed binary format for consistency
- **Tests**: Comprehensive test coverage for commitment creation and serialization

## Usage

```rust
use bitcoin_anchor::{F1r3flyBitcoinAnchor, AnchorConfig, F1r3flyStateCommitment};

// Create configuration
let config = AnchorConfig::default(); // Uses Auto method, Manual frequency

// Create anchor service
let anchor = F1r3flyBitcoinAnchor::new(config)?;

// Create state commitment
let commitment = F1r3flyStateCommitment::new(
    lfb_hash,           // [u8; 32] from Casper
    rspace_root,        // [u8; 32] from RSpace++
    block_height,       // i64 from Casper
    timestamp,          // u64 current time
    validator_set_hash, // [u8; 32] computed from validators
);

// Generate Bitcoin commitment hash
let bitcoin_hash = commitment.to_bitcoin_commitment()?; // [u8; 32]
```

## Dependencies

- **thiserror**: Error handling
- **serde**: Configuration serialization  
- **blake2**: Cryptographic hashing for commitments

## Next Steps

- **Step 3**: Add F1r3fly integration (casper, rspace++, block-storage dependencies)
- **Step 4**: Add Bitcoin client functionality (bitcoin, bitcoincore-rpc dependencies)  
- **Step 5**: Integrate RGB infrastructure (bp-core, rgb-core, commit-verify dependencies)

## Testing

```bash
cd f1r3fly/bitcoin-anchor
cargo test
```

All tests pass with comprehensive coverage of commitment creation and deterministic serialization. 