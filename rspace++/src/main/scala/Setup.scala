package rspacePlusPlus

import firefly.rtypes.{Address, Entry, Name}

final case class Setup(
    cityMatchCase: String,
    nameMatchCase: String,
    stateMatchCase: String,
    alice: Entry,
    bob: Entry,
    carol: Entry,
    dan: Entry,
    erin: Entry
)

object Setup {
  def apply(): Setup = {
    // Alice
    val aliceName    = Name("Alice", "Lincoln")
    val aliceAddress = Address("777 Ford St", "Crystal Lake", "Idaho", "223322")
    val alice        = Entry(Some(aliceName), Some(aliceAddress), "alicel@ringworld.net", "787-555-1212")

    // Bob
    val bobName    = Name("Bob", "Lahblah")
    val bobAddress = Address("1000 Main St", "Crystal Lake", "Idaho", "223322")
    val bob        = Entry(Some(bobName), Some(bobAddress), "blablah@tenex.net", "698-555-1212")

    // Carol
    val carolName    = Name("Carol", "Lahblah")
    val carolAddress = Address("22 Goldwater Way", "Herbert", "Nevada", "334433")
    val carol        = Entry(Some(carolName), Some(carolAddress), "carol@blablah.org", "232-555-1212")

    // Dan
    val danName    = Name("Dan", "Walters")
    val danAddress = Address("40 Shady Lane", "Crystal Lake", "Idaho", "223322")
    val dan        = Entry(Some(danName), Some(danAddress), "deejwalters@sdf.lonestar.org", "444-555-1212")

    // Erin
    val erinName    = Name("Erin", "Rush")
    val erinAddress = Address("23 Market St.", "Peony", "Idaho", "224422")
    val erin        = Entry(Some(erinName), Some(erinAddress), "erush@lasttraintogoa.net", "333-555-1212")

    Setup(
      "Crystal Lake",
      "Lahblah",
      "Idaho",
      alice,
      bob,
      carol,
      dan,
      erin
    )
  }
}
