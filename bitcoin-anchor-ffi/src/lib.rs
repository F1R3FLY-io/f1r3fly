//! Bitcoin Anchor FFI Bridge for F1r3fly
//!
//! This crate provides a Foreign Function Interface (FFI) bridge between
//! F1r3fly's Scala code and the bitcoin-anchor Rust library. It exposes
//! C-compatible functions that can be called via JNA from Scala.
//!
//! ## Architecture
//!
//! The FFI bridge handles:
//! - Protobuf serialization/deserialization across the FFI boundary
//! - Memory management between Rust and Scala (JNA)  
//! - Error handling and conversion
//! - Async-to-sync bridging for Bitcoin operations
//!
//! ## Memory Management
//!
//! All data passed across the FFI boundary uses a length-prefixed format:
//! - `[4 bytes: length][data bytes]`
//! - Rust allocates memory and returns pointers
//! - Scala must call `deallocate_memory()` to prevent leaks
//!
//! ## Error Handling
//!
//! All FFI functions return structured results via protobuf messages.
//! Functions never panic across the FFI boundary.

use std::ffi::c_void;
use std::ptr;
use std::slice;
use std::sync::Once;

use prost::Message;
use bitcoin_anchor::{F1r3flyStateCommitment, F1r3flyBitcoinAnchor, AnchorConfig};

// Import our protobuf types from models
use models::bitcoin_anchor::{
    F1r3flyStateCommitmentProto, 
    BitcoinAnchorResultProto, 
    BitcoinAnchorConfigProto
};

/// FFI Result type for consistent error handling
type FFIResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Global logger initialization - called once when the library is loaded
static INIT_LOGGER: Once = Once::new();

fn init_logger() {
    INIT_LOGGER.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug) // Default to Debug level
            .init();
        log::debug!("Bitcoin Anchor FFI logger initialized");
    });
}

/// Opaque handle to a BitcoinAnchor instance
pub struct BitcoinAnchorHandle {
    anchor: F1r3flyBitcoinAnchor,
}

/// Create a new Bitcoin anchor instance from configuration
///
/// # Safety
/// 
/// - `config_ptr` must point to valid protobuf data
/// - `config_len` must be the exact length of the data
/// - Caller must call `destroy_bitcoin_anchor()` to free the handle
#[no_mangle]
pub extern "C" fn create_bitcoin_anchor(
    config_ptr: *const u8,
    config_len: usize,
) -> *mut BitcoinAnchorHandle {
    // Initialize logger on first FFI call
    init_logger();
    
    match create_bitcoin_anchor_internal(config_ptr, config_len) {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(e) => {
            log::error!("Failed to create bitcoin anchor: {}", e);
            ptr::null_mut()
        }
    }
}

fn create_bitcoin_anchor_internal(
    config_ptr: *const u8,
    config_len: usize,
) -> FFIResult<BitcoinAnchorHandle> {
    // Safety: We assume the caller provides valid pointer and length
    let config_bytes = unsafe { slice::from_raw_parts(config_ptr, config_len) };
    
    // Deserialize configuration
    let config_proto = BitcoinAnchorConfigProto::decode(config_bytes)?;
    
    log::info!("Creating bitcoin anchor with network: {}", config_proto.network);
    
    // Convert protobuf config to native config
    let anchor_config = match config_proto.network.as_str() {
        "mainnet" => AnchorConfig::mainnet(),
        "signet" => AnchorConfig::signet(), 
        "regtest" => AnchorConfig::regtest(),
        _ => return Err(format!("Unknown network: {}", config_proto.network).into()),
    };
    let anchor = F1r3flyBitcoinAnchor::new(anchor_config)?;
    
    Ok(BitcoinAnchorHandle { anchor })
}

/// Destroy a Bitcoin anchor instance and free its memory
///
/// # Safety
///
/// - `handle` must be a valid pointer returned by `create_bitcoin_anchor()`
/// - Must not be called more than once on the same handle
/// - Handle becomes invalid after this call
#[no_mangle]
pub extern "C" fn destroy_bitcoin_anchor(handle: *mut BitcoinAnchorHandle) {
    if !handle.is_null() {
        unsafe {
            let _handle = Box::from_raw(handle);
            // Handle is dropped and memory freed
        }
        log::debug!("Bitcoin anchor handle destroyed");
    }
}

/// Process F1r3fly state finalization and create Bitcoin anchor
///
/// # Safety
///
/// - `handle` must be a valid BitcoinAnchorHandle pointer
/// - `state_ptr` must point to valid F1r3flyStateCommitmentProto data
/// - `state_len` must be the exact length of the state data
/// - Caller must call `deallocate_memory()` on the returned pointer
///
/// # Returns
///
/// Returns a pointer to length-prefixed BitcoinAnchorResultProto data.
/// Format: `[4 bytes: length][BitcoinAnchorResultProto bytes]`
/// Returns null pointer on error.
#[no_mangle]
pub extern "C" fn anchor_finalization(
    handle: *mut BitcoinAnchorHandle,
    state_ptr: *const u8,
    state_len: usize,
) -> *const u8 {
    match anchor_finalization_internal(handle, state_ptr, state_len) {
        Ok(result_ptr) => result_ptr,
        Err(e) => {
            log::error!("Bitcoin anchor finalization failed: {}", e);
            
            // Return error result as protobuf
            let error_result = BitcoinAnchorResultProto {
                success: false,
                error_message: e.to_string(),
                transaction_id: String::new(),
                fee_sats: 0,
                debug_info: format!("FFI Error: {}", e),
            };
            
            match serialize_result_with_length(&error_result) {
                Ok(ptr) => ptr,
                Err(_) => ptr::null(),
            }
        }
    }
}

