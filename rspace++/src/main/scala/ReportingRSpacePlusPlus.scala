package rspacePlusPlus

import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import coop.rchain.rspace.ReportingRspace.{ReportingEvent}
import com.sun.jna.{Native, Pointer}

object ReportingRSpacePlusPlus {
  def create[F[_]: Concurrent: Log, C, P, A, K](
      storePath: String
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

class ReportingRSpacePlusPlus[F[_]: Concurrent: Log, C, P, A, K](rspacePointer: Pointer)
    extends ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer) {

  def getReport: F[Seq[Seq[ReportingEvent]]] =
    Sync[F].delay(Seq.empty[Seq[ReportingEvent]])
}
