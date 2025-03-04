package rspacePlusPlus

import coop.rchain.rspace.internal.MultisetMultiMap
import coop.rchain.rspace.trace.{COMM, Event, IOEvent}

package object trace {

  type Log = Seq[Event]

  type ReplayData = MultisetMultiMap[IOEvent, COMM]
}
