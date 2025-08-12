package coop.rchain.casper.bitcoin

import cats.syntax.all._
import coop.rchain.casper.helper.TestNode
import coop.rchain.casper.helper.TestNode._
import coop.rchain.casper.protocol.BlockMessage
import coop.rchain.casper.util.ConstructDeploy
import coop.rchain.crypto.PublicKey
import coop.rchain.p2p.EffectsTestInstances.LogicalTime
import coop.rchain.shared.scalatestcontrib._
import coop.rchain.shared.Log
import monix.execution.Scheduler.Implicits.global
import monix.eval.Task
import org.scalatest.{FlatSpec, Matchers}

class BitcoinAnchorFinalizationIntegrationSpec extends FlatSpec with Matchers {

  import coop.rchain.casper.util.GenesisBuilder._
  import coop.rchain.casper.bitcoin.{BitcoinAnchorService, BitcoinAnchorTestUtils}

  implicit val timeEff             = new LogicalTime[Effect]
  implicit val logEff: Log[Effect] = Log.log[Effect]

  val genesis = buildGenesis(
    buildGenesisParameters(
      bondsFunction = (validators: Iterable[PublicKey]) => validators.map(pk => pk -> 10L).toMap
    )
  )

  "Bitcoin Anchor FFI Integration" should "successfully create and use real Bitcoin anchor service" in effectTest {
    // Test the actual Rust FFI bridge without full node integration
    BitcoinAnchorService.apply[Effect](BitcoinAnchorTestUtils.validConfig).use { bitcoinService =>
      for {
        _ <- Task.pure(println("SUCCESS: Bitcoin anchor service created with Rust FFI"))

        // Create a test commitment to verify the FFI bridge works
        // Now that FFI uses milliseconds, we can use current time
        commitment <- Task.pure(BitcoinAnchorTestUtils.createValidCommitment())
        _ <- Task.pure(
              println(s"Testing anchor finalization for block height ${commitment.blockHeight}")
            )

        // This calls the actual Rust FFI code
        result <- bitcoinService.anchorFinalization(commitment)

        _ <- Task.pure(
              println(
                s"Bitcoin anchor result: success=${result.success}, txId=${result.transactionId}"
              )
            )

        // Verify we got a valid result from the Rust code
        _ <- Task.pure(result.success shouldBe true)
        _ <- Task.pure(result.transactionId should not be empty)
        _ <- Task.pure(result.transactionId.length shouldBe 64) // Bitcoin transaction IDs are 64 hex characters

        _ <- Task.pure(println("SUCCESS: Rust FFI bridge working correctly"))
      } yield ()
    }
  }

  it should "work with multiple consecutive calls" in effectTest {
    // Test that the FFI bridge can handle multiple calls
    BitcoinAnchorService.apply[Effect](BitcoinAnchorTestUtils.validConfig).use { bitcoinService =>
      for {
        _ <- (1 to 3).toList.traverse { i =>
              for {
                commitment <- Task.pure(
                               BitcoinAnchorTestUtils
                                 .createValidCommitment()
                                 .copy(blockHeight = i.toLong)
                             )
                _      <- Task.pure(println(s"Anchoring block ${i}"))
                result <- bitcoinService.anchorFinalization(commitment)
                _      <- Task.pure(result.success shouldBe true)
              } yield ()
            }
        _ <- Task.pure(println("SUCCESS: Multiple FFI calls completed"))
      } yield ()
    }
  }

  it should "handle service lifecycle properly" in effectTest {
    // Test Resource management works correctly
    val test = BitcoinAnchorService.apply[Effect](BitcoinAnchorTestUtils.validConfig).use {
      service =>
        Task.pure(service != null)
    }

    for {
      serviceExists <- test
      _             <- Task.pure(serviceExists shouldBe true)
      _             <- Task.pure(println("SUCCESS: Bitcoin anchor service Resource lifecycle works"))
    } yield ()
  }

  it should "verify test infrastructure is working" in effectTest {
    // Simple verification that the test setup works without complex node networks
    for {
      _          <- Task.pure(println("Testing basic infrastructure..."))
      commitment <- Task.pure(BitcoinAnchorTestUtils.createValidCommitment())
      _          <- Task.pure(commitment.blockHeight should be > 0L)
      _          <- Task.pure(commitment.timestamp should be > 0L)
      _          <- Task.pure(println("SUCCESS: Test infrastructure verified"))
    } yield ()
  }
}
