import coop.rchain.casper.util.comm._
import java.nio.file.Paths
import monix.eval.Task

class Playground {
  val rhoContractPath = "./rholang/examples/tut-registry.rho"
  val vpk             = "aebb63dc0d50e4dd29ddd94fb52103bfe0dc4941fa0c2c8a9082a191af35ffa1"
  val host            = "127.0.0.1"
  val port            = 40402

  def go(): Unit =
    printCurrentDirectory()

  // Clients for executing gRPC calls on remote RNode instance
  implicit val deployServiceClient: GrpcDeployService[Task] =
    new GrpcDeployService[Task](
      host,
      port,
      16 * 1024 * 1024
    )
  implicit val proposeServiceClient: GrpcProposeService[Task] =
    new GrpcProposeService[Task](
      host,
      port,
      16 * 1024 * 1024
    )
  // val rhoContractCode = loadFileContent(rhoContractPath)

  // val ddp = DeployDataProto(
  //   term = rhoContractCode,
  //   timestamp = 0,
  //   phloPrice = 1,
  //   phloLimit = 1000000,
  //   shardId = "root",
  //   validAfterBlockNumber = 0
  // )

  // val signedDeploy = signDeploy(ECPrivateKey(vpk), ddp)

  // // Deploy
  // val deployVolumeContract: Future[Unit] = for {
  //   deployResponse <- DeployService.doDeploy(signedDeploy).toScala
  //   deployResult <- if (deployResponse.hasError) {
  //                    Future.failed(new Exception(deployResponse.getError))
  //                  } else {
  //                    Future.successful(deployResponse.getResult)
  //                  }
  //   deployId = deployResult.substring(deployResult.indexOf("DeployId is: ") + 13)
  //   proposeResponse <- proposeService
  //                       .propose(ProposeQuery.newBuilder().setIsAsync(false).build())
  //                       .toScala
  //   _ <- if (proposeResponse.hasError) {
  //         Future.failed(new Exception(proposeResponse.getError))
  //       } else {
  //         Future.successful(())
  //       }
  //   b64 = ByteString.copyFrom(Hex.decode(deployId))
  //   findResponse <- deployService
  //                    .findDeploy(FindDeployQuery.newBuilder().setDeployId(b64).build())
  //                    .toScala
  //   blockHash <- if (findResponse.hasError) {
  //                 Future.failed(new Exception(findResponse.getError))
  //               } else {
  //                 Future.successful(findResponse.getBlockInfo.getBlockHash)
  //               }
  //   isFinalizedResponse <- deployService
  //                           .isFinalized(IsFinalizedQuery.newBuilder().setHash(blockHash).build())
  //                           .toScala
  //   _ <- if (isFinalizedResponse.hasError || !isFinalizedResponse.getIsFinalized) {
  //         Future.failed(new Exception(isFinalizedResponse.getError))
  //       } else {
  //         Future.successful(())
  //       }
  // } yield ()

  // deployVolumeContract.onComplete {
  //   case Success(_)         => println("Deployment successful")
  //   case Failure(exception) => println(s"Deployment failed: ${exception.getMessage}")
  // }

  // def signDeploy(privKey: ECPrivateKey, deploy: DeployDataProto): DeployDataProto = {
  //   // Take a projection of only the fields used to validate the signature
  //   val projection = DeployDataProto(
  //     term = deploy.term,
  //     timestamp = deploy.timestamp,
  //     phloPrice = deploy.phloPrice,
  //     phloLimit = deploy.phloLimit,
  //     validAfterBlockNumber = deploy.validAfterBlockNumber,
  //     shardId = deploy.shardId
  //   )
  //   val serialized = ByteVector(projection.toByteArray)
  //   val deployer   = privKey.publicKey.decompressed
  //   val digest     = ByteVector(Blake2b256.hash(serialized.toArray))
  //   val signed     = privKey.sign(digest)
  //   projection.copy(
  //     sigAlgorithm = "secp256k1",
  //     sig = ByteString.copyFrom(signed.bytes.toArray),
  //     deployer = ByteString.copyFrom(deployer.decompressedBytes.toArray)
  //   )
  // }

  def printCurrentDirectory(): Unit = {
    val currentPath = Paths.get("").toAbsolutePath.toString
    println(s"Current working directory: $currentPath")
  }

  // def loadFileContent(filePath: String): String =
  //   new String(Files.readAllBytes(Paths.get(filePath)), StandardCharsets.UTF_8)

}

object PlaygroundApp extends App {
  val playground = new Playground()
  playground.go()
}