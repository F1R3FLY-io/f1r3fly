package rspacePlusPlus

import cats.Applicative
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.models.rspace_plus_plus_types.{ActionResult}
import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import cats.syntax.all._
import com.sun.jna.{Native, Pointer}
import java.nio.file.Files
import cats.effect.ContextShift
import scala.concurrent.ExecutionContext
import coop.rchain.metrics.Metrics
import coop.rchain.shared.Serialize
import coop.rchain.rspace.trace.Produce
import scala.collection.SortedSet
import coop.rchain.rspace.trace.Consume

class ReplayRSpacePlusPlus[F[_]: Concurrent: ContextShift: Log: Metrics, C, P, A, K](
    rspacePointer: Pointer
)(
    implicit
    serializeC: Serialize[Par],
    serializeP: Serialize[BindPattern],
    serializeA: Serialize[ListParWithRandom],
    serializeK: Serialize[TaggedContinuation],
    scheduler: ExecutionContext
) extends RSpacePlusPlus_RhoTypes[F](rspacePointer)
    with IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation] {

  protected def logF: Log[F] = Log[F]
  def syncF: Sync[F]         = Sync[F]

  protected[this] override def lockedConsume(
      channels: Seq[C],
      patterns: Seq[P],
      continuation: K,
      persist: Boolean,
      peeks: SortedSet[Int],
      consumeRef: Consume
  ): F[MaybeActionResult] = { println("\nreplay lockedConsume"); ??? }

  protected[this] override def lockedProduce(
      channel: C,
      data: A,
      persist: Boolean,
      produceRef: Produce
  ): F[MaybeActionResult] = { println("\nreplay lockedProduce"); ??? }

  // override def createCheckpoint(): F[Checkpoint] = checkReplayData >> syncF.defer {
  //   for {
  //     changes       <- storeAtom.get().changes
  //     _             = println("\nthis should not be called")
  //     nextHistory   <- historyRepositoryAtom.get().checkpoint(changes.toList)
  //     _             = historyRepositoryAtom.set(nextHistory)
  //     historyReader <- nextHistory.getHistoryReader(nextHistory.root)
  //     _             <- createNewHotStore(historyReader)
  //     _             <- restoreInstalls()
  //   } yield Checkpoint(nextHistory.history.root, Seq.empty)
  // }

  override def clear(): F[Unit] =
    syncF.delay { println("\nhit replay clear"); replayData.clear() } >> super.clear()

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

  def create[F[_]: Concurrent: ContextShift: Log: Metrics, C, P, A, K](
      storePath: String
  )(
      implicit
      serializeC: Serialize[Par],
      serializeP: Serialize[BindPattern],
      serializeA: Serialize[ListParWithRandom],
      serializeK: Serialize[TaggedContinuation],
      scheduler: ExecutionContext
  ): F[ReplayRSpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay {
      val rspacePointer = INSTANCE.space_new(storePath);
      new ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer)
    }

}
