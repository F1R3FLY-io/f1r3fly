package rspacePlusPlus

import coop.rchain.rspace.HotStoreAction
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.history.History
import coop.rchain.rspace.history.HistoryReader
import coop.rchain.shared.Serialize
import coop.rchain.rspace.HotStoreTrieAction
import rspacePlusPlus.state.RSpacePlusPlusExporter
import rspacePlusPlus.state.RSpacePlusPlusImporter
import rspacePlusPlus.history.RSpacePlusPlusHistoryReader

trait HistoryRepository[F[_], C, P, A, K] {
  def checkpoint(actions: List[HotStoreAction]): F[HistoryRepository[F, C, P, A, K]]

  def doCheckpoint(actions: Seq[HotStoreTrieAction]): F[HistoryRepository[F, C, P, A, K]]

  def reset(root: Blake2b256Hash): F[HistoryRepository[F, C, P, A, K]]

  def history: History[F]

  def exporter: F[RSpacePlusPlusExporter[F]]

  def importer: F[RSpacePlusPlusImporter[F]]

  def getHistoryReader(
      stateHash: Blake2b256Hash
  ): F[RSpacePlusPlusHistoryReader[F, Blake2b256Hash, C, P, A, K]]

  def getSerializeC: Serialize[C]

  def root: Blake2b256Hash
}