fn anchor_finalization_internal(
    handle: *mut BitcoinAnchorHandle,
    state_ptr: *const u8,
    state_len: usize,
) -> FFIResult<*const u8> {
    // Validate handle
    if handle.is_null() {
        return Err("Invalid bitcoin anchor handle".into());
    }
    
    let anchor_handle = unsafe { &*handle };
    
    // Safety: We assume the caller provides valid pointer and length  
    let state_bytes = unsafe { slice::from_raw_parts(state_ptr, state_len) };
    
    // Deserialize F1r3fly state commitment
    let state_proto = F1r3flyStateCommitmentProto::decode(state_bytes)?;
    
    log::info!("Processing finalization for block height: {}", state_proto.block_height);
    log::debug!("LFB hash: {}", hex::encode(&state_proto.lfb_hash));
    log::debug!("RSpace root: {}", hex::encode(&state_proto.rspace_root));
    
    // Convert protobuf to native F1r3fly commitment
    let commitment = convert_proto_to_commitment(&state_proto)?;
    
    // Create real Bitcoin commitment using build_commitment_only
    let result = match unsafe { create_real_bitcoin_commitment(&(*handle).anchor, &commitment, &state_proto) } {
        Ok(commitment_result) => {
            log::info!("Bitcoin anchor completed successfully: {}", commitment_result.transaction_id);
            commitment_result
        }
        Err(e) => {
            log::error!("Bitcoin anchor failed: {}", e);
            BitcoinAnchorResultProto {
                success: false,
                error_message: format!("Bitcoin anchor error: {}", e),
                transaction_id: String::new(),
                fee_sats: 0,
                debug_info: format!("Failed to create commitment for block {}", state_proto.block_height),
            }
        }
    };
    
    serialize_result_with_length(&result)
}

/// Create real Bitcoin commitment using the bitcoin-anchor crate
///
/// This function uses build_commitment_only to create actual Bitcoin commitment
/// structures without requiring UTXOs or signing.
fn create_real_bitcoin_commitment(
    anchor: &F1r3flyBitcoinAnchor,
    commitment: &F1r3flyStateCommitment,
    state_proto: &F1r3flyStateCommitmentProto,
) -> FFIResult<BitcoinAnchorResultProto> {
    use bitcoin::Address;
    use std::str::FromStr;
    
    log::debug!("Creating Bitcoin commitment for F1r3fly state");
    
    // Validate commitment data (Phase 2 requirement)
    validate_commitment_data(commitment, state_proto)?;
    
    // Create PSBT transaction with the F1r3fly commitment
    // This creates the Bitcoin commitment structure with the actual F1r3fly state
    let commitment_psbt = anchor.build_psbt_transaction(
        commitment,
        bitcoin_anchor::Sats(1000), // Minimal fee for structure
    )?;
    
    // Extract real transaction ID and commitment data
    let transaction_id = commitment_psbt.txid().to_string();
    let fee_sats = 1000; // Fixed fee for commitment-only operations
    
    // Create debug info with real commitment details
    let debug_info = format!(
        "Real Bitcoin commitment created:\n\
        Block: {}\n\
        Network: {}\n\
        TXID: {}\n\
        LFB Hash: {}\n\
        RSpace Root: {}\n\
        Validator Set: {}\n\
        Commitment verified: {}",
        state_proto.block_height,
        anchor.network(),
        transaction_id,
        hex::encode(&state_proto.lfb_hash),
        hex::encode(&state_proto.rspace_root),
        hex::encode(&state_proto.validator_set_hash),
        commitment_psbt.verify_commitment()
    );
    
    log::info!("Bitcoin commitment created - TXID: {}", transaction_id);
    log::debug!("Commitment details: {}", debug_info);
    
    // Ensure proper cleanup of Bitcoin objects (Phase 2 requirement)
    // The commitment_psbt will be automatically dropped here, ensuring memory cleanup
    drop(commitment_psbt);
    
    Ok(BitcoinAnchorResultProto {
        success: true,
        error_message: String::new(),
        transaction_id,
        fee_sats,
        debug_info,
    })
}

