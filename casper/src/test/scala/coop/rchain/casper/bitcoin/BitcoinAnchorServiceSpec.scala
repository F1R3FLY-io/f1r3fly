package coop.rchain.casper.bitcoin

import cats.effect.IO
import com.google.protobuf.ByteString
import coop.rchain.shared.Log
import monix.eval.Task
import monix.execution.Scheduler.Implicits.global
import org.scalatest.WordSpec

class BitcoinAnchorServiceSpec extends WordSpec {

  import BitcoinAnchorTestUtils._

  implicit val logTask: Log[Task] = Log.log[Task]
  implicit val logIO: Log[IO]     = Log.log[IO]

  "BitcoinAnchorService" should {
    "create service successfully with valid configuration" in {
      val test = BitcoinAnchorService.apply[Task](validConfig).use { service =>
        Task {
          assert(service != null)
          assert(service.isInstanceOf[BitcoinAnchorService[Task]])
        }
      }

      test.runSyncUnsafe()
    }

    "successfully call anchor finalization without errors" in {
      val commitment = createValidCommitment()

      val test = BitcoinAnchorService.apply[Task](validConfig).use { service =>
        service.anchorFinalization(commitment).map { result =>
          assert(result.success == true)
          assert(result.errorMessage == "")
          assert(result.transactionId.nonEmpty)
          assert(result.transactionId.startsWith("mock_tx_"))
          assert(result.feeSats >= 0)
        }
      }

      test.runSyncUnsafe()
    }

    "properly cleanup resources on shutdown" in {
      // Test that Resource cleanup works without throwing exceptions
      val test = BitcoinAnchorService.apply[Task](validConfig).use { service =>
        Task.pure(()) // Service automatically cleaned up when Resource exits
      }

      // If this completes without hanging or throwing, Resource cleanup worked
      test.runSyncUnsafe()
    }

    "handle invalid configuration gracefully" in {
      val test = BitcoinAnchorService
        .apply[Task](invalidConfig)
        .use { _ =>
          Task.pure(())
        }
        .attempt

      val result = test.runSyncUnsafe()
      assert(result.isLeft)
    }

    "handle malformed commitment data gracefully" in {
      val invalidCommitment = F1r3flyStateCommitmentProto(
        lfbHash = ByteString.EMPTY,    // Invalid: too short
        rspaceRoot = ByteString.EMPTY, // Invalid: too short
        blockHeight = -1L,             // Invalid: negative
        timestamp = 0L,
        validatorSetHash = ByteString.EMPTY // Invalid: too short
      )

      val test = BitcoinAnchorService.apply[Task](validConfig).use { service =>
        service.anchorFinalization(invalidCommitment).attempt.map {
          case Left(_) => // Expected failure
            assert(true)
          case Right(result) =>
            // If FFI accepts invalid data, it should at least indicate failure
            assert(result.success == false)
            assert(result.errorMessage.nonEmpty)
        }
      }

      test.runSyncUnsafe()
    }

    "work with different network configurations" in {
      val networks = List("regtest", "signet", "mainnet")

      networks.foreach { network =>
        val config = validConfig.copy(network = network)

        val test = BitcoinAnchorService.apply[Task](config).use { service =>
          service.anchorFinalization(createValidCommitment()).map { result =>
            assert(result.success == true)
            assert(result.transactionId.nonEmpty)
          }
        }

        test.runSyncUnsafe()
      }
    }

    "handle concurrent calls safely" in {
      val commitment = createValidCommitment()

      val test = BitcoinAnchorService.apply[Task](validConfig).use { service =>
        val concurrentCalls = List.fill(5) {
          service.anchorFinalization(commitment)
        }

        Task.parSequence(concurrentCalls).map { results =>
          assert(results.length == 5)
          results.foreach { result =>
            assert(result.success == true)
            assert(result.transactionId.nonEmpty)
          }
        }
      }

      test.runSyncUnsafe()
    }

    "create service from CasperConf when enabled" in {
      val test = BitcoinAnchorService.createForEngine[Task](enabledCasperConf).use { serviceOpt =>
        Task {
          assert(serviceOpt.isDefined)
          assert(serviceOpt.get.isInstanceOf[BitcoinAnchorService[Task]])
        }
      }

      test.runSyncUnsafe()
    }

    "return None from CasperConf when disabled" in {
      val test = BitcoinAnchorService.createForEngine[Task](disabledCasperConf).use { serviceOpt =>
        Task {
          assert(serviceOpt.isEmpty)
        }
      }

      test.runSyncUnsafe()
    }
  }
}
