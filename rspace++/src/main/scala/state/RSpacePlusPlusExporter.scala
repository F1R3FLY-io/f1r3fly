package rspacePlusPlus.state

import cats.effect.Sync
import cats.syntax.all._
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.state.{TrieExporter, TrieNode}
import scodec.bits.ByteVector
import coop.rchain.shared.Log
import java.nio.ByteBuffer
import java.nio.file.Path
import cats.effect.Concurrent
import coop.rchain.rspace.state.exporters.RSpaceExporterItems.StoreItems
import com.google.protobuf.ByteString

trait RSpacePlusPlusExporter[F[_]] extends TrieExporter[F] {
  type KeyHash = Blake2b256Hash

  // Get current root
  def getRoot: F[KeyHash]

  def traverseHistory(
      startPath: Seq[(Blake2b256Hash, Option[Byte])],
      skip: Int,
      take: Int,
      getFromHistory: ByteVector => F[Option[ByteVector]]
  ): F[Vector[TrieNode[Blake2b256Hash]]]

  def getHistory[Value](
      startPath: Seq[(Blake2b256Hash, Option[Byte])],
      skip: Int,
      take: Int,
      fromBuffer: ByteBuffer => Value
  )(implicit m: Sync[F], l: Log[F]): F[StoreItems[Blake2b256Hash, Value]]

  def getData[Value](
      startPath: Seq[(Blake2b256Hash, Option[Byte])],
      skip: Int,
      take: Int,
      fromBuffer: ByteBuffer => Value
  )(implicit m: Sync[F], l: Log[F]): F[StoreItems[Blake2b256Hash, Value]]

  def getHistoryAndData(
      startPath: Seq[(Blake2b256Hash, Option[Byte])],
      skip: Int,
      take: Int
      // fromBuffer: ByteBuffer => Value
  )(
      implicit m: Sync[F],
      l: Log[F]
  ): F[(StoreItems[Blake2b256Hash, ByteString], StoreItems[Blake2b256Hash, ByteString])]

  // Export to disk

  def writeToDisk[C, P, A, K](root: Blake2b256Hash, dirPath: Path, chunkSize: Int)(
      implicit m: Concurrent[F],
      l: Log[F]
  ): F[Unit]
}

object RSpacePlusPlusExporter {

  final case class Counter(skip: Int, take: Int)

  final case object EmptyHistoryException extends Exception

  // Pretty printer helpers
  def pathPretty(path: (Blake2b256Hash, Option[Byte])): String = {
    val (hash, idx) = path
    val idxStr      = idx.fold("--")(i => String.format("%02x", Integer.valueOf(i & 0xff)))
    s"$idxStr:${hash.bytes.toHex.take(8)}"
  }
}
