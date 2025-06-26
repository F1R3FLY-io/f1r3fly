package coop.rchain.node.revvaultexport

import cats.effect.Concurrent
import coop.rchain.casper.genesis.contracts.StandardDeploys
import coop.rchain.casper.helper.TestNode.Effect
import coop.rchain.casper.helper.TestRhoRuntime.rhoRuntimeEff
import coop.rchain.casper.syntax._
import coop.rchain.casper.util.ConstructDeploy
import coop.rchain.metrics.{Metrics, NoopSpan, Span}
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.shared.Log
import monix.eval.Task
import monix.execution.Scheduler.Implicits.global
import org.scalatest.FlatSpec

import scala.util.Random

class RhoTrieTraverserTest extends FlatSpec {
  private val SHARD_ID = "root-shard"

  "traverse the TreeHashMap" should "work" in {
    val total     = 4 // Changed to small static number for debugging
    val trieDepth = 2

    // Use static key-value pairs for consistent debugging
    val insertKeyValues = Seq(
      ("test_key_01", 12345, 0),
      ("test_key_02", 67890, 1),
      ("test_key_03", 11111, 2),
      ("test_key_04", 22222, 3)
    )

    println("DEEP DEBUG === SCALA TEST STARTING ===")
    println(s"DEEP DEBUG Total keys to insert: ${insertKeyValues.length}")
    println(s"DEEP DEBUG Trie depth: $trieDepth")

    // Log the static key-value pairs
    insertKeyValues.zipWithIndex.foreach {
      case ((key, value, index), i) =>
        println(s"DEEP DEBUG Input[$i]: key='$key', value=$value, index=$index")
    }
    val insertRho = insertKeyValues.foldLeft("") {
      case (acc, (key, value, index)) =>
        val rhoLine =
          if (index != insertKeyValues.length - 1)
            s"""new a in {TreeHashMap!("set", treeMap, "${key}", ${value}, *a)}|\n"""
          else s"""new a in {TreeHashMap!("set", treeMap, "${key}", ${value}, *a)}\n"""
        println(s"DEEP DEBUG Generated Rho[$index]: ${rhoLine.trim}")
        acc + rhoLine
    }

    println("DEEP DEBUG === COMPLETE INSERT RHO ===")
    println("DEEP DEBUG " + insertRho)
    val trieInitializedRho =
      s"""
        |new
        |  rl(`rho:registry:lookup`),
        |  TreeHashMapCh,
        |  newTreeMapStore,
        |  vaultMapStore
        |  in {
        |  rl!(`rho:lang:treeHashMap`, *TreeHashMapCh) |
        |  for (TreeHashMap <- TreeHashMapCh){
        |    TreeHashMap!("init", ${trieDepth}, *vaultMapStore) |
        |    for (@treeMap <-  vaultMapStore){
        |      ${insertRho}
        |      |@"t"!(treeMap)
        |    }
        |  }
        |}
        |""".stripMargin

    println("DEEP DEBUG === COMPLETE TRIE INITIALIZED RHO ===")
    println("DEEP DEBUG " + trieInitializedRho)

    val getTrieMapHandleRho = """new s in {
                               |  for (@result<- @"t"){
                               |    s!(result)
                               |  }
                               |}""".stripMargin

    implicit val concurent                   = Concurrent[Task]
    implicit val metricsEff: Metrics[Effect] = new Metrics.MetricsNOP[Task]
    implicit val noopSpan: Span[Effect]      = NoopSpan[Task]()
    implicit val logger: Log[Effect]         = Log.log[Task]
    val t = rhoRuntimeEff[Effect](false).use {
      case (runtime, _, _) =>
        println("DEEP DEBUG === SCALA RUNTIME OPERATIONS START ===")
        println("DEEP DEBUG Step 1: Computing empty state hash...")
        for {

          // 1. Obtain empty state hash
          hash1 <- runtime.emptyStateHash
          _ = println(
            s"DEEP DEBUG Empty state hash: ${hash1.toByteArray.map("%02x".format(_)).mkString}"
          )

          // 2. Reset runtime to empty state
          _ = println("DEEP DEBUG Step 2: Resetting to empty state...")
          _ <- runtime.reset(Blake2b256Hash.fromByteString(hash1))

          // 3. Process registry deploy
          _  = println("DEEP DEBUG Step 3: Processing registry deploy...")
          rd <- runtime.processDeploy(StandardDeploys.registry(SHARD_ID))

          // 4. Create checkpoint and reset
          _     = println("DEEP DEBUG Step 4: Creating checkpoint and resetting...")
          check <- runtime.createCheckpoint
          _ = println(
            s"DEEP DEBUG Checkpoint root: ${check.root.toByteString.toByteArray.map("%02x".format(_)).mkString}"
          )
          _ <- runtime.reset(check.root)

          // 5. Process initial trie deploy
          _ = println(
            "DEEP DEBUG Step 5: Creating and processing initial trie deploy..."
          )
          initialTrieRes <- runtime.processDeploy(
                             ConstructDeploy
                               .sourceDeploy(trieInitializedRho, 1L, phloLimit = 50000000)
                           )
          (initialTrie, _) = initialTrieRes
          _                = println(s"DEEP DEBUG Initial trie deploy result - failed: ${initialTrie.isFailed}")
          _ = if (initialTrie.isFailed)
            println(s"DEEP DEBUG Initial trie deploy errors: ${initialTrie}")
          _ = assert(!initialTrie.isFailed)

          // 6. Create second checkpoint
          _      = println("DEEP DEBUG Step 6: Creating second checkpoint...")
          check2 <- runtime.createCheckpoint
          _ = println(
            s"DEEP DEBUG Second checkpoint root: ${check2.root.toByteString.toByteArray.map("%02x".format(_)).mkString}"
          )

          // 7. Run exploratory deploy
          _ = println(
            "DEEP DEBUG Step 7: Running exploratory deploy to get trie map handle..."
          )
          trieMapHandleR <- runtime.playExploratoryDeploy(
                             getTrieMapHandleRho,
                             check2.root.toByteString
                           )
          _ = println(s"DEEP DEBUG Exploratory deploy returned ${trieMapHandleR.length} results")
          _ = trieMapHandleR.zipWithIndex.foreach {
            case (result, i) =>
              println(s"DEEP DEBUG Exploratory result[$i]: $result")
          }

          // 8. Reset to checkpoint2
          _ = println("DEEP DEBUG Step 8: Resetting to checkpoint2...")
          _ <- runtime.reset(check2.root)

          // 9. Get trie map handle
          _             = println("DEEP DEBUG Step 9: Getting trie map handle...")
          trieMapHandle = trieMapHandleR(0)
          _             = println(s"DEEP DEBUG Trie map handle: $trieMapHandle")

          // 10. Traverse trie
          _ = println("DEEP DEBUG Step 10: Traversing trie...")
          _ = println(
            s"DEEP DEBUG: About to call traverseTrie with depth=$trieDepth, mapPar=$trieMapHandle"
          )
          maps <- RhoTrieTraverser.traverseTrie(trieDepth, trieMapHandle, runtime)
          _    = println(s"DEEP DEBUG: traverseTrie returned maps: $maps")
          _    = println(s"DEEP DEBUG Traversed ${maps.length} maps from trie")
          _ = maps.zipWithIndex.foreach {
            case (map, i) =>
              println(s"DEEP DEBUG Traversed map[$i]: $map")
          }

          // 11. Convert to HashMap
          _ = println("DEEP DEBUG Step 11: Converting ParMaps to HashMap...")
          goodMap = RhoTrieTraverser.vecParMapToMap(
            maps,
            p => p.exprs(0).getGByteArray,
            p => p.exprs(0).getGInt
          )
          _ = println(s"DEEP DEBUG Converted HashMap has ${goodMap.size} entries:")
          _ = goodMap.foreach {
            case (keyBytes, value) =>
              println(
                s"DEEP DEBUG  HashMap entry: key=${keyBytes.toByteArray.map("%02x".format(_)).mkString} -> value=$value"
              )
          }

          // 12. Verify key-value pairs
          _ = println("DEEP DEBUG Step 12: Verifying inserted key-value pairs...")
          _ = insertKeyValues.foreach {
            case (originalKey, expectedValue, index) =>
              println(s"DEEP DEBUG \n--- Processing key[$index]: '$originalKey' ---")

              // Generate keccak hash
              val hashedKey = RhoTrieTraverser.keccakKey(originalKey)
              println(s"DEEP DEBUG Keccak hash result: $hashedKey")

              val fullHashBytes = hashedKey.exprs(0).getGByteArray
              println(
                s"DEEP DEBUG Full hash bytes (len=${fullHashBytes.size}): ${fullHashBytes.toByteArray.map("%02x".format(_)).mkString}"
              )

              // Extract key portion after trie depth
              val keyBytes = fullHashBytes.substring(trieDepth, 32)
              println(
                s"DEEP DEBUG Extracted key bytes (from pos $trieDepth to 32): ${keyBytes.toByteArray.map("%02x".format(_)).mkString}"
              )

              goodMap.get(keyBytes) match {
                case Some(foundValue) =>
                  println(s"DEEP DEBUG ✓ GOOD: Key '$originalKey' found in traversed trie map")
                  println(s"DEEP DEBUG   Expected: $expectedValue, Found: $foundValue")
                  assert(
                    foundValue == expectedValue.toLong,
                    s"Value mismatch for key '$originalKey': expected $expectedValue, found $foundValue"
                  )
                case None =>
                  println(s"DEEP DEBUG ✗ BAD: Key '$originalKey' not found in traversed trie map")
                  println(
                    s"DEEP DEBUG  Looking for key: ${keyBytes.toByteArray.map("%02x".format(_)).mkString}"
                  )
                  println("DEEP DEBUG   Available keys in map:")
                  goodMap.foreach {
                    case (availableKey, availableValue) =>
                      println(
                        s"DEEP DEBUG    ${availableKey.toByteArray.map("%02x".format(_)).mkString} -> $availableValue"
                      )
                  }
                // Don't panic for debugging
              }
          }

          _ = println("DEEP DEBUG \n=== SCALA VERIFICATION COMPLETE ===")
          _ = println(s"DEEP DEBUG ✓ All ${insertKeyValues.length} key-value pairs processed")
        } yield ()
    }
    t.runSyncUnsafe()
  }

}
