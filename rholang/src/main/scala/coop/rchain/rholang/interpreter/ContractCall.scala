package coop.rchain.rholang.interpreter

import cats.effect.{Concurrent, Sync}
import cats.syntax.all._
import coop.rchain.crypto.hash.Blake2b512Random
import coop.rchain.metrics.Span
import coop.rchain.models.{ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.rholang.interpreter.RhoRuntime.RhoTuplespace

/**
  * This is a tool for unapplying the messages sent to the system contracts.
  *
  * The unapply returns (Producer, Seq[Par]).
  *
  * The Producer is the function with the signature (Seq[Par], Par) => F[Unit] which can be used to send a message
  * through a channel. The first argument with type Seq[Par] is the content of the message and the second argument is
  * the channel.
  *
  * Note that the random generator and the sequence number extracted from the incoming message are required for sending
  * messages back to the caller so they are given as the first argument list to the produce function.
  *
  * The Seq[Par] returned by unapply contains the message content and can be further unapplied as needed to match the
  * required signature.
  *
  * @param space the rspace instance
  * @param dispatcher the dispatcher
  */
class ContractCall[F[_]: Concurrent: Span](
    space: RhoTuplespace[F],
    dispatcher: Dispatch[F, ListParWithRandom, TaggedContinuation]
) {
  type Producer[M[_]] = (Seq[Par], Par) => M[Seq[Par]]

  // TODO: pass _cost[F] as an implicit parameter
  private def produce(
      rand: Blake2b512Random
  )(values: Seq[Par], ch: Par): F[Seq[Par]] =
    for {
      produceResult <- space.produce(
                        ch,
                        ListParWithRandom(values, rand),
                        persist = false
                      )
      dispatchResult <- produceResult.fold(Sync[F].delay[Dispatch.DispatchType](Dispatch.Skip)) {
                         case (cont, channels, produce) =>
                           dispatcher.dispatch(
                             cont.continuation,
                             channels.map(_.matchedDatum),
                             space.isReplay,
                             produce.outputValue.map(raw => Par.parseFrom(raw))
                           )
                       }
    } yield dispatchResult match {
      case Dispatch.DeterministicCall            => Seq.empty
      case Dispatch.NonDeterministicCall(output) => output.map(Par.parseFrom)
      case Dispatch.Skip                         => Seq.empty
    }

  def unapply(
      contractArgs: (Seq[ListParWithRandom], Boolean, Seq[Par])
  ): Option[(Producer[F], Boolean, Seq[Par], Seq[Par])] =
    contractArgs match {
      case (
          Seq(
            ListParWithRandom(
              args,
              rand
            )
          ),
          isReplay,
          previous
          ) =>
        Some((produce(rand), isReplay, previous, args))
      case _ => None
    }
}
