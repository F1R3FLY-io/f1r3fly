package coop.rchain.rholang.interpreter

import coop.rchain.metrics
import coop.rchain.metrics.{Metrics, NoopSpan, Span}
import coop.rchain.rholang.Resources.mkRuntime
import coop.rchain.rholang.interpreter.errors.UserAbortError
import coop.rchain.rholang.syntax._
import coop.rchain.shared.Log
import monix.eval.Task
import monix.execution.Scheduler.Implicits.global
import org.scalatest.{FlatSpec, Matchers}

import scala.concurrent.duration._

class AbortSpec extends FlatSpec with Matchers {
  private val tmpPrefix                   = "rspace-store-"
  private val maxDuration                 = 5.seconds
  implicit val logF: Log[Task]            = Log.log[Task]
  implicit val noopMetrics: Metrics[Task] = new metrics.Metrics.MetricsNOP[Task]
  implicit val noopSpan: Span[Task]       = NoopSpan[Task]()

  "rho:execution:abort" should "execute successfully when called with arguments" in {
    val rhoCode =
      """
        |new abort(`rho:execution:abort`) in {
        |  abort!("Test abort")
        |}
        |""".stripMargin

    val result = execute(rhoCode)
    // Abort should execute successfully (no errors) and terminate execution
    result.errors should be(empty)
    result.cost.value should be(316L +- 100L)
    result.succeeded should be(true)
  }

  it should "allow parallel processes to execute independently" in {
    val rhoCode =
      """
        |new result1, abort(`rho:execution:abort`) in {
        |  result1!("Process 1 executed") |
        |  abort!("Process 2 aborted")
        |}
        |""".stripMargin

    val result = execute(rhoCode)
    result.cost.value should be(600L +- 100L) // ~300 is a cost of 1 Par, so 600 is 2 Par dispatched
    result.errors should be(empty)
  }

  private def execute(source: String): EvaluateResult =
    mkRuntime[Task](tmpPrefix)
      .use { runtime =>
        runtime.evaluate(source)
      }
      .runSyncUnsafe(maxDuration)
}
