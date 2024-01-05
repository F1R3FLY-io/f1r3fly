import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import coop.rchain.shared.Log
import cats.effect.Concurrent
import rspacePlusPlus.{Defaults, RSpacePlusPlus_RhoTypes}
import cats.effect.{ContextShift, IO}
import scala.concurrent.{ExecutionContext}

// See rspace/src/test/scala/coop/rchain/rspace/StorageExamplesTests.scala
class RSpacePlusPlusTest extends AnyFunSuite with Matchers {
  implicit val cs: ContextShift[IO] = IO.contextShift(ExecutionContext.global)
  implicit val log                  = Log.log[IO]

  val _                = System.setProperty("jna.library.path", "./target/release/")
  val space            = new RSpacePlusPlus_RhoTypes[IO]();
  val rhotypesDefaults = new Defaults();

  test(
    "CORE-365: A joined consume on duplicate channels followed by two produces on that channel" +
      "should return a continuation and the produced data"
  ) {
    Log[IO].debug("Hit test").unsafeRunSync()

    val res = space
      .produce(rhotypesDefaults.myPar, rhotypesDefaults.myListParWithRandom, false)
      .unsafeRunSync()

    assert(res.isEmpty)
  }
}
