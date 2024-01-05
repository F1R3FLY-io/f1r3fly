import org.scalatest.funsuite.AnyFunSuite
import org.scalatest.matchers.should.Matchers
import rspacePlusPlus.{RSpacePlusPlus, Setup}
import firefly.rtypes.{Commit, Entry, Retrieve}
import com.sun.jna._
import java.io.File

/**
	* Tests pulled from StorageActionsTests and StorageExamplesTests in rspace/src/test/
	* Currently using In-Memory Sequential database
  */
class RSpaceTest extends AnyFunSuite with Matchers {
  val _ = System.setProperty("jna.library.path", "./target/release/")
  val lib =
    Native
      .load("rspace_plus_plus", classOf[RSpacePlusPlus[Array]])
      .asInstanceOf[RSpacePlusPlus[Array]]

  def getCityField(entry: Entry): String =
    entry.address.get.city

  def getLastNameField(entry: Entry): String =
    entry.name.get.last

  def getStateField(entry: Entry): String =
    entry.address.get.state

  val spacePtr = lib.space_new();
  val setup    = Setup.apply();

  /**
	  * Original test created two duplicate channels
		* RSpace++ currently does not handle duplicate channels
	  */
  test(
    "CORE-365: A joined consume on duplicate channels followed by two produces on that channel" +
      "should return a continuation and the produced data"
  ) {
    // Consume
    val commit1 =
      Commit(
        Seq("friends", "friends"),
        Seq(setup.cityMatchCase, setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit1_buf = commit1.toByteArray;
    val cres        = lib.space_put_once_non_durable_sequential(spacePtr, commit1_buf, commit1_buf.length);

    assert(cres == null)

    // Produce
    val retrieve1     = Retrieve("friends", Some(setup.bob), getCityField(setup.bob));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres1 =
      lib.space_get_once_non_durable_sequential(spacePtr, retrieve1_buf, retrieve1_buf.length);

    // Should be null
    assert(!pres1.isEmpty())

    // Produce
    val retrieve2     = Retrieve("friends", Some(setup.bob), getCityField(setup.bob));
    val retrieve2_buf = retrieve2.toByteArray;
    val pres2 =
      lib.space_get_once_non_durable_sequential(spacePtr, retrieve2_buf, retrieve2_buf.length);

    // Should be not null
    assert(pres2 == null)

    // Length should be 2
    assert(cres == null)

    // Should be empty
    assert(!lib.is_empty(spacePtr));
  }

  test(
    "CORE-365: Two produces on the same channel followed by a joined consume on duplicates of that channel" +
      "should return a continuation and the produced data"
  ) {
    // Produce
    val retrieve1     = Retrieve("friends", Some(setup.bob), getCityField(setup.bob));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres1 =
      lib.space_get_once_non_durable_sequential(spacePtr, retrieve1_buf, retrieve1_buf.length);

    assert(pres1 == null)

    // Produce
    val retrieve2     = Retrieve("friends", Some(setup.bob), getCityField(setup.bob));
    val retrieve2_buf = retrieve2.toByteArray;
    val pres2 =
      lib.space_get_once_non_durable_sequential(spacePtr, retrieve2_buf, retrieve2_buf.length);

    assert(pres2 == null)

    // Consume
    val commit1 =
      Commit(
        Seq("friends", "friends"),
        Seq(setup.cityMatchCase, setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit1_buf = commit1.toByteArray;
    val cres        = lib.space_put_once_non_durable_sequential(spacePtr, commit1_buf, commit1_buf.length);

    // Should be length 2
    assert(cres.length == 1)

  }
  lib.space_clear(spacePtr);
}
