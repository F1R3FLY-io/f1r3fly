import org.scalatest.funsuite.AnyFunSuite
import rspacePlusPlus.{RSpacePlusPlus, Setup}
import firefly.rtypes.{Commit, Entry, Retrieve}
import com.sun.jna._
import java.io.File

class MemConcTest extends AnyFunSuite {
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

  // In-Memory Concurrent
  test("MemConcProduceMatch") {
    // Consume
    val commit =
      Commit(Seq("friends"), Seq(setup.cityMatchCase), "I am the continuation, for now...");
    val commit_buf = commit.toByteArray;
    val cres       = lib.space_put_once_non_durable_concurrent(spacePtr, commit_buf, commit_buf.length);

    // Produce
    val retrieve     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve_buf = retrieve.toByteArray;
    val pres =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve_buf, retrieve_buf.length);

    assert(cres == null)
    assert(!pres.isEmpty())
    assert(lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcProduceNoMatch") {
    // Consume
    val commit =
      Commit(Seq("friends"), Seq(setup.cityMatchCase), "I am the continuation, for now...");
    val commit_buf = commit.toByteArray;
    val cres       = lib.space_put_once_non_durable_concurrent(spacePtr, commit_buf, commit_buf.length);

    // Produce
    val retrieve     = Retrieve("friends", Some(setup.carol), getCityField(setup.carol));
    val retrieve_buf = retrieve.toByteArray;
    val pres =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve_buf, retrieve_buf.length);

    assert(cres == null)
    assert(pres == null)
    assert(!lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcConsumeMatch") {
    // Produce
    val retrieve     = Retrieve("friends", Some(setup.bob), getLastNameField(setup.bob));
    val retrieve_buf = retrieve.toByteArray;
    val pres =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve_buf, retrieve_buf.length);

    // Consume
    val commit =
      Commit(Seq("friends"), Seq(setup.nameMatchCase), "I am the continuation, for now...");
    val commit_buf = commit.toByteArray;
    val cres       = lib.space_put_once_non_durable_concurrent(spacePtr, commit_buf, commit_buf.length);

    assert(pres == null)
    assert(!cres.isEmpty)
    assert(lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcMultipleChannelsConsumeMatch") {
    // Produce
    val retrieve1     = Retrieve("colleagues", Some(setup.dan), getStateField(setup.dan));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres1 =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve1_buf, retrieve1_buf.length);

    // Produce
    val retrieve2     = Retrieve("friends", Some(setup.erin), getStateField(setup.erin));
    val retrieve2_buf = retrieve2.toByteArray;
    val pres2 =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve2_buf, retrieve2_buf.length);

    // Consume
    val commit =
      Commit(
        Seq("friends", "colleagues"),
        Seq(setup.stateMatchCase, setup.stateMatchCase),
        "I am the continuation, for now..."
      );
    val commit_buf = commit.toByteArray;
    val cres       = lib.space_put_once_non_durable_concurrent(spacePtr, commit_buf, commit_buf.length);

    assert(pres1 == null)
    assert(pres2 == null)
    assert(!cres.isEmpty)
    assert(cres.length == 2)
    assert(lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcConsumePersist") {
    // Consume
    val commit =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit_buf = commit.toByteArray;
    val cres =
      lib.space_put_always_non_durable_concurrent(spacePtr, commit_buf, commit_buf.length);

    assert(cres == null)
    assert(!lib.is_empty(spacePtr));

    // Produce
    val retrieve1     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve1_buf, retrieve1_buf.length);

    assert(!pres.isEmpty())
    assert(!lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcConsumePersistExistingMatches") {
    // Produce
    val retrieve1     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres1 =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve1_buf, retrieve1_buf.length);

    assert(pres1 == null)

    // Produce
    val retrieve2     = Retrieve("friends", Some(setup.bob), getCityField(setup.alice));
    val retrieve2_buf = retrieve2.toByteArray;
    val pres2 =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve2_buf, retrieve2_buf.length);

    assert(pres2 == null)

    // Consume
    val commit1 =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit1_buf = commit1.toByteArray;
    val cres1 =
      lib.space_put_always_non_durable_concurrent(spacePtr, commit1_buf, commit1_buf.length);

    assert(cres1.length == 1)
    assert(!lib.is_empty(spacePtr));

    val commit2 =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit2_buf = commit2.toByteArray;
    val cres2 =
      lib.space_put_always_non_durable_concurrent(spacePtr, commit2_buf, commit2_buf.length);

    assert(cres2.length == 1)
    assert(lib.is_empty(spacePtr));

    val commit3 =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit3_buf = commit3.toByteArray;
    val cres3 =
      lib.space_put_always_non_durable_concurrent(spacePtr, commit3_buf, commit3_buf.length);

    assert(cres3 == null)
    assert(!lib.is_empty(spacePtr));

    // Produce
    val retrieve3     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve3_buf = retrieve3.toByteArray;
    val pres3 =
      lib.space_get_once_non_durable_concurrent(spacePtr, retrieve3_buf, retrieve3_buf.length);

    assert(!pres3.isEmpty())
    assert(!lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcProducePersist") {
    // Produce
    val retrieve     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve_buf = retrieve.toByteArray;
    val pres =
      lib.space_get_always_non_durable_concurrent(spacePtr, retrieve_buf, retrieve_buf.length);

    assert(pres == null)
    assert(!lib.is_empty(spacePtr));

    // Consume
    val commit =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit_buf = commit.toByteArray;
    val cres       = lib.space_put_once_non_durable_concurrent(spacePtr, commit_buf, commit_buf.length);

    assert(!cres.isEmpty)
    assert(cres.length == 1)
    assert(!lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }

  test("MemConcProducePersistExistingMatches") {
    // Consume
    val commit1 =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit1_buf = commit1.toByteArray;
    val cres1 =
      lib.space_put_once_non_durable_concurrent(spacePtr, commit1_buf, commit1_buf.length);

    assert(cres1 == null)
    assert(!lib.is_empty(spacePtr));

    // Produce
    val retrieve1     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve1_buf = retrieve1.toByteArray;
    val pres1 =
      lib.space_get_always_non_durable_concurrent(spacePtr, retrieve1_buf, retrieve1_buf.length);

    assert(!pres1.isEmpty())
    assert(lib.is_empty(spacePtr));

    // Produce
    val retrieve2     = Retrieve("friends", Some(setup.alice), getCityField(setup.alice));
    val retrieve2_buf = retrieve2.toByteArray;
    val pres2 =
      lib.space_get_always_non_durable_concurrent(spacePtr, retrieve2_buf, retrieve2_buf.length);

    // Consume
    val commit2 =
      Commit(
        Seq("friends"),
        Seq(setup.cityMatchCase),
        "I am the continuation, for now..."
      );
    val commit2_buf = commit2.toByteArray;
    val cres2 =
      lib.space_put_once_non_durable_concurrent(spacePtr, commit2_buf, commit2_buf.length);

    assert(pres2 == null)
    assert(!cres2.isEmpty)
    assert(!lib.is_empty(spacePtr));
    lib.space_clear(spacePtr);
  }
}
