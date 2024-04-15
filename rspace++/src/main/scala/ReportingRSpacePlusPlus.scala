package rspacePlusPlus

import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import coop.rchain.rspace.ReportingRspace.{ReportingEvent}
import com.sun.jna.{Native, Pointer}
import cats.effect.ContextShift
import scala.concurrent.ExecutionContext

object ReportingRSpacePlusPlus {
  def create[F[_]: Concurrent: ContextShift: Log, C, P, A, K](
      storePath: String
  )(
      implicit scheduler: ExecutionContext
  ): F[ReportingRSpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay {
      val INSTANCE: JNAInterface =
        Native
          .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
          .asInstanceOf[JNAInterface]

      val rspacePointer = INSTANCE.space_new(storePath);
      new ReportingRSpacePlusPlus[F, C, P, A, K](rspacePointer)
    }
}

class ReportingRSpacePlusPlus[F[_]: Concurrent: ContextShift: Log, C, P, A, K](
    rspacePointer: Pointer
)(
    implicit scheduler: ExecutionContext
) extends ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer) {

  def getReport: F[Seq[Seq[ReportingEvent]]] =
    Sync[F].delay(Seq.empty[Seq[ReportingEvent]])
}
