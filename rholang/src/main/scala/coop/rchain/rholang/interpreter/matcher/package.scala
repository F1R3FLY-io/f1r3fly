package coop.rchain.rholang.interpreter

import cats.data.StateT
import cats.effect.Sync
import cats.mtl._
import cats.syntax.all._
import cats.{~>, Monad}
import coop.rchain.catscontrib.MonadError_
import coop.rchain.models.Par
import coop.rchain.rholang.interpreter.accounting._

import scala.collection.immutable.Stream

package object matcher {

  type FreeMap = Map[Int, Par]

  type MatcherMonadT[F[_], A] = StateT[StreamT[F, *], FreeMap, A]

  import coop.rchain.rholang.interpreter.matcher.StreamT

  implicit def matcherMonadCostLog[F[_]: Sync: _cost](): _cost[MatcherMonadT[F, *]] =
    λ[F ~> MatcherMonadT[F, *]](fa => StateT.liftF(StreamT.liftF(fa)))

  // The naming convention means: this is an effect-type alias.
  // Will be used similarly to capabilities, but for more generic and probably low-level/implementation stuff.
  // Adopted from: http://atnos-org.github.io/eff/org.atnos.site.Tutorial.html#write-an-interpreter-for-your-program
  type _freeMap[F[_]] = MonadState[F, FreeMap]
  type _short[F[_]]   = MonadError_[F, Unit] //arises from and corresponds to the OptionT/StreamT in the stack

  // Implicit summoner methods, just like `Monad.apply` on `Monad`'s companion object.
  def _freeMap[F[_]](implicit ev: _freeMap[F]): _freeMap[F] = ev
  def _short[F[_]](implicit ev: _short[F]): _short[F]       = ev

  private[matcher] def run[F[_]: Monad, A](
      f: MatcherMonadT[F, A]
  ): F[Stream[(FreeMap, A)]] =
    StreamT.run(f.run(emptyMap))

  /*

The `runFirst` function performs the following steps:

1. It starts with a `MatcherMonadT[F, A]`, which represents a computation that produces a stream of `(FreeMap, A)` pairs within the monad `F`.
2. It calls `f.run(emptyMap)`, which runs the computation with an initial empty state (`emptyMap`).
3. `StreamT.dropTail` is applied to the result, which removes the tail of the stream, effectively turning it into a stream that produces at most one element.
4. `StreamT.run` is then called on the result of `dropTail`, which flattens the stream out of the `StreamT` transformer, yielding an `F[Stream[(FreeMap, A)]]`.
5. Finally, it maps over the resulting stream and takes the `headOption`, which extracts the first element of the stream if it exists, resulting in an `F[Option[(FreeMap, A)]]`.

The `runFirst` function is used to run the computation and get the first result of the stream, along with the state associated with that result, all within the context of the monad `F`.

   */
  private[rholang] def runFirst[F[_]: Monad, A](
      f: MatcherMonadT[F, A]
  ): F[Option[(FreeMap, A)]] =
    StreamT
      .run(StreamT.dropTail(f.run(emptyMap)))
      .map(_.headOption)

  private[matcher] def attemptOpt[F[_], A](
      f: F[A]
  )(implicit short: _short[F]): F[Option[A]] = {
    import short.instance
    f.attempt.map(_.fold(_ => None, Some(_)))
  }

  def emptyMap: FreeMap = Map.empty[Int, Par]

}
