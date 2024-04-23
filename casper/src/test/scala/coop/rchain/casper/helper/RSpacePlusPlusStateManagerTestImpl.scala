package coop.rchain.casper.helper

import cats.syntax.all._
import cats.effect.Sync
import rspacePlusPlus.state.{
  RSpacePlusPlusExporter,
  RSpacePlusPlusImporter,
  RSpacePlusPlusStateManager
}

final case class RSpacePlusPlusStateManagerTestImpl[F[_]: Sync]()
    extends RSpacePlusPlusStateManager[F] {
  override def exporter: RSpacePlusPlusExporter[F] = ???

  override def importer: RSpacePlusPlusImporter[F] = ???

  override def isEmpty: F[Boolean] = ???
}
