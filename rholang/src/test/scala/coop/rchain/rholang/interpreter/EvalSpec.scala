package coop.rchain.rholang.interpreter

import coop.rchain.metrics
import coop.rchain.metrics.{Metrics, NoopSpan, Span}
import coop.rchain.rholang.Resources.mkRuntime
import coop.rchain.rholang.syntax._
import coop.rchain.shared.Log
import monix.eval.Task
import monix.execution.Scheduler.Implicits.global
import org.scalatest.{FlatSpec, Matchers}

import scala.concurrent.duration._
import scala.io.Source

class EvalSpec extends FlatSpec with Matchers {
  private val tmpPrefix                   = "rspace-store-"
  private val maxDuration                 = 5.seconds
  implicit val logF: Log[Task]            = Log.log[Task]
  implicit val noopMetrics: Metrics[Task] = new metrics.Metrics.MetricsNOP[Task]
  implicit val noopSpan: Span[Task]       = NoopSpan[Task]()

  "EvalTest" should "work" in {
    val rhoScript = loadScript("examples/EvalTest.rho")
    execute(rhoScript)
  }

  private def execute(source: String): EvaluateResult =
    mkRuntime[Task](tmpPrefix)
      .use { runtime =>
        runtime.evaluate(source)
      }
      .runSyncUnsafe(maxDuration)

  private def loadScript(filePath: String): String = {
    val source = Source.fromFile(filePath)
    try {
      source.getLines().mkString("\n")
    } finally {
      source.close()
    }
  }
}
