import coop.rchain.casper.util.comm._
import java.nio.file.Paths
import monix.eval.Task
import coop.rchain.crypto.PrivateKey
import com.google.protobuf.ByteString
import monix.execution.Scheduler.Implicits.global
import coop.rchain.casper.protocol.DeployData
import java.nio.file.Files
import java.nio.charset.StandardCharsets
import coop.rchain.crypto.signatures.Secp256k1
import coop.rchain.crypto.signatures.Signed
import scala.util.{Failure, Success, Try}
import scodec.bits.ByteVector
import coop.rchain.casper.protocol.FindDeployQuery
import io.circe.Decoder
import coop.rchain.casper.protocol.LightBlockInfo

import io.circe._
import io.circe.generic.semiauto._
import scala.util.matching.Regex
import coop.rchain.casper.protocol.IsFinalizedQuery

class NodeCliPlayground {
  val rhoContractPath = "./rholang/examples/tut-registry.rho"
  val vpkHex          = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
  val vpkBytes        = javax.xml.bind.DatatypeConverter.parseHexBinary(vpkHex)
  val host            = "127.0.0.1"
  val port            = 40402

  def go(): Unit = {
    implicit val deployServiceClient: GrpcDeployService[Task] =
      new GrpcDeployService[Task](
        host,
        port,
        16 * 1024 * 1024
      )

    val rhoContractCode = loadFileContent(rhoContractPath)

    val d = DeployData(
      term = rhoContractCode,
      timestamp = 0,
      phloPrice = 1,
      phloLimit = 1000000,
      shardId = "root",
      validAfterBlockNumber = 0
    )

    val deployTask =
      DeployService[Task].deploy(Signed(d, Secp256k1, PrivateKey(ByteString.copyFrom(vpkBytes))))

    deployTask.runAsync {
      case Right(res) => {

        res match {
          case Right(str) => {
            println(s"Deploy $str")

            val deployIdPatternFull = "DeployId is: (.+)".r
            val deployId            = deployIdPatternFull.findFirstMatchIn(str).map(_.group(1))

            deployId match {
              case Some(id) => {
                propose()
                findDeploy(id)
              }
              case None =>
                println("Deploy ID not found in the output.")
            }
          }
          case Left(errors) => errors.foreach(println)
        }
      }
      case Left(err) => println(s"Deploy failed: $err")
    }
  }

  def propose(): Unit = {
    implicit val proposeServiceClient: GrpcProposeService[Task] =
      new GrpcProposeService[Task](
        host,
        port,
        16 * 1024 * 1024
      )

    val proposeTask =
      ProposeService[Task].propose(true)

    proposeTask.runAsync {
      case Right(res) => {

        res match {
          case Right(str) => {
            println("\npropose result: " + str)
          }
          case Left(errors) => errors.foreach(println)
        }
      }
      case Left(err) => println(s"Propose failed: $err")
    }
  }

  def findDeploy(deployId: String): Unit = {
    implicit val deployServiceClient: GrpcDeployService[Task] =
      new GrpcDeployService[Task](
        host,
        port,
        16 * 1024 * 1024
      )

    val base64String = convert(deployId)
    base64String match {
      case Some(value) => {
        val findDeployTask =
          DeployService[Task].findDeploy(
            FindDeployQuery(ByteString.copyFrom(value))
          )

        findDeployTask.runAsync {
          case Right(res) => {

            res match {
              case Right(str) => {
                // println(s"findDeploy result: $str")

                val blockHash = extractBlockHash(str).getOrElse("Not found")
                val isFinalizedTask =
                  DeployService[Task].isFinalized(
                    IsFinalizedQuery(blockHash)
                  )

                isFinalizedTask.runAsync {
                  case Right(res) => {
                    res match {
                      case Right(str) => {
                        println(s"isFinalized Result: $str")
                      }
                      case Left(errors) => errors.foreach(println)
                    }
                  }
                  case Left(err) => println(s"findDeploy failed: $err")
                }
              }
              case Left(errors) => errors.foreach(println)
            }
          }
          case Left(err) => println(s"findDeploy failed: $err")
        }
      }
      case None => println("Error converting hex to base64.")
    }
  }

  def loadFileContent(filePath: String): String =
    new String(Files.readAllBytes(Paths.get(filePath)), StandardCharsets.UTF_8)

  def printCurrentDirectory(): Unit = {
    val currentPath = Paths.get("").toAbsolutePath.toString
    println(s"Current working directory: $currentPath")
  }

  def convert(hexString: String): Option[Array[Byte]] =
    Try(ByteVector.fromHex(hexString).get.toArray) match {
      case Success(base64String) => Some(base64String)
      case Failure(exception)    => None
    }

  def extractBlockHash(input: String): Option[String] = {
    val blockHashPattern: Regex = """blockHash: "([a-fA-F0-9]+)"""".r
    blockHashPattern.findFirstMatchIn(input).map(_.group(1))
  }

}

object NodeCliPlaygroundApp extends App {
  val playground = new NodeCliPlayground()
  playground.go()
}
