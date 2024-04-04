import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import coop.rchain.shared.Log
import cats.effect.Concurrent
import rspacePlusPlus.{Defaults, RSpacePlusPlus_RhoTypes}
import cats.effect.{ContextShift, IO}
import scala.concurrent.{ExecutionContext}
import org.scalatest.{BeforeAndAfterEach}
import java.nio.file.{Files, Paths}
import java.util.Comparator
import rspacePlusPlus.JNAInterface
import com.sun.jna.{Native}

class ScalaToRustTest extends AnyFunSuite with Matchers with BeforeAndAfterEach {
  implicit val cs: ContextShift[IO] = IO.contextShift(ExecutionContext.global)
  implicit val log                  = Log.log[IO]

  val testDirectoryPath                  = "./src/test/scala-to-rust_lmdb"
  var space: RSpacePlusPlus_RhoTypes[IO] = _

  val INSTANCE: JNAInterface =
    Native
      .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
      .asInstanceOf[JNAInterface]

  test("produce should return null pointer (none)") {}

  override def beforeEach(): Unit = {
    val _ = System.setProperty("jna.library.path", "./target/release/")
    space = RSpacePlusPlus_RhoTypes.create[IO](testDirectoryPath).unsafeRunSync()
  }

  override def afterEach(): Unit = {
    space.clear().unsafeRunSync()
    deleteRecursively(testDirectoryPath)
  }

  private def deleteRecursively(path: String): Unit = {
    val directory = Paths.get(path)
    if (Files.exists(directory)) {
      Files
        .walk(directory)
        .sorted(Comparator.reverseOrder())
        .forEach(Files.delete)
    }
  }
}
