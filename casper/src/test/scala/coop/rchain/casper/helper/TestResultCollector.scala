package coop.rchain.casper.helper
import cats.effect.Concurrent
import cats.effect.concurrent.Ref
import cats.syntax.all._
import coop.rchain.crypto.hash.Blake2b512Random
import coop.rchain.metrics.Span
import coop.rchain.models.Expr.ExprInstance.{ETupleBody, GBool}
import coop.rchain.models.rholang.implicits._
import coop.rchain.models.{ETuple, Expr, ListParWithRandom, Par}
import coop.rchain.rholang.interpreter.SystemProcesses.ProcessContext
import coop.rchain.rholang.interpreter.{ContractCall, RhoType}

object IsAssert {
  def unapply(
      p: Seq[Par]
  ): Option[(String, Long, Par, String, Par)] =
    p match {
      case Seq(
          RhoType.String(testName),
          RhoType.Number(attempt),
          assertion,
          RhoType.String(clue),
          ackChannel
          ) =>
        Some((testName, attempt, assertion, clue, ackChannel))
      case _ => None
    }
}

object IsComparison {
  def unapply(
      p: Par
  ): Option[(Par, String, Par)] =
    p.singleExpr().collect {
      case Expr(ETupleBody(ETuple(List(expected, RhoType.String(operator), actual), _, _))) =>
        (expected, operator, actual)
    }
}
object IsSetFinished {
  def unapply(p: Seq[Par]): Option[Boolean] =
    p match {
      case Seq(RhoType.Boolean(hasFinished)) =>
        Some(hasFinished)
      case _ => None
    }
}

sealed trait RhoTestAssertion {
  val testName: String
  val clue: String
  val isSuccess: Boolean
}

case class RhoAssertTrue(testName: String, override val isSuccess: Boolean, clue: String)
    extends RhoTestAssertion
case class RhoAssertEquals(testName: String, expected: Par, actual: Par, clue: String)
    extends RhoTestAssertion {
  override val isSuccess: Boolean = actual == expected
}
case class RhoAssertNotEquals(testName: String, unexpected: Par, actual: Par, clue: String)
    extends RhoTestAssertion {
  override val isSuccess: Boolean = actual != unexpected
}

case class TestResult(
    assertions: Map[String, Map[Long, List[RhoTestAssertion]]],
    hasFinished: Boolean
) {
  def addAssertion(attempt: Long, assertion: RhoTestAssertion): TestResult = {
    val currentAttemptAssertions = assertions.getOrElse(assertion.testName, Map.empty)
    val newAssertion =
      (attempt, assertion :: currentAttemptAssertions.getOrElse(attempt, List.empty))
    val newCurrentAttemptAssertions = currentAttemptAssertions + newAssertion
    TestResult(assertions.updated(assertion.testName, newCurrentAttemptAssertions), hasFinished)
  }
  def setFinished(hasFinished: Boolean): TestResult =
    TestResult(assertions, hasFinished = hasFinished)
}

case class AckedActionCtx(ackChannel: Par, rand: Blake2b512Random, sequenceNumber: Long)

object TestResultCollector {
  def apply[F[_]: Concurrent: Span]: F[TestResultCollector[F]] =
    Ref
      .of(TestResult(Map.empty, hasFinished = false))
      .map(new TestResultCollector(_))
}

class TestResultCollector[F[_]: Concurrent: Span](result: Ref[F, TestResult]) {

  def getResult: F[TestResult] = result.get

  def handleMessage(
      ctx: ProcessContext[F]
  )(message: Seq[ListParWithRandom], isReplay: Boolean, previousOutput: Seq[Par]): F[Seq[Par]] = {

    val isContractCall = new ContractCall[F](ctx.space, ctx.dispatcher)

    (message, isReplay, previousOutput) match {
      case isContractCall(
          produce,
          _,
          _,
          IsAssert(testName, attempt, assertion, clue, ackChannel)
          ) =>
        assertion match {
          case IsComparison(expected, "==", actual) =>
            val assertion = RhoAssertEquals(testName, expected, actual, clue)
            for {
              _      <- result.update(_.addAssertion(attempt, assertion))
              output = Seq(GBool(assertion.isSuccess): Par)
              _      <- produce(output, ackChannel)
            } yield output
          case IsComparison(unexpected, "!=", actual) =>
            val assertion = RhoAssertNotEquals(testName, unexpected, actual, clue)
            for {
              _      <- result.update(_.addAssertion(attempt, assertion))
              output = Seq(GBool(assertion.isSuccess): Par)
              _      <- produce(output, ackChannel)
            } yield output
          case RhoType.Boolean(condition) =>
            for {
              _      <- result.update(_.addAssertion(attempt, RhoAssertTrue(testName, condition, clue)))
              output = Seq(GBool(condition): Par)
              _      <- produce(output, ackChannel)
            } yield output

          case _ =>
            for {
              _ <- result.update(
                    _.addAssertion(
                      attempt,
                      RhoAssertTrue(
                        testName,
                        isSuccess = false,
                        s"Failed to evaluate assertion $assertion"
                      )
                    )
                  )
              output = Seq(GBool(false): Par)
              _      <- produce(output, ackChannel)
            } yield output
        }
      case isContractCall(_, _, _, IsSetFinished(hasFinished)) =>
        result.update(_.setFinished(hasFinished)).map(_ => Seq.empty[Par])
    }
  }
}
