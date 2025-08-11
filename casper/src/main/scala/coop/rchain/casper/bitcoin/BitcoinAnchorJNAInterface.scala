package coop.rchain.casper.bitcoin

import com.sun.jna.{Library, Native, Pointer}

/**
  * JNA interface for calling Bitcoin anchor FFI functions from Scala.
  *
  * This interface provides a bridge to the Rust bitcoin-anchor-ffi library
  * using Java Native Access (JNA). All data is passed via protobuf serialization
  * with length-prefixed format for memory safety.
  *
  * Memory Management:
  * - All functions that return data allocate memory in Rust
  * - Caller MUST call deallocateMemory() to prevent memory leaks
  * - Failed function calls return null pointers
  *
  * Data Format:
  * - Input: Raw protobuf bytes with exact length
  * - Output: Length-prefixed format: [4 bytes: length][protobuf data]
  *
  * @see bitcoin-anchor-ffi/src/lib.rs for the corresponding Rust implementation
  */
trait BitcoinAnchorJNAInterface extends Library {

  /**
    * Create a new Bitcoin anchor instance from configuration.
    *
    * @param configPtr Pointer to BitcoinAnchorConfigProto serialized data
    * @param configLen Length of the configuration data in bytes
    * @return Opaque handle to BitcoinAnchor instance, or null on failure
    *
    * @note Caller must call destroy_bitcoin_anchor() to free the handle
    */
  def create_bitcoin_anchor(configPtr: Pointer, configLen: Int): Pointer

  /**
    * Destroy a Bitcoin anchor instance and free its memory.
    *
    * @param handle Valid handle returned by create_bitcoin_anchor()
    *
    * @note Must not be called more than once on the same handle
    * @note Handle becomes invalid after this call
    */
  def destroy_bitcoin_anchor(handle: Pointer): Unit

  /**
    * Process F1r3fly state finalization and create Bitcoin anchor.
    *
    * @param handle Valid BitcoinAnchor handle
    * @param statePtr Pointer to F1r3flyStateCommitmentProto serialized data
    * @param stateLen Length of the state data in bytes
    * @return Pointer to length-prefixed BitcoinAnchorResultProto data, or null on error
    *
    * @note Caller MUST call deallocate_memory() on the returned pointer
    * @note Return format: [4 bytes: length][BitcoinAnchorResultProto bytes]
    */
  def anchor_finalization(handle: Pointer, statePtr: Pointer, stateLen: Int): Pointer

  /**
    * Deallocate memory that was allocated by Rust FFI functions.
    *
    * @param ptr Pointer returned by a Rust FFI function
    * @param len Length of the allocated data in bytes
    *
    * @note Must not be called more than once on the same pointer
    * @note Required for all non-null pointers returned by FFI functions
    */
  def deallocate_memory(ptr: Pointer, len: Int): Unit
}

/**
  * Singleton object for loading and accessing the Bitcoin anchor native library.
  *
  * The library is loaded from the rust_libraries/release directory and provides
  * thread-safe access to the JNA interface.
  */
object BitcoinAnchorJNAInterface {

  /**
    * Cached instance of the loaded native library interface.
    *
    * Uses JNA's Native.load() to dynamically load the libbitcoin_anchor_ffi.so
    * library and create the interface proxy.
    */
  lazy val instance: BitcoinAnchorJNAInterface = {
    try {
      Native.load("bitcoin_anchor_ffi", classOf[BitcoinAnchorJNAInterface])
    } catch {
      case e: UnsatisfiedLinkError =>
        throw new RuntimeException(
          s"Failed to load bitcoin_anchor_ffi native library. " +
            s"Make sure libbitcoin_anchor_ffi.so is in the library path. " +
            s"Run ./scripts/build_rust_libraries.sh to build the library.",
          e
        )
    }
  }

  /**
    * Get the JNA interface instance with error handling.
    *
    * @return The loaded BitcoinAnchorJNAInterface instance
    * @throws RuntimeException if the native library cannot be loaded
    */
  def apply(): BitcoinAnchorJNAInterface = instance
}
