package rspacePlusPlus

import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import coop.rchain.rspace.ReportingRspace.{ReportingEvent}
import com.sun.jna.{Native, Pointer}
import cats.effect.ContextShift
import scala.concurrent.ExecutionContext
import coop.rchain.shared.Serialize
import coop.rchain.models.Par
import coop.rchain.models.BindPattern
import coop.rchain.models.ListParWithRandom
import coop.rchain.models.TaggedContinuation
import coop.rchain.metrics.Metrics

// import coop.rchain.models.rspace_plus_plus_types.ReportingEventNestedListProto

object ReportingRSpacePlusPlus {
  def create[F[_]: Concurrent: ContextShift: Log: Metrics, C, P, A, K](
      storePath: String
  )(
      implicit
      serializeC: Serialize[Par],
      serializeP: Serialize[BindPattern],
      serializeA: Serialize[ListParWithRandom],
      serializeK: Serialize[TaggedContinuation],
      scheduler: ExecutionContext
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

class ReportingRSpacePlusPlus[F[_]: Concurrent: ContextShift: Log: Metrics, C, P, A, K](
    rspacePointer: Pointer
)(
    implicit
    serializeC: Serialize[Par],
    serializeP: Serialize[BindPattern],
    serializeA: Serialize[ListParWithRandom],
    serializeK: Serialize[TaggedContinuation],
    scheduler: ExecutionContext
) extends ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer) {

  def getReport: F[Seq[Seq[ReportingEvent]]] =
    Sync[F].delay(Seq.empty[Seq[ReportingEvent]])
}
