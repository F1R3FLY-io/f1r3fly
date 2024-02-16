package rspacePlusPlus

import cats.Applicative
import cats.implicits._
import com.sun.jna.{Memory, Native, Pointer}
import coop.rchain.models.rspace_plus_plus_types.{
  ActionResult,
  ConsumeParams,
  FreeMapProto,
  InstallParams,
  SortedSetElement,
  ToMapResult
}
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import scala.collection.SortedSet
import coop.rchain.rspace.{ContResult, Result}
import coop.rchain.rspace.trace.{Consume, Produce}
import coop.rchain.rspace.{Checkpoint, SoftCheckpoint}
import coop.rchain.rspace.internal.{Datum, Row, WaitingContinuation}
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.shared.Log
import cats.effect.{Concurrent, Sync}
import coop.rchain.models.rspace_plus_plus_types.CheckpointProto
import coop.rchain.rspace.trace.Event

/**
  * This class contains predefined types for Channel, Pattern, Data, and Continuation - RhoTypes
  * These types (C, P, A, K) MUST MATCH the corresponding types on the Rust side in 'rspace_rhotypes/lib.rs'
  */
class RSpacePlusPlus_RhoTypes[F[_]: Concurrent: Log]
    extends RSpaceOpsPlusPlus[F]
    with ISpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation] {

  type C = Par;
  type P = BindPattern;
  type A = ListParWithRandom;
  type K = TaggedContinuation;

  type MaybeActionResult =
    Option[
      (
          ContResult[C, P, K],
          Seq[Result[C, A]]
      )
    ]

  // val jnaLibraryPath = System.getProperty("jna.library.path")
  // println(s"Current jna.library.path: $jnaLibraryPath")

  val INSTANCE: JNAInterface =
    Native
      .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
      .asInstanceOf[JNAInterface]

  val rspacePointer: Pointer = space_new()

  private def space_new(): Pointer =
    INSTANCE.space_new()

  def print(): Unit =
    INSTANCE.space_print(rspacePointer)

  // ISpacePlusPlus trait function
  def clear(): F[Unit] =
    Applicative[F].pure { INSTANCE.space_clear(rspacePointer) }

  @SuppressWarnings(Array("org.wartremover.warts.Throw"))
  def spatialMatchResult(target: C, pattern: C): F[Option[(Map[Int, Par], Unit)]] =
    for {
      result <- Sync[F].delay {
                 val targetBytes        = target.toByteArray
                 val targetBytesLength  = targetBytes.length
                 val patternBytes       = pattern.toByteArray
                 val patternBytesLength = patternBytes.length

                 val payloadSize   = targetBytesLength.toLong + patternBytesLength.toLong
                 val payloadMemory = new Memory(payloadSize)

                 payloadMemory.write(0, targetBytes, 0, targetBytesLength)
                 payloadMemory.write(targetBytesLength.toLong, patternBytes, 0, patternBytesLength)

                 val spatialMatchResultPtr = INSTANCE.spatial_match_result(
                   payloadMemory,
                   targetBytesLength,
                   patternBytesLength
                 )

                 // Not sure is this line is needed
                 // Need to figure out how to deallocate 'payloadMemory'
                 payloadMemory.clear()

                 if (spatialMatchResultPtr != null) {
                   val resultByteslength = spatialMatchResultPtr.getInt(0)

                   try {
                     val resultBytes  = spatialMatchResultPtr.getByteArray(4, resultByteslength)
                     val freeMapProto = FreeMapProto.parseFrom(resultBytes)

                     Some((freeMapProto.entries, ()))
                   } finally {
                     INSTANCE.deallocate_memory(spatialMatchResultPtr, resultByteslength)
                   }
                 } else {
                   None
                 }
               }
    } yield result

  // TuplespacePlusPlus trait functions

  @SuppressWarnings(Array("org.wartremover.warts.Throw"))
  def produce(channel: C, data: A, persist: Boolean): F[MaybeActionResult] =
    for {
      result <- Sync[F].delay {
                 val channelBytes       = channel.toByteArray
                 val channelBytesLength = channelBytes.length
                 val dataBytes          = data.toByteArray
                 val dataBytesLength    = dataBytes.length

                 val payloadSize   = channelBytesLength.toLong + dataBytesLength.toLong
                 val payloadMemory = new Memory(payloadSize)

                 payloadMemory.write(0, channelBytes, 0, channelBytesLength)
                 payloadMemory.write(channelBytesLength.toLong, dataBytes, 0, dataBytesLength)

                 val produceResultPtr = INSTANCE.produce(
                   rspacePointer,
                   payloadMemory,
                   channelBytesLength,
                   dataBytesLength,
                   persist
                 )

                 // Not sure is this line is needed
                 // Need to figure out how to deallocate 'payloadMemory'
                 payloadMemory.clear()

                 if (produceResultPtr != null) {
                   val resultByteslength = produceResultPtr.getInt(0)

                   try {
                     val resultBytes  = produceResultPtr.getByteArray(4, resultByteslength)
                     val actionResult = ActionResult.parseFrom(resultBytes)
                     val contResult = actionResult.contResult.getOrElse({
                       Log[F].debug("ContResult is None")
                       throw new RuntimeException("ContResult is None")
                     })
                     val results = actionResult.results

                     Some(
                       (
                         ContResult(
                           continuation = contResult.continuation.getOrElse({
                             Log[F].debug("ContResult is None")
                             throw new RuntimeException("ContResult is None")
                           }),
                           persistent = contResult.persistent,
                           channels = contResult.channels,
                           patterns = contResult.patterns,
                           peek = contResult.peek
                         ),
                         results.map(
                           r =>
                             Result(
                               channel = r.channel.getOrElse({
                                 Log[F].debug("Channel is None in Seq[Result]")
                                 throw new RuntimeException("Channel is None in Seq[Result]")
                               }),
                               matchedDatum = r.matchedDatum.getOrElse({
                                 Log[F].debug("MatchedDatum is None in Seq[Result]")
                                 throw new RuntimeException("MatchedDatum is None in Seq[Result]")
                               }),
                               removedDatum = r.removedDatum.getOrElse({
                                 Log[F].debug("RemovedDatum is None in Seq[Result]")
                                 throw new RuntimeException("RemovedDatum is None in Seq[Result]")
                               }),
                               persistent = r.persistent
                             )
                         )
                       )
                     )
                   } finally {
                     INSTANCE.deallocate_memory(produceResultPtr, resultByteslength)
                   }
                 } else {
                   None
                 }
               }
    } yield result

  @SuppressWarnings(Array("org.wartremover.warts.Throw"))
  def consume(
      channels: Seq[C],
      patterns: Seq[P],
      continuation: K,
      persist: Boolean,
      peeks: SortedSet[Int] = SortedSet.empty
  ): F[MaybeActionResult] =
    for {
      result <- Sync[F].delay {
                 val consumeParams = ConsumeParams(
                   channels,
                   patterns,
                   Some(continuation),
                   persist,
                   peeks.map(SortedSetElement(_)).toSeq
                 )
                 val consumeParamsBytes = consumeParams.toByteArray

                 val payloadMemory = new Memory(consumeParamsBytes.length.toLong)
                 payloadMemory.write(0, consumeParamsBytes, 0, consumeParamsBytes.length)

                 val consumeResultPtr = INSTANCE.consume(
                   rspacePointer,
                   payloadMemory,
                   consumeParamsBytes.length
                 )

                 // Not sure if these lines are needed
                 // Need to figure out how to deallocate each memory instance
                 payloadMemory.clear()

                 if (consumeResultPtr != null) {
                   val resultByteslength = consumeResultPtr.getInt(0)

                   try {
                     val resultBytes  = consumeResultPtr.getByteArray(4, resultByteslength)
                     val actionResult = ActionResult.parseFrom(resultBytes)
                     val contResult = actionResult.contResult.getOrElse({
                       Log[F].debug("ContResult is None")
                       throw new RuntimeException("ContResult is None")
                     })
                     val results = actionResult.results

                     Some(
                       (
                         ContResult(
                           continuation = contResult.continuation.getOrElse({
                             Log[F].debug("ContResult is None")
                             throw new RuntimeException("ContResult is None")
                           }),
                           persistent = contResult.persistent,
                           channels = contResult.channels,
                           patterns = contResult.patterns,
                           peek = contResult.peek
                         ),
                         results.map(
                           r =>
                             Result(
                               channel = r.channel.getOrElse({
                                 Log[F].debug("Channel is None in Seq[Result]")
                                 throw new RuntimeException("Channel is None in Seq[Result]")
                               }),
                               matchedDatum = r.matchedDatum.getOrElse({
                                 Log[F].debug("MatchedDatum is None in Seq[Result]")
                                 throw new RuntimeException("MatchedDatum is None in Seq[Result]")
                               }),
                               removedDatum = r.removedDatum.getOrElse({
                                 Log[F].debug("RemovedDatum is None in Seq[Result]")
                                 throw new RuntimeException("RemovedDatum is None in Seq[Result]")
                               }),
                               persistent = r.persistent
                             )
                         )
                       )
                     )
                   } finally {
                     INSTANCE.deallocate_memory(consumeResultPtr, resultByteslength)
                   }
                 } else {
                   None
                 }
               }
    } yield result

  @SuppressWarnings(Array("org.wartremover.warts.Throw"))
  def install(channels: Seq[C], patterns: Seq[P], continuation: K): F[Option[(K, Seq[A])]] =
    for {
      result <- Sync[F].delay {
                 val installParams = InstallParams(
                   channels,
                   patterns,
                   Some(continuation)
                 )
                 val installParamsBytes = installParams.toByteArray

                 val payloadMemory = new Memory(installParamsBytes.length.toLong)
                 payloadMemory.write(0, installParamsBytes, 0, installParamsBytes.length)

                 val installResultPtr = INSTANCE.install(
                   rspacePointer,
                   payloadMemory,
                   installParamsBytes.length
                 )

                 // Not sure if these lines are needed
                 // Need to figure out how to deallocate each memory instance
                 payloadMemory.clear()

                 if (installResultPtr != null) {
                   throw new RuntimeException("Installing can be done only on startup")
                 } else {
                   None
                 }
               }
    } yield result

  // ISpacePlusPlus trait functions

  def createCheckpoint(): F[Checkpoint] =
    for {
      result <- Sync[F].delay {
                 val checkpointResultPtr = INSTANCE.create_checkpoint(
                   rspacePointer
                 )

                 if (checkpointResultPtr != null) {
                   val resultByteslength = checkpointResultPtr.getInt(0)

                   try {
                     val resultBytes    = checkpointResultPtr.getByteArray(4, resultByteslength)
                     val checkpoint     = CheckpointProto.parseFrom(resultBytes)
                     val checkpointRoot = checkpoint.root

                     Checkpoint(
                       root = Blake2b256Hash.fromByteArray(checkpointRoot.toByteArray),
                       log = Seq.empty[Event]
                     )

                   } finally {
                     INSTANCE.deallocate_memory(checkpointResultPtr, resultByteslength)
                   }
                 } else {
                   throw new RuntimeException("Checkpoint pointer from rust was null")
                 }
               }
    } yield result

  def reset(root: Blake2b256Hash): F[Unit] = {
    println("reset")
    ???
  }

  def getData(channel: C): F[Seq[Datum[A]]] = {
    println("getData")
    ???
  }

  def getWaitingContinuations(channels: Seq[C]): F[Seq[WaitingContinuation[P, K]]] = {
    println("getWaitingContinuations")
    ???
  }

  def getJoins(channel: C): F[Seq[Seq[C]]] = {
    println("getJoins")
    ???
  }

  @SuppressWarnings(Array("org.wartremover.warts.Throw"))
  def toMap: F[Map[Seq[C], Row[P, A, K]]] = {
    val byteArrayPtr = INSTANCE.to_map(rspacePointer)
    val length       = byteArrayPtr.getInt(0)
    val resultBytes  = byteArrayPtr.getByteArray(4, length)
    val toMapResult  = ToMapResult.parseFrom(resultBytes)

    Sync[F].delay {
      val map = toMapResult.mapEntries.map { mapEntry =>
        val key = mapEntry.key
        val value = mapEntry.value match {
          case Some(row) =>
            Row(
              data = row.data.map(
                datum =>
                  Datum(
                    a = datum.a.get,
                    persist = datum.persist,
                    source = datum.source match {
                      case Some(produceEvent) =>
                        Produce(
                          channelsHash =
                            Blake2b256Hash.fromByteArray(produceEvent.channelHash.toByteArray),
                          hash = Blake2b256Hash.fromByteArray(produceEvent.hash.toByteArray),
                          persistent = produceEvent.persistent
                        )
                      case None => {
                        Log[F].debug("ProduceEvent is None");
                        throw new RuntimeException("ProduceEvent is None")
                      }
                    }
                  )
              ),
              wks = row.wks.map(
                wk =>
                  WaitingContinuation(
                    patterns = wk.patterns,
                    continuation = wk.continuation.get,
                    persist = wk.persist,
                    peeks = wk.peeks.map(_.value).to[SortedSet],
                    source = wk.source match {
                      case Some(consumeEvent) =>
                        Consume(
                          channelsHashes = consumeEvent.channelHashes.map(
                            bs => Blake2b256Hash.fromByteArray(bs.toByteArray)
                          ),
                          hash = Blake2b256Hash.fromByteArray(consumeEvent.hash.toByteArray),
                          persistent = consumeEvent.persistent
                        )
                      case None => {
                        Log[F].debug("ConsumeEvent is None");
                        throw new RuntimeException("ConsumeEvent is None")
                      }
                    }
                  )
              )
            )
          case None => {
            Log[F].debug("Row is None"); throw new RuntimeException("Row is None")
          }
        }
        (key, value)
      }.toMap

      map
    }
  }

  def createSoftCheckpoint(): F[SoftCheckpoint[C, P, A, K]] = {
    println("createSoftCheckpoint")
    ???
  }

  def revertToSoftCheckpoint(checkpoint: SoftCheckpoint[C, P, A, K]): F[Unit] = {
    println("revertToSoftCheckpoint")
    ???
  }

  def spawn: F[ISpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay(new RSpacePlusPlus_RhoTypes[F]())
}

object RSpacePlusPlus_RhoTypes {
  def create[F[_]: Concurrent: Log](
      ): F[RSpacePlusPlus_RhoTypes[F]] =
    Sync[F].delay(new RSpacePlusPlus_RhoTypes[F]())

  def createWithReplay[F[_]: Concurrent: Log, C, P, A, K](
      ): F[(RSpacePlusPlus_RhoTypes[F], ReplayRSpacePlusPlus[F, C, P, A, K])] =
    Sync[F].delay((new RSpacePlusPlus_RhoTypes[F](), new ReplayRSpacePlusPlus[F, C, P, A, K]()))
}
