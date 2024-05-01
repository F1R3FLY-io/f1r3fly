package rspacePlusPlus

import java.nio.charset.StandardCharsets
import java.nio.ByteBuffer
import io.circe.parser._
import io.circe.generic.auto._
import org.scalacheck.Gen
import scala.collection.SortedSet
import com.sun.jna.{Memory, Native, Pointer}

import coop.rchain.crypto.hash.Blake2b512Random
import coop.rchain.models.rspace_plus_plus_types.{SortedSetElement}

import cats.Id

import coop.rchain.rspace.hashing.Blake2b256Hash
import scodec.bits.ByteVector
import coop.rchain.models.rspace_plus_plus_types.StoreStateDataMapEntry
import coop.rchain.rspace.history.RadixTree

object Main {
  def main(args: Array[String]): Unit = {
    def testFunction(): Unit = {
      System.setProperty("jna.library.path", "./rspace++/target/debug/")

      // val currentDir = java.nio.file.Paths.get("").toAbsolutePath.toString
      // println(s"Current directory: $currentDir")

      // val jnaLibraryPath = System.getProperty("jna.library.path")
      // println(s"Current jna.library.path: $jnaLibraryPath")

      val INSTANCE: JNAInterface =
        Native
          .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
          .asInstanceOf[JNAInterface]

      // val testFunctionPtr   = INSTANCE.test_function
      // val length            = testFunctionPtr.getInt(0)
      // val testFunctionBytes = testFunctionPtr.getByteArray(4, length)
      // val testFunctionProto = StoreStateDataMapEntry.parseFrom(testFunctionBytes)

      // println(testFunctionProto)

      // val input      = "hello world".getBytes("UTF-8")
      // val blake2Hash = Blake2b256Hash.create(input)
      // println(s"\nHash: ${blake2Hash.bytes.toHex}")

      val (nodeHashBytes, _) = RadixTree.hashNode(RadixTree.emptyNode)
      println("\nNode Hash: " + Blake2b256Hash.fromByteVector(nodeHashBytes))
      // println("\n") ByteString.copyFrom(checkpoint.root.bytes.toArray)
    }

    testFunction
  }

}
