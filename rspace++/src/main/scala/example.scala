package rspacePlusPlus

import firefly.rtypes.{Commit, Entry, Retrieve}
import com.sun.jna._

object Example {
  def run(args: Array[String]): Unit = {

    // Demo Code
    /*
    def cityMatchCase(entry: Entry): String =
      entry.address.get.city

    // def nameMatchCase(entry: Entry): String =
    //   entry.name.get.last

    def stateMatchCase(entry: Entry): String =
      entry.address.get.state

    val _ = System.setProperty("jna.library.path", "./rspace++/target/release/")
    val lib =
      Native
        .load("rspace_plus_plus", classOf[RustLibrary.RustLib])
        .asInstanceOf[RustLibrary.RustLib]

    val spacePtr = lib.space_new();
    val setup    = Setup.apply();

    // DiskSeqProducePersistExistingMatches
    if (args.apply(0) == "put_once_durable_sequential") {
      val receive1 =
        Receive(
          Seq("friends"),
          Seq(setup.cityPattern),
          "continuation-print_entry"
        );
      val receive1_buf = receive1.toByteArray;

      val cres1 = lib.space_put_once_durable_sequential(spacePtr, receive1_buf, receive1_buf.length);
      if (cres1 == null) {
        println("\n RESPONSE: " + cres1)
      } else {
        println("\n RESPONSE: " + cres1.apply(0))
      }

      lib.space_print(spacePtr, "friends")
    } else if (args.apply(0) == "get_always_durable_sequential") {
      val send1     = Send("friends", Some(setup.alice), cityMatchCase(setup.alice));
      val send1_buf = send1.toByteArray;

      val pres1 = lib.space_get_always_durable_sequential(spacePtr, send1_buf, send1_buf.length);
      println("\n RESPONSE: " + pres1)

      lib.space_print(spacePtr, "friends")

      // MemSeqMultipleChannelsConsumeMatch - db getting cleared after each sbt run
    } else if (args.apply(0) == "get_once_non_durable_sequential") {
      @SuppressWarnings(Array("org.wartremover.warts.Var"))
      var data = setup.dan
      if (args.apply(2) == "erin") {
        data = setup.erin
      }

      val send1     = Send(args.apply(1), Some(data), stateMatchCase(data));
      val send1_buf = send1.toByteArray;
      val pres1     = lib.space_get_once_non_durable_sequential(spacePtr, send1_buf, send1_buf.length);
      println("\n RESPONSE: " + pres1)

      lib.space_print(spacePtr, args.apply(1))
    } else if (args.apply(0) == "put_once_non_durable_sequential") {
      val receive =
        Receive(
          Seq("friends", "colleagues"),
          Seq(setup.statePattern, setup.statePattern),
          "continuation-print_entry"
        );
      val receive_buf = receive.toByteArray;
      val cres =
        lib.space_put_once_non_durable_sequential(spacePtr, receive_buf, receive_buf.length);

      if (cres == null) {
        println("\n RESPONSE: " + cres)
      } else {
        println("\n RESPONSE: " + cres)
      }

      lib.space_print(spacePtr, "friends")
      lib.space_print(spacePtr, "colleagues")
    } else {
      println(args.apply(0))
      lib.space_clear(spacePtr);
    }
		*/
  }
}
