package coop.rchain.rholang

import com.sun.jna.{Library, Memory, Native, Pointer}

/**
  * The JNA interface for Rholang Rust
  */
trait JNAInterface extends Library {
  def get_hot_changes(runtime_ptr: Pointer): Pointer

  def evaluate(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Pointer

  def create_checkpoint(runtime_ptr: Pointer): Pointer

  def reset(
      runtime_ptr: Pointer,
      root_pointer: Pointer,
      root_bytes_len: Int
  ): Unit

  def set_block_data(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Unit

  def bootstrap_registry(runtime_ptr: Pointer): Pointer

  def create_runtime(rspace_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Pointer
}

object JNAInterfaceLoader {
  val RHOLANG_RUST_INSTANCE: JNAInterface =
    Native
      .load("rholang", classOf[JNAInterface])
}
