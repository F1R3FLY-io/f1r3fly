package rspacePlusPlus

import cats.Applicative
import cats.implicits._
import com.sun.jna.{Memory, Native, Pointer}
import coop.rchain.models.rspace_plus_plus_types.{
  ActionResult,
  ConsumeParams,
  DatumsProto,
  FreeMapProto,
  InstallParams,
  JoinsProto,
  SortedSetElement,
  StoreToMapResult,
  WaitingContinuationsProto,
	ChannelProto
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
class RSpacePlusPlus_RhoTypes[F[_]: Concurrent: Log](rspacePointer: Pointer)
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

  def print(): Unit =
    INSTANCE.space_print(rspacePointer)

  // ISpacePlusPlus trait function
  def clear(): F[Unit] =
    Applicative[F].pure { INSTANCE.space_clear(rspacePointer) }

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

  def reset(root: Blake2b256Hash): F[Unit] =
    for {
      _ <- Sync[F].delay {
            val rootBytes = root.bytes.toArray

            val rootMemory = new Memory(rootBytes.length.toLong)
            rootMemory.write(0, rootBytes, 0, rootBytes.length)

            val _ = INSTANCE.reset(
              rspacePointer,
              rootMemory,
              rootBytes.length
            )

            // Not sure if these lines are needed
            // Need to figure out how to deallocate each memory instance
            rootMemory.clear()
          }
    } yield ()

  def getData(channel: C): F[Seq[Datum[A]]] =
    for {
      result <- Sync[F].delay {
                 val channelBytes = channel.toByteArray

                 val payloadMemory = new Memory(channelBytes.length.toLong)
                 payloadMemory.write(0, channelBytes, 0, channelBytes.length)

                 val getDataResultPtr = INSTANCE.get_data(
                   rspacePointer,
                   payloadMemory,
                   channelBytes.length
                 )

                 // Not sure if these lines are needed
                 // Need to figure out how to deallocate each memory instance
                 payloadMemory.clear()

                 if (getDataResultPtr != null) {
                   val resultByteslength = getDataResultPtr.getInt(0)

                   try {
                     val resultBytes  = getDataResultPtr.getByteArray(4, resultByteslength)
                     val datumsProto  = DatumsProto.parseFrom(resultBytes)
                     val datumsProtos = datumsProto.datums

                     val datums: Seq[Datum[A]] =
                       datumsProtos.map(
                         datum =>
                           Datum(
                             datum.a.get,
                             datum.persist,
                             source = datum.source match {
                               case Some(produceEvent) =>
                                 Produce(
                                   channelsHash = Blake2b256Hash.fromByteArray(
                                     produceEvent.channelHash.toByteArray
                                   ),
                                   hash =
                                     Blake2b256Hash.fromByteArray(produceEvent.hash.toByteArray),
                                   persistent = produceEvent.persistent
                                 )
                               case None => {
                                 Log[F].debug("ProduceEvent is None");
                                 throw new RuntimeException("ProduceEvent is None")
                               }
                             }
                           )
                       )

                     datums
                   } finally {
                     INSTANCE.deallocate_memory(getDataResultPtr, resultByteslength)
                   }
                 } else {
                   throw new RuntimeException("getDataResultPtr is null")
                 }
               }
    } yield result

  def getWaitingContinuations(channels: Seq[C]): F[Seq[WaitingContinuation[P, K]]] =
    for {
      result <- Sync[F].delay {
                 val channelsProto = ChannelProto(channels)
                 val channelsBytes = channelsProto.toByteArray

                 val payloadMemory = new Memory(channelsBytes.length.toLong)
                 payloadMemory.write(0, channelsBytes, 0, channelsBytes.length)

                 val getWaitingContinuationResultPtr = INSTANCE.get_waiting_continuations(
                   rspacePointer,
                   payloadMemory,
                   channelsBytes.length
                 )

                 // Not sure if these lines are needed
                 // Need to figure out how to deallocate each memory instance
                 payloadMemory.clear()

                 if (getWaitingContinuationResultPtr != null) {
                   val resultByteslength = getWaitingContinuationResultPtr.getInt(0)

                   try {
                     val resultBytes =
                       getWaitingContinuationResultPtr.getByteArray(4, resultByteslength)
                     val wksProto  = WaitingContinuationsProto.parseFrom(resultBytes)
                     val wksProtos = wksProto.wks

                     val wks: Seq[WaitingContinuation[P, K]] =
                       wksProtos.map(
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
                                   hash =
                                     Blake2b256Hash.fromByteArray(consumeEvent.hash.toByteArray),
                                   persistent = consumeEvent.persistent
                                 )
                               case None => {
                                 Log[F].debug("ConsumeEvent is None");
                                 throw new RuntimeException("ConsumeEvent is None")
                               }
                             }
                           )
                       )

                     wks
                   } finally {
                     INSTANCE.deallocate_memory(getWaitingContinuationResultPtr, resultByteslength)
                   }
                 } else {
                   throw new RuntimeException("getWaitingContinuationResultPtr is null")
                 }
               }
    } yield result

  def getJoins(channel: C): F[Seq[Seq[C]]] =
    for {
      result <- Sync[F].delay {
                 val channelBytes = channel.toByteArray

                 val payloadMemory = new Memory(channelBytes.length.toLong)
                 payloadMemory.write(0, channelBytes, 0, channelBytes.length)

                 val getJoinsResultPtr = INSTANCE.get_joins(
                   rspacePointer,
                   payloadMemory,
                   channelBytes.length
                 )

                 // Not sure if these lines are needed
                 // Need to figure out how to deallocate each memory instance
                 payloadMemory.clear()

                 if (getJoinsResultPtr != null) {
                   val resultByteslength = getJoinsResultPtr.getInt(0)

                   try {
                     val resultBytes = getJoinsResultPtr.getByteArray(4, resultByteslength)
                     val joinsProto  = JoinsProto.parseFrom(resultBytes)
                     val joinsProtos = joinsProto.joins

                     val joins: Seq[Seq[C]] =
                       joinsProtos.map(
                         join => join.join
                       )

                     joins

                   } finally {
                     INSTANCE.deallocate_memory(getJoinsResultPtr, resultByteslength)
                   }
                 } else {
                   throw new RuntimeException("getJoinsResultPtr is null")
                 }
               }
    } yield result

  @SuppressWarnings(Array("org.wartremover.warts.Throw"))
  def toMap: F[Map[Seq[C], Row[P, A, K]]] = {
    val byteArrayPtr = INSTANCE.to_map(rspacePointer)
    val length       = byteArrayPtr.getInt(0)
    val resultBytes  = byteArrayPtr.getByteArray(4, length)
    val toMapResult  = StoreToMapResult.parseFrom(resultBytes)

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
    for {
      result <- Sync[F].delay {
                 val rspace = INSTANCE.spawn(
                   rspacePointer
                 )

                 new RSpacePlusPlus_RhoTypes[F](rspace)
               }
    } yield result
}

object RSpacePlusPlus_RhoTypes {
  def create[F[_]: Concurrent: Log](
      ): F[RSpacePlusPlus_RhoTypes[F]] =
    Sync[F].delay {
      val INSTANCE: JNAInterface =
        Native
          .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
          .asInstanceOf[JNAInterface]

      val rspacePointer = INSTANCE.space_new();
      new RSpacePlusPlus_RhoTypes[F](rspacePointer)
    }

  def createWithReplay[F[_]: Concurrent: Log, C, P, A, K](
      ): F[(RSpacePlusPlus_RhoTypes[F], ReplayRSpacePlusPlus[F, C, P, A, K])] =
    Sync[F].delay {
      val INSTANCE: JNAInterface =
        Native
          .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
          .asInstanceOf[JNAInterface]

      val rspacePointer = INSTANCE.space_new();
      (
        new RSpacePlusPlus_RhoTypes[F](rspacePointer),
        new ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer)
      )
    }
}
