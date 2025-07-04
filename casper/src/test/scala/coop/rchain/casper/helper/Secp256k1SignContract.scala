package coop.rchain.casper.helper

import cats.effect.Concurrent
import coop.rchain.crypto.signatures.Secp256k1
import cats.syntax.functor._
import coop.rchain.metrics.Span
import coop.rchain.models.{ListParWithRandom, Par}
import coop.rchain.rholang.interpreter.SystemProcesses
import coop.rchain.rholang.interpreter.{ContractCall, RhoType}

object Secp256k1SignContract {

  def get[F[_]: Concurrent: Span](
      ctx: SystemProcesses.ProcessContext[F]
  )(message: Seq[ListParWithRandom], isReplay: Boolean, previousOutput: Seq[Par]): F[Seq[Par]] = {
    val isContractCall = new ContractCall(ctx.space, ctx.dispatcher)
    (message, isReplay, previousOutput) match {
      case isContractCall(
          produce,
          _,
          _,
          Seq(RhoType.ByteArray(hash), RhoType.ByteArray(sk), ackCh)
          ) =>
        val sig    = Secp256k1.sign(hash, sk)
        val output = Seq(RhoType.ByteArray(sig))
        produce(output, ackCh)
    }
  }
}
