package rspacePlusPlus

import com.sun.jna._
import java.nio.charset.StandardCharsets
import java.nio.ByteBuffer

final case class _Name(first: String, last: String)
final case class _Address(street: String, city: String, state: String, zip: String)
final case class _Entry(name: _Name, address: _Address, email: String, phone: String)
final case class Pres_Class(continuation: String, data: _Entry)

final case class Result[C, A](
    channel: C,
    matchedDatum: A,
    removedDatum: A,
    persistent: Boolean
)
final case class ContResult[C, P, K](
    continuation: K,
    persistent: Boolean,
    channels: Seq[C],
    patterns: Seq[P],
    peek: Boolean = false
)

// Problem here is JNA can't find decalared fields "x" and "y" bc uses reflection
// Adding "MyStruct" as return type to any of 16 functions will throw error

// @SuppressWarnings(Array("org.wartremover.warts.Var"))
// @Structure.FieldOrder(Array("x", "y"))
// class MyStruct extends Structure {
//   @scala.native
//   val x: Int = 0
//   @scala.native
//   val y: Int = 0
// }

/** The interface for RSpace++
  *
  */
trait RSpacePlusPlus[F[_]] extends Library {
  def space_new(): Pointer
  def is_empty(rspace: Pointer): Boolean
  def space_print(rspace: Pointer, channel: String): Unit
  def space_clear(rspace: Pointer): Unit

  // Verb Set 1
  def space_get_once_durable_concurrent(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  def space_get_once_non_durable_concurrent(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  def space_get_once_durable_sequential(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  def space_get_once_non_durable_sequential(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  // Verb Set 2
  def space_get_always_durable_concurrent(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  def space_get_always_non_durable_concurrent(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  def space_get_always_durable_sequential(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  def space_get_always_non_durable_sequential(
      rspace: Pointer,
      retrieve: Array[Byte],
      retrieve_len: Int
  ): String

  // Verb Set 3
  def space_put_once_durable_concurrent(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  def space_put_once_non_durable_concurrent(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  def space_put_once_durable_sequential(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  def space_put_once_non_durable_sequential(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  // Verb Set 4
  def space_put_always_durable_concurrent(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  def space_put_always_non_durable_concurrent(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  def space_put_always_durable_sequential(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]

  def space_put_always_non_durable_sequential(
      rspace: Pointer,
      commit: Array[Byte],
      commit_len: Int
  ): Array[String]
}
