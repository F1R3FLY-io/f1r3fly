package coop.rchain.rholang

import com.sun.jna.{Library, Memory, Native, Pointer}

/**
  * The JNA interface for Rholang Rust
  */
trait JNAInterface extends Library {
  def set_block_data(runtime_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Unit

  def create_runtime(rspace_ptr: Pointer, params_ptr: Pointer, params_bytes_len: Int): Pointer
}

object JNAInterfaceLoader {
  val RHOLANG_RUST_INSTANCE: JNAInterface =
    Native
      .load("rholang", classOf[JNAInterface])
}
