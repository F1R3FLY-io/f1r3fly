package rspacePlusPlus

import com.sun.jna.{Library, Pointer}
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}

/**
  * The JNA interface for Rust RSpace++
  */
trait JNAInterface extends Library {
  def space_new(): Pointer
  def space_print(rspace: Pointer): Unit
  def space_clear(rspace: Pointer): Unit

  def spatial_match_result(
      payload_pointer: Pointer,
      target_bytes_len: Int,
      pattern_bytes_len: Int
  ): Pointer

  def produce(
      rspace: Pointer,
      payload_pointer: Pointer,
      channel_bytes_len: Int,
      data_bytes_len: Int,
      persist: Boolean
  ): Pointer

  def consume(
      rspace: Pointer,
      payload_pointer: Pointer,
      payload_bytes_len: Int
  ): Pointer

  def install(
      rspace: Pointer,
      payload_pointer: Pointer,
      payload_bytes_len: Int
  ): Pointer

  def create_checkpoint(rspace: Pointer): Pointer

  def reset(
      rspace: Pointer,
      root_pointer: Pointer,
      root_bytes_len: Int
  ): Unit

  def get_data(
      rspace: Pointer,
      channel_pointer: Pointer,
      channel_bytes_len: Int
  ): Pointer

  def get_waiting_continuations(
      rspace: Pointer,
      channels_pointer: Pointer,
      channels_bytes_len: Int
  ): Pointer

  def get_joins(
      rspace: Pointer,
      channel_pointer: Pointer,
      channel_bytes_len: Int
  ): Pointer

  def to_map(rspace: Pointer): Pointer

  def spawn(rspace: Pointer): Pointer

  def deallocate_memory(ptr: Pointer, len: Int): Unit
}
