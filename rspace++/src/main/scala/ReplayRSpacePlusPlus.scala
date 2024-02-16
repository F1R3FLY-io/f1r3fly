package rspacePlusPlus

import cats.Applicative
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.models.rspace_plus_plus_types.{ActionResult}
import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}

class ReplayRSpacePlusPlus[F[_]: Concurrent: Log, C, P, A, K]
    extends RSpacePlusPlus_RhoTypes[F]()
    with IReplaySpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation] {

  protected def logF: Log[F] = {
    println("logF")
    ???
  }

  def spawnReplay: F[IReplaySpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay(new ReplayRSpacePlusPlus[F, C, P, A, K]())
}
