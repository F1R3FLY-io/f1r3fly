package rspacePlusPlus

import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import coop.rchain.rspace.ReportingRspace.{ReportingEvent}

object ReportingRSpacePlusPlus {
  def create[F[_]: Concurrent: Log, C, P, A, K](
      ): F[ReportingRSpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay(new ReportingRSpacePlusPlus[F, C, P, A, K]())
}

class ReportingRSpacePlusPlus[F[_]: Concurrent: Log, C, P, A, K]
    extends ReplayRSpacePlusPlus[F, C, P, A, K]() {

  def getReport: F[Seq[Seq[ReportingEvent]]] =
    Sync[F].delay(Seq.empty[Seq[ReportingEvent]])
}
