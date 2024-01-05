package coop.rchain.rspace.history

import cats.Parallel
import cats.effect.{Concurrent, Sync}
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.history.History._
import coop.rchain.rspace.history.instances.RadixHistory
import coop.rchain.store.KeyValueStore
import scodec.bits.ByteVector

/**
  * Helper trait to be able to return self type for _process_ and _reset_ methods.
  *
  * Used to extend History with old _find_ method in [[HistoryWithFind]].
  *
  * TODO: Delete when old ("merging") [[HistoryMergingInstances.MergingHistory]] is removed.
  */
trait HistorySelf[F[_]] {
  type HistoryF <: HistorySelf[F]
}

/**
  * History definition represents key-value API for RSpace tuple space
  *
  * [[History]] contains only references to data stored on keys ([[KeyPath]]).
  *
  * [[ColdStoreInstances.ColdKeyValueStore]] holds full data referenced by [[LeafPointer]] in [[History]].
  */
trait History[F[_]] extends HistorySelf[F] {
  override type HistoryF <: History[F]

  /**
    * Read operation on the Merkle tree
    */
  def read(key: ByteVector): F[Option[ByteVector]]

  /**
    * Insert/update/delete operations on the underlying Merkle tree (key-value store)
    */
  def process(actions: List[HistoryAction]): F[HistoryF]

  /**
    * Get the root of the Merkle tree
    */
  def root: Blake2b256Hash

  /**
    * Returns History with specified root pointer
    */
  def reset(root: Blake2b256Hash): F[HistoryF]
}

/**
  * Support for old ("merging") History
  *
  * TODO: Delete when old ("merging") [[HistoryMergingInstances.MergingHistory]] is removed.
  */
trait HistoryWithFind[F[_]] extends History[F] {
  override type HistoryF <: HistoryWithFind[F]

  /**
    * Read operation on the Merkle tree ("merging" History)
    */
  def find(key: KeyPath): F[(TriePointer, Vector[Trie])]
}

object History {
  val emptyRootHash: Blake2b256Hash = RadixHistory.emptyRootHash
//  val emptyRootHash: Blake2b256Hash = HistoryMergingInstances.emptyRootHash //for MergingHistory

  def create[F[_]: Concurrent: Sync: Parallel](
      root: Blake2b256Hash,
      store: KeyValueStore[F]
  ): F[RadixHistory[F]] = RadixHistory(root, RadixHistory.createStore(store))
//  ): F[HistoryWithFind[F]] =
//    Sync[F].delay(
//      HistoryMergingInstances.merging(root, HistoryStoreInstances.historyStore[F](store))
//    ) //for MergingHistory

  type KeyPath = Seq[Byte]
}
