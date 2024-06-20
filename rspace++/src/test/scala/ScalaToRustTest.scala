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
import com.sun.jna.{Memory, Native, Pointer}
import com.google.protobuf.ByteString
import coop.rchain.models._
import coop.rchain.models.Var.VarInstance.FreeVar
import coop.rchain.models.TaggedContinuation.TaggedCont.ParBody
import coop.rchain.models.Expr.ExprInstance.GInt
import coop.rchain.models.rholang.implicits._
import coop.rchain.crypto.hash.Blake2b512Random
import cats.instances.boolean
import coop.rchain.models.rspace_plus_plus_types.ConsumeParams
import coop.rchain.models.rspace_plus_plus_types.ActionResult
import rspacePlusPlus.JNAInterfaceLoader.INSTANCE

class ScalaToRustTest extends AnyFunSuite with Matchers with BeforeAndAfterEach {
  implicit val cs: ContextShift[IO] = IO.contextShift(ExecutionContext.global)
  implicit val log                  = Log.log[IO]

  val _                 = System.setProperty("jna.library.path", "./target/release/")
  val testDirectoryPath = "./src/test/scala/scala-to-rust_lmdb"
  var spacePtr: Pointer = _

  val channels = channelsN(1)
  val patterns = patternsN(1)
  val cont     = continuation()
  val channel  = channels.head
  val data     = ListParWithRandom().withPars(Seq(Par()))

  test("single produce should return null pointer (none)") {
    val result = doProduce(spacePtr, channel, data, false)
    result shouldBe null
  }

  test("single consume should return null pointer (none)") {
    val result = doConsume(spacePtr, channels, patterns, cont, false)
    result shouldBe null
  }

  test("produce then consume should parse correct proto message") {
    val produceRes = doProduce(spacePtr, channel, data, false)
    produceRes shouldBe null

    val consumeRes            = doConsume(spacePtr, channels, patterns, cont, false)
    val consumeResByteslength = consumeRes.getInt(0)
    val consumeResBytes       = consumeRes.getByteArray(4, consumeResByteslength)

    noException should be thrownBy ActionResult.parseFrom(consumeResBytes)
  }

  def doProduce(
      rspacePointer: Pointer,
      channel: Par,
      data: ListParWithRandom,
      persist: Boolean
  ): Pointer = {
    val channelBytes       = channel.toByteArray
    val channelBytesLength = channelBytes.length
    val dataBytes          = data.toByteArray
    val dataBytesLength    = dataBytes.length

    val payloadSize   = channelBytesLength.toLong + dataBytesLength.toLong
    val payloadMemory = new Memory(payloadSize)

    payloadMemory.write(0, channelBytes, 0, channelBytesLength)
    payloadMemory.write(channelBytesLength.toLong, dataBytes, 0, dataBytesLength)

    INSTANCE.produce(
      rspacePointer,
      payloadMemory,
      channelBytesLength,
      dataBytesLength,
      persist
    )
  }

  def doConsume(
      rspacePointer: Pointer,
      channels: Seq[Par],
      patterns: Seq[BindPattern],
      continuation: TaggedContinuation,
      persist: Boolean
  ): Pointer = {
    val consumeParams = ConsumeParams(
      channels,
      patterns,
      Some(continuation),
      persist
    )
    val consumeParamsBytes = consumeParams.toByteArray

    val payloadMemory = new Memory(consumeParamsBytes.length.toLong)
    payloadMemory.write(0, consumeParamsBytes, 0, consumeParamsBytes.length)

    INSTANCE.consume(
      rspacePointer,
      payloadMemory,
      consumeParamsBytes.length
    )
  }

  override def beforeEach(): Unit =
    spacePtr = INSTANCE.space_new(testDirectoryPath)

  override def afterEach(): Unit = {
    INSTANCE.space_clear(spacePtr)
    deleteRecursively(testDirectoryPath)
  }

  def channelsN(n: Int): List[Par] =
    (1 to n).map(x => byteName(x.toByte)).toList

  private def byteName(b: Byte): Par = GPrivate(ByteString.copyFrom(Array[Byte](b)))

  def patternsN(n: Int): List[BindPattern] =
    (1 to n)
      .map(
        _ => BindPattern(Vector(EVar(Var(FreeVar(0)))), freeCount = 1)
      )
      .toList

  def continuation(
      par: Par = Par().withExprs(Seq(GInt(1))),
      r: Blake2b512Random = Blake2b512Random(128)
  ): TaggedContinuation =
    TaggedContinuation(ParBody(ParWithRandom(par, r)))

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
