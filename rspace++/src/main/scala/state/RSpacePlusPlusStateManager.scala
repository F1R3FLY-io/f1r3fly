package rspacePlusPlus.state

import coop.rchain.state.StateManager

trait RSpacePlusPlusStateManager[F[_]] extends StateManager[F] {
  def exporter: RSpacePlusPlusExporter[F]
  def importer: RSpacePlusPlusImporter[F]
}

object RSpacePlusPlusStateManager {
  def apply[F[_]](implicit instance: RSpacePlusPlusStateManager[F]): RSpacePlusPlusStateManager[F] =
    instance

}
