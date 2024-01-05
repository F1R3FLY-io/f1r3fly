import scala.collection.mutable.ListBuffer
// import ../../../../shared/main/scala/coop/rchain/shared/Language.scala
// import _root_.coop.rchain.shared.Language.ignore
// import coop.rchain.shared.Language.ignore
import cats.{Id}

object Tutuorial {

  /* Here we define a type for channels */

  final case class Channel(name: String)

  /* Ordering for Channel */

  implicit val channelOrdering: Ordering[Channel] =
    (x: Channel, y: Channel) => x.name.compare(y.name)

  /* Here we define a type for data */

  final case class Name(first: String, last: String)
  final case class Address(street: String, city: String, state: String, zip: String)
  final case class Entry(name: Name, address: Address, email: String, phone: String)

  /* Here we define a type for patterns */

  sealed trait Pattern                       extends Product with Serializable
  final case class NameMatch(last: String)   extends Pattern
  final case class CityMatch(city: String)   extends Pattern
  final case class StateMatch(state: String) extends Pattern

  class Printer extends ((Seq[Entry]) => Unit) with Serializable {

    def apply(entries: Seq[Entry]): Unit =
      entries.foreach {
        case Entry(name, address, email, phone) =>
          val nameStr = s"${name.last}, ${name.first}"
          val addrStr = s"${address.street}, ${address.city}, ${address.state} ${address.zip}"
          Console.printf(s"""|
                             |=== ENTRY ===
                             |name:    $nameStr
                             |address: $addrStr
                             |email:   $email
                             |phone:   $phone
                             |""".stripMargin)
      }
  }

  def main(args: Array[String]) = {

    implicit val keyValueStoreManager = InMemoryStoreManager[Id]

    val store = keyValueStoreManager.rSpaceStores
    val space = RSpace.create[Id, Channel, Pattern, Entry, Printer](store)

    Console.printf("\nExample One: Let's consume and then produce...\n")

    val cres =
      space
        .consume(
          Seq(Channel("friends")),
          Seq(CityMatch(city = "Crystal Lake")),
          new Printer,
          persist = true
        ) // it should be fine to do that -- type of left side is Nothing (no invalid states)

    assert(cres.isEmpty)

    val pres1 = space.produce(Channel("friends"), alice, persist = false)
    val pres2 = space.produce(Channel("friends"), bob, persist = false)
    val pres3 = space.produce(Channel("friends"), carol, persist = false)

    assert(pres1.nonEmpty)
    assert(pres2.nonEmpty)
    assert(pres3.isEmpty)

    println("Hello, world")
  }
}
