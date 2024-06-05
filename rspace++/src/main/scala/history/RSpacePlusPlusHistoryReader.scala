package rspacePlusPlus.history

import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.internal._
import coop.rchain.rspace.serializers.ScodecSerialize.{DatumB, JoinsB, WaitingContinuationB}
import scodec.bits.ByteVector

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryReader.scala
/**
  * Reader for particular history (state verified on blockchain)
  *
  * @tparam F effect type
  * @tparam Key type for hash of a channel
  * @tparam C type for Channel => this is Par
  * @tparam P type for Pattern => this is BindPattern
  * @tparam A type for Abstraction => this is ListParWithRandom
  * @tparam K type for Continuation => this is TaggedContinuation
  */
trait RSpacePlusPlusHistoryReader[F[_], Key, C, P, A, K] {

  // Get current root which reader reads from
  def root: Key

  def getDataProj[R](key: Key)(proj: (Datum[A], ByteVector) => R): F[Seq[R]] = ???

  def getContinuationsProj[R](key: Key)(
      proj: (WaitingContinuation[P, K], ByteVector) => R
  ): F[Seq[R]] = ???

  def getJoinsProj[R](key: Key)(proj: (Seq[C], ByteVector) => R): F[Seq[R]] = ???

  /**
    * Defaults
    */
  def getData(key: Key): F[Seq[Datum[A]]]

  def getContinuations(key: Key): F[Seq[WaitingContinuation[P, K]]]

  def getJoins(key: Key): F[Seq[Seq[C]]]

  /**
    * Get reader which accepts non-serialized and hashed keys
    */
  def base: RSpacePlusPlusHistoryReaderBase[F, C, P, A, K]

  // See rspace/src/main/scala/coop/rchain/rspace/history/syntax/HistoryReaderSyntax.scala
  def readerBinary: RSpacePlusPlusHistoryReaderBinary[F, C, P, A, K]
}

/**
  * History reader base, version of a reader which accepts non-serialized and hashed keys
  */
trait RSpacePlusPlusHistoryReaderBase[F[_], C, P, A, K] {
  def getDataProj[R](key: C): ((Datum[A], ByteVector) => R) => F[Seq[R]] = ???

  def getContinuationsProj[R](
      key: Seq[C]
  ): ((WaitingContinuation[P, K], ByteVector) => R) => F[Seq[R]] = ???

  def getJoinsProj[R](key: C): ((Seq[C], ByteVector) => R) => F[Seq[R]] = ???

  /**
    * Defaults
    */
  def getData(key: C): F[Seq[Datum[A]]]

  def getContinuations(key: Seq[C]): F[Seq[WaitingContinuation[P, K]]]

  def getJoins(key: C): F[Seq[Seq[C]]]
}

/**
  * History reader with binary data included in result
  */
trait RSpacePlusPlusHistoryReaderBinary[F[_], C, P, A, K] {
  def getData(key: Blake2b256Hash): F[Seq[DatumB[A]]]

  def getContinuations(key: Blake2b256Hash): F[Seq[WaitingContinuationB[P, K]]]

  def getJoins(key: Blake2b256Hash): F[Seq[JoinsB[C]]]
}