/// Validate F1r3fly commitment data before creating Bitcoin commitment (Phase 2)
fn validate_commitment_data(
    commitment: &F1r3flyStateCommitment,
    state_proto: &F1r3flyStateCommitmentProto,
) -> FFIResult<()> {
    // Validate hash lengths (must be exactly 32 bytes)
    if state_proto.lfb_hash.len() != 32 {
        return Err(format!("Invalid LFB hash length: {} (expected 32)", state_proto.lfb_hash.len()).into());
    }
    
    if state_proto.rspace_root.len() != 32 {
        return Err(format!("Invalid RSpace root length: {} (expected 32)", state_proto.rspace_root.len()).into());
    }
    
    if state_proto.validator_set_hash.len() != 32 {
        return Err(format!("Invalid validator set hash length: {} (expected 32)", state_proto.validator_set_hash.len()).into());
    }
    
    // Validate block height (must be non-negative)
    if state_proto.block_height < 0 {
        return Err(format!("Invalid block height: {} (must be non-negative)", state_proto.block_height).into());
    }
    
    // Validate timestamp (must be reasonable - not zero and not too far in future)
    if state_proto.timestamp == 0 {
        return Err("Invalid timestamp: cannot be zero".into());
    }
    
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    if state_proto.timestamp > current_time + (3600 * 1000) { // Allow 1 hour future tolerance (in milliseconds)
        return Err(format!("Invalid timestamp: {} is too far in the future (current time: {})", 
                          state_proto.timestamp, current_time).into());
    }
    
    // Verify that native commitment matches protobuf data
    if commitment.lfb_hash != state_proto.lfb_hash.as_slice() {
        return Err("LFB hash mismatch between protobuf and native commitment".into());
    }
    
    if commitment.rspace_root != state_proto.rspace_root.as_slice() {
        return Err("RSpace root mismatch between protobuf and native commitment".into());
    }
    
    if commitment.validator_set_hash != state_proto.validator_set_hash.as_slice() {
        return Err("Validator set hash mismatch between protobuf and native commitment".into());
    }
    
    log::debug!("Commitment data validation passed");
    Ok(())
}

fn convert_proto_to_commitment(
    proto: &F1r3flyStateCommitmentProto,
) -> FFIResult<F1r3flyStateCommitment> {
    // Convert Vec<u8> to [u8; 32] arrays
    let lfb_hash: [u8; 32] = proto.lfb_hash.as_slice()
        .try_into()
        .map_err(|_| "Invalid LFB hash length")?;
        
    let rspace_root: [u8; 32] = proto.rspace_root.as_slice()
        .try_into()
        .map_err(|_| "Invalid RSpace root length")?;
        
    let validator_set_hash: [u8; 32] = proto.validator_set_hash.as_slice()
        .try_into()
        .map_err(|_| "Invalid validator set hash length")?;
    
    Ok(F1r3flyStateCommitment::new(
        lfb_hash,
        rspace_root,
        proto.block_height,
        proto.timestamp,
        validator_set_hash,
    ))
}

fn serialize_result_with_length(result: &BitcoinAnchorResultProto) -> FFIResult<*const u8> {
    let result_bytes = result.encode_to_vec();
    let len = result_bytes.len() as u32;
    
    // Create length-prefixed data: [4 bytes length][data]
    let mut data_with_length = len.to_le_bytes().to_vec();
    data_with_length.extend(result_bytes);
    
    // Leak the memory so it can be accessed from Scala
    // Scala must call deallocate_memory() to free this
    let boxed_data = data_with_length.into_boxed_slice();
    Ok(Box::leak(boxed_data).as_ptr() as *const u8)
}

/// Deallocate memory that was allocated by Rust and returned to Scala
///
/// # Safety
///
/// - `ptr` must be a pointer returned by a Rust FFI function
/// - `len` must be the exact length of the allocated data
/// - Must not be called more than once on the same pointer
#[no_mangle]
pub extern "C" fn deallocate_memory(ptr: *const u8, len: usize) {
    if !ptr.is_null() && len > 0 {
        unsafe {
            // Reconstruct the Box and let it drop to free memory
            let _data = Box::from_raw(slice::from_raw_parts_mut(ptr as *mut u8, len));
        }
        log::trace!("Deallocated {} bytes at {:p}", len, ptr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ffi_basic_flow() {
        // Test basic FFI flow with mock data
        let config = BitcoinAnchorConfigProto {
            network: "regtest".to_string(),
            enabled: true,
            esplora_url: "http://localhost:3002".to_string(),
            fee_rate: 1.0,
            max_fee_sats: 10000,
        };
        
        let config_bytes = config.encode_to_vec();
        
        // Test handle creation
        let handle = create_bitcoin_anchor(config_bytes.as_ptr(), config_bytes.len());
        assert!(!handle.is_null());
        
        // Clean up
        destroy_bitcoin_anchor(handle);
    }
    
    #[test]
    fn test_protobuf_conversion() {
        let proto = F1r3flyStateCommitmentProto {
            lfb_hash: vec![1u8; 32],
            rspace_root: vec![2u8; 32],
            block_height: 12345,
            timestamp: 1234567890,
            validator_set_hash: vec![3u8; 32],
        };
        
        let commitment = convert_proto_to_commitment(&proto).unwrap();
        assert_eq!(commitment.block_height, 12345);
        assert_eq!(commitment.timestamp, 1234567890);
        assert_eq!(commitment.lfb_hash, [1u8; 32]);
    }
}
