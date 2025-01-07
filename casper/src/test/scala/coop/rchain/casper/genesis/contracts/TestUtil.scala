package coop.rchain.casper.genesis.contracts

import cats.FlatMap
import cats.effect.Sync
import cats.syntax.all._
import coop.rchain.crypto.hash.Blake2b512Random
import coop.rchain.models.Par
import coop.rchain.rholang.build.CompiledRholangSource
import coop.rchain.rholang.interpreter.accounting.Cost
import coop.rchain.rholang.interpreter.RhoRuntime
import coop.rchain.rholang.interpreter.compiler.Compiler
import coop.rchain.models.rholang_scala_rust_types.SourceToAdtParams

object TestUtil {
  def eval[F[_]: Sync, Env](
      source: CompiledRholangSource[Env],
      runtime: RhoRuntime[F]
  )(implicit rand: Blake2b512Random): F[Unit] = eval(source.code, runtime, source.env)

  def eval[F[_]: Sync](
      code: String,
      runtime: RhoRuntime[F],
      normalizerEnv: Map[String, Par]
  )(implicit rand: Blake2b512Random): F[Unit] = {
    val params = SourceToAdtParams(code, normalizerEnv)
    // println(s"\nhit eval, normalizerEnv: $normalizerEnv")
    val par = CompiledRholangSource.sourceToAdt(params)
    // println(s"\nhit eval, normalized par: $par")
    evalTerm(par, runtime)

    // for {
    //   term <- Compiler[F].sourceToADT(code, normalizerEnv)
    //   // _    = println(s"\nhit eval, normalizerEnv: $normalizerEnv")
    //   // _ = println(s"\nhit eval, normalized par: $term")
    //   _ <- evalTerm(term, runtime)
    // } yield ()

  }

  private def evalTerm[F[_]: FlatMap](
      term: Par,
      runtime: RhoRuntime[F]
  )(implicit rand: Blake2b512Random): F[Unit] =
    for {
      _ <- runtime.setCostToMax
      _ <- runtime.inj(term)
    } yield ()
}
