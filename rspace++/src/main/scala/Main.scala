package rspacePlusPlus

import java.nio.charset.StandardCharsets
import java.nio.ByteBuffer
import io.circe.parser._
import io.circe.generic.auto._
import org.scalacheck.Gen
import scala.collection.SortedSet

import coop.rchain.crypto.hash.Blake2b512Random
import coop.rchain.models.{SortedSetElement}

import cats.Id

import coop.rchain.rspace.hashing.Blake2b256Hash
import scodec.bits.ByteVector

object Main {
  def main(args: Array[String]): Unit = {
    val protobuf      = SortedSetElement(24)
    val protobufBytes = protobuf.toByteArray
    val byteVector    = ByteVector(protobufBytes)
    val hash          = Blake2b256Hash.create(byteVector)

    println("\n" + byteVector)
    println("\nHash: " + hash)

    // val rspace = new RSpacePlusPlus_16_RhoTypes();
    // val rspace = new RSpacePlusPlus_RhoTypes[Id]();

    // val cres =
    //   rspace.put_once_durable_sequential(myPars, Seq(myBindPattern), myTaggedContinuation)
    // val pres = rspace.get_once_durable_sequential(myPar, myListParWithRandom)
    // rspace.print()
    // rspace.clear()

    // val r1 = rspace.produce(myPar, myListParWithRandom, false);
    // println("r1: " + r1);
    // rspace.print();

    // val r2 =
    //   rspace.consume(myPars, Seq(myBindPattern), myTaggedContinuation, false, SortedSet.empty[Int]);
    // println("r2: " + r2);
    // rspace.print();

    // rspace.clear();
  }
}
