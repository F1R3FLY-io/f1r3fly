package rspacePlusPlus.state

import cats.effect.Sync
import cats.effect.concurrent.Ref
import cats.syntax.all._
import coop.rchain.catscontrib.Catscontrib._
import coop.rchain.catscontrib.ski._
import coop.rchain.rspace.state.instances.RSpaceExporterStore.NoRootError

object RSpacePlusPlusStateManagerImpl {
  def apply[F[_]: Sync](
      exporter: RSpacePlusPlusExporter[F],
      importer: RSpacePlusPlusImporter[F]
  ): RSpacePlusPlusStateManager[F] =
    RSpacePlusPlusStateManagerImpl[F](exporter, importer)

  private final case class RSpacePlusPlusStateManagerImpl[F[_]: Sync](
      exporter: RSpacePlusPlusExporter[F],
      importer: RSpacePlusPlusImporter[F]
  ) extends RSpacePlusPlusStateManager[F] {

    override def isEmpty: F[Boolean] = hasRoot

    def hasRoot: F[Boolean] =
      exporter.getRoot.map(kp(true)).handleError {
        case NoRootError => false
      }
  }
}
