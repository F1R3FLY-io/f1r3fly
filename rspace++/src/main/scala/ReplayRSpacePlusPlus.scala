package rspacePlusPlus

import cats.Applicative
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.models.rspace_plus_plus_types.{ActionResult}
import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import cats.syntax.all._
import com.sun.jna.{Native, Pointer}
import java.nio.file.Files

class ReplayRSpacePlusPlus[F[_]: Concurrent: Log, C, P, A, K](rspacePointer: Pointer)
    extends RSpacePlusPlus_RhoTypes[F](rspacePointer)
    with IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation] {

  protected def logF: Log[F] = {
    println("logF")
    ???
  }

  def spawnReplay
      : F[IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation]] =
    for {
      result <- Sync[F].delay {
                 val rspace = INSTANCE.spawn(
                   rspacePointer
                 )

                 new ReplayRSpacePlusPlus[
                   F,
                   Par,
                   BindPattern,
                   ListParWithRandom,
                   TaggedContinuation
                 ](rspace)
               }
    } yield result
}

object ReplayRSpacePlusPlus {
  val INSTANCE: JNAInterface =
    Native
      .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
      .asInstanceOf[JNAInterface]

  def create[F[_]: Concurrent: Log, C, P, A, K](
      storePath: String
  ): F[ReplayRSpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay {
      val rspacePointer = INSTANCE.space_new(storePath);
      new ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer)
    }

}
