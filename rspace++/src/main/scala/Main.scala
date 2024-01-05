package rspacePlusPlus

import com.sun.jna._
import java.nio.charset.StandardCharsets
import java.nio.ByteBuffer
import io.circe.parser._
import io.circe.generic.auto._

import firefly.rtypes.{Address, Commit, Entry, Name, OptionResult, Retrieve}

object Main extends Library {
  val _ = System.setProperty("jna.library.path", "./rspace++/target/release/")

  // val libraryPath = System.getProperty("jna.library.path")
  // println(s"JNA library path is: $libraryPath")

  val lib = Native
    .load("rspace_plus_plus", classOf[RSpacePlusPlus[Array]])
    .asInstanceOf[RSpacePlusPlus[Array]]

  def getCityField(entry: Entry): String =
    entry.address.get.city

  def getLastNameField(entry: Entry): String =
    entry.name.get.last

  def getStateField(entry: Entry): String =
    entry.address.get.state

  def main(args: Array[String]): Unit = {
    val spacePtr = lib.space_new()
    val setup    = Setup.apply();

    val channel = "friends"

    // Consume
    val commit1 =
      Commit(Seq("friends"), Seq(setup.cityMatchCase), "I am the continuation, for now...");
    val commit1_buf = commit1.toByteArray;
    val cres1       = lib.space_put_once_durable_concurrent(spacePtr, commit1_buf, commit1_buf.length);
    println(cres1)

    // Consume
    // val commit2 =
    //   Commit(Seq("friends"), Seq(setup.cityPattern), "I am the continuation, for now...");
    // val commit2_buf = commit2.toByteArray;
    // val cres2       = lib.space_put_once_durable_concurrent(spacePtr, commit2_buf, commit2_buf.length);
    // println(cres2)

    // Produce
    val retrieve1     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres1         = lib.space_get_once_durable_concurrent(spacePtr, retrieve1_buf, retrieve1_buf.length);
    println(pres1);

    // Produce
    // val retrieve2     = Retrieve("friends", Some(setup.alice), cityMatchCase(setup.alice));
    // val retrieve2_buf = retrieve2.toByteArray;
    // val pres2         = lib.space_get_once_durable_concurrent(spacePtr, retrieve2_buf, retrieve2_buf.length);
    // println(pres2);

    lib.space_print(spacePtr, channel)
    lib.space_clear(spacePtr)

    val pres1_obj = decode[Pres_Class](pres1)

    pres1_obj match {
      case Right(p)  => println(p)
      case Left(err) => println(err)
    }
  }
}
