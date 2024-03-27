package rspacePlusPlus

import cats.Applicative
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.models.rspace_plus_plus_types.{ActionResult}
import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import com.sun.jna.{Native, Pointer}

class ReplayRSpacePlusPlus[F[_]: Concurrent: Log, C, P, A, K](rspacePointer: Pointer)
    extends RSpacePlusPlus_RhoTypes[F](rspacePointer)
    with IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation] {

  protected def logF: Log[F] = {
    println("logF")
    ???
  }

  def spawnReplay
      : F[IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation]] =
    ReplayRSpacePlusPlus
      .create[F, Par, BindPattern, ListParWithRandom, TaggedContinuation]("./rspace++_lmdb")
      .asInstanceOf[F[
        IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation]
      ]]
}

object ReplayRSpacePlusPlus {
  def create[F[_]: Concurrent: Log, C, P, A, K](
      storePath: String
  ): F[ReplayRSpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay {
      val INSTANCE: JNAInterface =
        Native
          .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
          .asInstanceOf[JNAInterface]

      val rspacePointer = INSTANCE.space_new(storePath);
      new ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer)
    }

}
