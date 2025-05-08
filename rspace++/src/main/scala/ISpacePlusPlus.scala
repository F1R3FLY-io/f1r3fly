package rspacePlusPlus

import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.{Checkpoint, SoftCheckpoint}
import coop.rchain.rspace.internal.{Datum, Row, WaitingContinuation}

// See rspace/src/main/scala/coop/rchain/rspace/ISpace.scala
/** The interface for RSpace
  *
  * @tparam C a type representing a channel
  * @tparam P a type representing a pattern
  * @tparam A a type representing an arbitrary piece of data and match result
  * @tparam K a type representing a continuation
  */
trait ISpacePlusPlus[F[_], C, P, A, K] extends TuplespacePlusPlus[F, C, P, A, K] {

  /** Creates a checkpoint.
    *
    * @return A [[Checkpoint]]
    */
  def createCheckpoint(): F[Checkpoint]

  /** Resets the store to the given root.
    *
    * @param root A BLAKE2b256 Hash representing the checkpoint
    */
  def reset(root: Blake2b256Hash): F[Unit]

  def getData(channel: C): F[Seq[Datum[A]]]

  def getWaitingContinuations(channels: Seq[C]): F[Seq[WaitingContinuation[P, K]]]

  def getJoins(channel: C): F[Seq[Seq[C]]]

  /** Clears the store.  Does not affect the history trie.
    */
  def clear(): F[Unit]

  // TODO: this should not be exposed
  def toMap: F[Map[Seq[C], Row[P, A, K]]]

  /**
    Allows to create a "soft" checkpoint which doesn't persist the checkpointed data into history.
    This operation is significantly faster than {@link #createCheckpoint()} because the computationally
    expensive operation of creating the history trie is avoided.
    */
  def createSoftCheckpoint(): F[SoftCheckpoint[C, P, A, K]]

  /**
    Reverts the ISpace to the state checkpointed using {@link #createSoftCheckpoint()}
    */
  def revertToSoftCheckpoint(checkpoint: SoftCheckpoint[C, P, A, K]): F[Unit]
}
