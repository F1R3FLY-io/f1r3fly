package coop.rchain.rholang.interpreter.compiler.normalizer.processes

import cats.syntax.all._
import coop.rchain.models.Connective.ConnectiveInstance.{ConnNotBody, ConnOrBody}
import coop.rchain.rholang.interpreter.compiler.{NameVisitOutputs, ProcVisitInputs}
import coop.rchain.rholang.interpreter.errors.{InterpreterError, PatternReceiveError}

object Utils {
  def failOnInvalidConnective(
      input: ProcVisitInputs,
      nameRes: NameVisitOutputs
  ): Either[InterpreterError, NameVisitOutputs] =
    if (input.boundMapChain.depth == 0) {
      Either
        .fromOption(
          nameRes.freeMap.connectives
            .collectFirst {
              case (_: ConnOrBody, sourcePosition) =>
                PatternReceiveError(s"\\/ (disjunction) at $sourcePosition")
              case (_: ConnNotBody, sourcePosition) =>
                PatternReceiveError(s"~ (negation) at $sourcePosition")
            },
          nameRes
        )
        .swap
    } else Right(nameRes)
}
