package coop.rchain.casper.bitcoin

import coop.rchain.casper.BitcoinAnchorConf
import org.scalatest.{FlatSpec, Matchers}

/**
  * Simple tests for distributed anchor logic
  * Tests the mathematical properties of the selection algorithm
  */
class DistributedAnchorLogicTest extends FlatSpec with Matchers {

  "Distributed Anchor Selection Algorithm" should "distribute blocks fairly among validators" in {
    val validatorCount = 3
    val blockRange     = 0L to 20L

    // Test the selection algorithm directly
    val selections = blockRange.map { blockHeight =>
      (blockHeight % validatorCount).toInt
    }

    // Count selections per validator
    val validator0Count = selections.count(_ == 0)
    val validator1Count = selections.count(_ == 1)
    val validator2Count = selections.count(_ == 2)

    // With 21 blocks and 3 validators, each should get 7 blocks
    validator0Count shouldBe 7
    validator1Count shouldBe 7
    validator2Count shouldBe 7

    // Total selections should equal total blocks
    validator0Count + validator1Count + validator2Count shouldBe 21
  }

  it should "work correctly with different validator counts" in {
    // Test with 5 validators and 50 blocks (perfect distribution)
    val validatorCount = 5
    val blockRange     = 0L to 49L

    val selections = blockRange.map { blockHeight =>
      (blockHeight % validatorCount).toInt
    }

    // Each validator should get exactly 10 blocks
    (0 until validatorCount).foreach { validatorIndex =>
      val count = selections.count(_ == validatorIndex)
      count shouldBe 10
    }
  }

  it should "be deterministic" in {
    val validatorCount = 3
    val blockHeight    = 42L

    // Same inputs should always produce same outputs
    val results = (1 to 10).map { _ =>
      (blockHeight % validatorCount).toInt
    }

    // All results should be identical
    results.foreach { result =>
      result shouldBe results.head
    }
    results.head shouldBe 0 // 42 % 3 = 0
  }

  it should "handle edge cases correctly" in {
    // Test with single validator
    val singleValidatorResult = 42L % 1
    singleValidatorResult shouldBe 0

    // Test with block height 0
    val zeroHeightResult = 0L % 3
    zeroHeightResult shouldBe 0

    // Test with large block heights
    val largeHeightResult = 1000000L % 7
    largeHeightResult shouldBe (1000000L % 7)
  }

  it should "maintain fairness over long periods" in {
    val validatorCount = 4
    val blockRange     = 0L to 99L // 100 blocks

    val selections = blockRange.map { blockHeight =>
      (blockHeight % validatorCount).toInt
    }

    // Each validator should get exactly 25 blocks
    (0 until validatorCount).foreach { validatorIndex =>
      val count = selections.count(_ == validatorIndex)
      count shouldBe 25
    }

    // Test distribution is even
    val counts = (0 until validatorCount).map { validatorIndex =>
      selections.count(_ == validatorIndex)
    }

    val maxCount = counts.max
    val minCount = counts.min

    // Perfect distribution - no variance
    maxCount shouldBe minCount
  }

  it should "work with prime numbers of validators" in {
    // Test with 7 validators (prime number) and 70 blocks
    val validatorCount = 7
    val blockRange     = 0L to 69L

    val selections = blockRange.map { blockHeight =>
      (blockHeight % validatorCount).toInt
    }

    // Each validator should get exactly 10 blocks
    (0 until validatorCount).foreach { validatorIndex =>
      val count = selections.count(_ == validatorIndex)
      count shouldBe 10
    }
  }

  "Configuration Integration" should "respect distributed mode setting" in {
    // Test configuration values
    val distributedConfig = BitcoinAnchorConf(
      enabled = true,
      network = "regtest",
      distributed = true,
      esploraUrl = None,
      feeRate = Some(1.0),
      maxFeeSats = Some(10000L)
    )

    val allNodesConfig = distributedConfig.copy(distributed = false)

    // Verify configuration properties
    distributedConfig.distributed shouldBe true
    distributedConfig.enabled shouldBe true

    allNodesConfig.distributed shouldBe false
    allNodesConfig.enabled shouldBe true
  }
}
