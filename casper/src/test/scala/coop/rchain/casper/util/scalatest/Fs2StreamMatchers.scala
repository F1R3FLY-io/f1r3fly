package coop.rchain.casper.util.scalatest

import fs2.Stream
import monix.eval.Task
import org.scalatest.matchers.{MatchResult, Matcher}

import java.util.concurrent.TimeoutException
import scala.concurrent.duration.{DurationInt, FiniteDuration}

trait Fs2StreamMatchers {

  /**
    * Checks if Stream will not produce any more elements == stream is empty
    *
    * @param timeout duration to wait for new elements
    */
  class EmptyMatcher[A](timeout: FiniteDuration) extends Matcher[Stream[Task, A]] {
    import monix.execution.Scheduler.Implicits.global

    def apply(left: Stream[Task, A]) = {
      val res = left.take(1).timeout(timeout).compile.toList.attempt.runSyncUnsafe()

      val isEmpty = res.isLeft && res.left.get.isInstanceOf[TimeoutException]

      val onFail    = if (!isEmpty) s"Stream is not empty, emitted: ${res.right.get}" else ""
      val onSuccess = s"Stream is empty"

      MatchResult(isEmpty, onFail, onSuccess)
    }
  }

  def notEmit = new EmptyMatcher[Any](250.millis)

  def notEmit(timeout: FiniteDuration) = new EmptyMatcher[Any](timeout)
}
