package coop.rchain.rholang

import com.sun.jna.{Library, Memory, Native, Pointer}

/**
  * The JNA interface for Rholang Rust
  */
trait JNAInterface extends Library {
  def get_hot_changes(runtime_ptr: Pointer): Pointer

  def evaluate(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Pointer

  def create_checkpoint(runtime_ptr: Pointer): Pointer

  def create_soft_checkpoint(runtime_ptr: Pointer): Pointer

  def revert_to_soft_checkpoint(
      runtime_ptr: Pointer,
      payload_pointer: Pointer,
      payload_bytes_len: Int
  ): Unit

  def reset(
      runtime_ptr: Pointer,
      root_pointer: Pointer,
      root_bytes_len: Int
  ): Unit

  def set_block_data(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Unit

  def set_invalid_blocks(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Unit

  def bootstrap_registry(runtime_ptr: Pointer): Pointer

  def create_runtime(rspace_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Pointer

  /* REPLAY */

  def rig(runtime_ptr: Pointer, log_pointer: Pointer, log_bytes_len: Int): Unit

  def check_replay_data(runtime_ptr: Pointer): Unit

  def replay_get_hot_changes(runtime_ptr: Pointer): Pointer

  def replay_evaluate(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Pointer

  def replay_create_checkpoint(runtime_ptr: Pointer): Pointer

  def replay_create_soft_checkpoint(runtime_ptr: Pointer): Pointer

  def replay_revert_to_soft_checkpoint(
      runtime_ptr: Pointer,
      payload_pointer: Pointer,
      payload_bytes_len: Int
  ): Unit

  def replay_reset(
      runtime_ptr: Pointer,
      root_pointer: Pointer,
      root_bytes_len: Int
  ): Unit

  def replay_set_block_data(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Unit

  def replay_set_invalid_blocks(
      runtime_ptr: Pointer,
      params_ptr: Pointer,
      params_bytes_len: Int
  ): Unit

  def replay_bootstrap_registry(runtime_ptr: Pointer): Pointer

  def create_replay_runtime(
      replay_space_ptr: Pointer,
      params_ptr: Pointer,
      params_bytes_len: Int
  ): Pointer
}

object JNAInterfaceLoader {
  val RHOLANG_RUST_INSTANCE: JNAInterface =
    Native
      .load("rholang", classOf[JNAInterface])
}
