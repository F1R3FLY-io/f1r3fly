package rspacePlusPlus

import cats.Applicative
import cats.implicits._
import com.sun.jna.{Memory, Native, Pointer}
import coop.rchain.models.rspace_plus_plus_types.{
  ActionResult,
  ChannelsProto,
  ConsumeParams,
  ConsumeProto,
  DatumProto,
  DatumsProto,
  FreeMapProto,
  HotStoreStateProto,
  InstallParams,
  JoinProto,
  JoinsProto,
  ProduceCounterMapEntry,
  ProduceProto,
  SoftCheckpointProto,
  SortedSetElement,
  StoreStateContMapEntry,
  StoreStateDataMapEntry,
  StoreStateInstalledContMapEntry,
  StoreStateInstalledJoinsMapEntry,
  StoreStateJoinsMapEntry,
  StoreToMapResult,
  WaitingContinuationProto,
  WaitingContinuationsProto
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
import coop.rchain.rspace.HotStoreState

import scala.collection.immutable.Map
import com.google.protobuf.ByteString
import coop.rchain.rspace.state.RSpaceExporter
import coop.rchain.rspace.state.RSpaceImporter
import coop.rchain.shared.Serialize
import coop.rchain.rspace.history.HistoryReader
import coop.rchain.rspace.HotStoreTrieAction
import coop.rchain.rspace.HotStoreAction
import coop.rchain.rspace.history.History
import coop.rchain.state.TrieNode
import java.nio.ByteBuffer
import scodec.bits.ByteVector
import com.typesafe.scalalogging.Logger
import cats.effect.ContextShift
import scala.concurrent.ExecutionContext
import rspacePlusPlus.state.{RSpacePlusPlusExporter, RSpacePlusPlusImporter}
import rspacePlusPlus.history.{RSpacePlusPlusHistoryReader}
import coop.rchain.rspace.state.exporters.RSpaceExporterItems.StoreItems
import java.nio.file.Path
import coop.rchain.metrics.Metrics
import coop.rchain.rspace.history.HistoryReaderBase
import rspacePlusPlus.history.{RSpacePlusPlusHistoryReaderBase, RSpacePlusPlusHistoryReaderBinary}
import coop.rchain.rspace.serializers.ScodecSerialize.DatumB
import coop.rchain.rspace.serializers.ScodecSerialize.WaitingContinuationB
import coop.rchain.rspace.serializers.ScodecSerialize.JoinsB
import coop.rchain.rspace.trace.COMM
import coop.rchain.models.rspace_plus_plus_types.IOEventProto
import coop.rchain.models.rspace_plus_plus_types.EventProto
import coop.rchain.models.rspace_plus_plus_types.CommProto
import coop.rchain.rspace.RSpaceMetricsSource
import coop.rchain.rspace.concurrent.ConcurrentTwoStepLockF
import rspacePlusPlus.JNAInterfaceLoader.{INSTANCE}
import coop.rchain.rspace.serializers.ScodecSerialize.encodeDatum
import coop.rchain.rspace.serializers.ScodecSerialize.encodeContinuation
import _root_.coop.rchain.rspace.serializers.ScodecSerialize.encodeJoin

// NOTE: Concurrent two step lock NOT implemented
abstract class RSpaceOpsPlusPlus[F[_]: Concurrent: ContextShift: Log: Metrics](
    rspacePointer: Pointer
)(
    implicit
    serializeC: Serialize[Par],
    serializeP: Serialize[BindPattern],
    serializeA: Serialize[ListParWithRandom],
    serializeK: Serialize[TaggedContinuation],
    scheduler: ExecutionContext
) extends ISpacePlusPlus[F, Par, BindPattern, ListParWithRandom, TaggedContinuation] {

  type C = Par;
  type P = BindPattern;
  type A = ListParWithRandom;
  type K = TaggedContinuation;

  type MaybeActionResult = Option[(ContResult[C, P, K], Seq[Result[C, A]])]

  protected[this] val dataLogger: Logger =
    Logger("rspacePlusPlus")

  def historyRepo: HistoryRepository[F, C, P, A, K] = new HistoryRepository[F, C, P, A, K] {

    override def checkpoint(actions: List[HotStoreAction]): F[HistoryRepository[F, C, P, A, K]] = {
      println("\ncheckpoint"); ???
    }

    override def doCheckpoint(
        actions: Seq[HotStoreTrieAction]
    ): F[HistoryRepository[F, C, P, A, K]] = { println("\ndoCheckpoint"); ??? }

    override def reset(root: Blake2b256Hash): F[HistoryRepository[F, C, P, A, K]] = {
      println("\nreset"); ???
    }

    override def history: History[F] = { println("\nhistory"); ??? }

    override def exporter: F[RSpacePlusPlusExporter[F]] =
      for {
        rspaceExporter <- Sync[F].delay {
                           new RSpacePlusPlusExporter[F] {

                             override def getNodes(
                                 startPath: Seq[(Blake2b256Hash, Option[Byte])],
                                 skip: Int,
                                 take: Int
                             ): F[Seq[TrieNode[Blake2b256Hash]]] = {
                               println("getNodes")
                               ???
                             }

                             override def getHistoryItems[Value](
                                 keys: Seq[Blake2b256Hash],
                                 fromBuffer: ByteBuffer => Value
                             ): F[Seq[(Blake2b256Hash, Value)]] = {
                               println("getHistoryItems")
                               ???
                             }

                             override def getDataItems[Value](
                                 keys: Seq[Blake2b256Hash],
                                 fromBuffer: ByteBuffer => Value
                             ): F[Seq[(Blake2b256Hash, Value)]] = {
                               println("getDataItems")
                               ???
                             }

                             override def getRoot: F[Blake2b256Hash] = { println("getRoot"); ??? }

                             def traverseHistory(
                                 startPath: Seq[(Blake2b256Hash, Option[Byte])],
                                 skip: Int,
                                 take: Int,
                                 getFromHistory: ByteVector => F[Option[ByteVector]]
                             ): F[Vector[TrieNode[Blake2b256Hash]]] = {
                               println("traverseHistory")
                               ???
                             }

                             def getHistory[Value](
                                 startPath: Seq[(Blake2b256Hash, Option[Byte])],
                                 skip: Int,
                                 take: Int,
                                 fromBuffer: ByteBuffer => Value
                             )(
                                 implicit m: Sync[F],
                                 l: Log[F]
                             ): F[StoreItems[Blake2b256Hash, Value]] = {
                               println("getHistory")
                               ???
                             }

                             def getData[Value](
                                 startPath: Seq[(Blake2b256Hash, Option[Byte])],
                                 skip: Int,
                                 take: Int,
                                 fromBuffer: ByteBuffer => Value
                             )(
                                 implicit m: Sync[F],
                                 l: Log[F]
                             ): F[StoreItems[Blake2b256Hash, Value]] = {
                               println("getData")
                               ???
                             }

                             def getHistoryAndData[Value](
                                 startPath: Seq[(Blake2b256Hash, Option[Byte])],
                                 skip: Int,
                                 take: Int,
                                 fromBuffer: ByteBuffer => Value
                             )(
                                 implicit m: Sync[F],
                                 l: Log[F]
                             ): F[
                               (
                                   StoreItems[Blake2b256Hash, Value],
                                   StoreItems[Blake2b256Hash, Value]
                               )
                             ] = {
                               println("getHistoryAndData")
                               ???
                             }

                             def writeToDisk[C, P, A, K](
                                 root: Blake2b256Hash,
                                 dirPath: Path,
                                 chunkSize: Int
                             )(
                                 implicit m: Concurrent[F],
                                 l: Log[F]
                             ): F[Unit] = {
                               println("writeToDisk")
                               ???
                             }
                           }
                         }
      } yield rspaceExporter

    override def importer: F[RSpacePlusPlusImporter[F]] =
      for {
        rspaceImporter <- Sync[F].delay {
                           new RSpacePlusPlusImporter[F] {

                             override def setHistoryItems[Value](
                                 data: Seq[(Blake2b256Hash, Value)],
                                 toBuffer: Value => ByteBuffer
                             ): F[Unit] = {
                               println("setHistoryItems")
                               ???
                             }

                             override def setDataItems[Value](
                                 data: Seq[(Blake2b256Hash, Value)],
                                 toBuffer: Value => ByteBuffer
                             ): F[Unit] = {
                               println("setDataItems")
                               ???
                             }

                             override def setRoot(key: Blake2b256Hash): F[Unit] = {
                               println("setRoot")
                               ???
                             }

                             override def getHistoryItem(
                                 hash: Blake2b256Hash
                             ): F[Option[ByteVector]] = {
                               println("getHistoryItem")
                               ???
                             }

                           }
                         }
      } yield rspaceImporter

    override def getHistoryReader(
        stateHash: Blake2b256Hash
    ): F[RSpacePlusPlusHistoryReader[F, Blake2b256Hash, C, P, A, K]] =
      for {
        historyReader <- Sync[F].delay {
                          new RSpacePlusPlusHistoryReader[F, Blake2b256Hash, C, P, A, K] {

                            override def root: Blake2b256Hash = {
                              println("root")
                              ???
                            }

                            override def getData(key: Blake2b256Hash): F[Seq[Datum[A]]] =
                              for {
                                result <- Sync[F].delay {
                                           val stateHashBytes       = stateHash.bytes.toArray
                                           val stateHashBytesLength = stateHashBytes.length
                                           val keyBytes             = key.bytes.toArray
                                           val keyBytesLength       = keyBytes.length

                                           val payloadSize   = stateHashBytesLength.toLong + keyBytesLength.toLong
                                           val payloadMemory = new Memory(payloadSize)

                                           payloadMemory
                                             .write(0, stateHashBytes, 0, stateHashBytesLength)
                                           payloadMemory.write(
                                             stateHashBytesLength.toLong,
                                             keyBytes,
                                             0,
                                             keyBytesLength
                                           )

                                           val getHistoryDataResultPtr = INSTANCE.get_history_data(
                                             rspacePointer,
                                             payloadMemory,
                                             stateHashBytesLength,
                                             keyBytesLength
                                           )

                                           // Not sure is this line is needed
                                           // Need to figure out how to deallocate 'payloadMemory'
                                           payloadMemory.clear()

                                           if (getHistoryDataResultPtr != null) {
                                             val resultByteslength =
                                               getHistoryDataResultPtr.getInt(0)

                                             try {
                                               val resultBytes =
                                                 getHistoryDataResultPtr
                                                   .getByteArray(4, resultByteslength)
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
                                                             channelsHash =
                                                               Blake2b256Hash.fromByteArray(
                                                                 produceEvent.channelHash.toByteArray
                                                               ),
                                                             hash = Blake2b256Hash.fromByteArray(
                                                               produceEvent.hash.toByteArray
                                                             ),
                                                             persistent = produceEvent.persistent
                                                           )
                                                         case None => {
                                                           println("ProduceEvent is None");
                                                           throw new RuntimeException(
                                                             "ProduceEvent is None"
                                                           )
                                                         }
                                                       }
                                                     )
                                                 )

                                               datums
                                             } catch {
                                               case e: Throwable =>
                                                 println(
                                                   "Error during scala getHistoryData operation: " + e
                                                 )
                                                 throw e
                                             } finally {
                                               INSTANCE.deallocate_memory(
                                                 getHistoryDataResultPtr,
                                                 resultByteslength
                                               )
                                             }
                                           } else {
                                             println("getHistoryDataResultPtr is null")
                                             throw new RuntimeException(
                                               "getHistoryDataResultPtr is null"
                                             )
                                           }
                                         }

                              } yield result

                            override def getContinuations(
                                key: Blake2b256Hash
                            ): F[Seq[WaitingContinuation[P, K]]] =
                              for {
                                result <- Sync[F].delay {
                                           val stateHashBytes       = stateHash.bytes.toArray
                                           val stateHashBytesLength = stateHashBytes.length
                                           val keyBytes             = key.bytes.toArray
                                           val keyBytesLength       = keyBytes.length

                                           val payloadSize   = stateHashBytesLength.toLong + keyBytesLength.toLong
                                           val payloadMemory = new Memory(payloadSize)

                                           payloadMemory
                                             .write(0, stateHashBytes, 0, stateHashBytesLength)
                                           payloadMemory.write(
                                             stateHashBytesLength.toLong,
                                             keyBytes,
                                             0,
                                             keyBytesLength
                                           )

                                           val getHistoryWaitingContinuationResultPtr =
                                             INSTANCE.get_history_waiting_continuations(
                                               rspacePointer,
                                               payloadMemory,
                                               stateHashBytesLength,
                                               keyBytesLength
                                             )

                                           // Not sure if these lines are needed
                                           // Need to figure out how to deallocate each memory instance
                                           payloadMemory.clear()

                                           if (getHistoryWaitingContinuationResultPtr != null) {
                                             val resultByteslength =
                                               getHistoryWaitingContinuationResultPtr.getInt(0)

                                             try {
                                               val resultBytes =
                                                 getHistoryWaitingContinuationResultPtr
                                                   .getByteArray(4, resultByteslength)
                                               val wksProto =
                                                 WaitingContinuationsProto.parseFrom(
                                                   resultBytes
                                                 )
                                               val wksProtos = wksProto.wks

                                               val wks: Seq[WaitingContinuation[P, K]] =
                                                 wksProtos
                                                   .map(
                                                     wk =>
                                                       WaitingContinuation(
                                                         patterns = wk.patterns,
                                                         continuation = wk.continuation.get,
                                                         persist = wk.persist,
                                                         peeks = wk.peeks.map(_.value).to[SortedSet],
                                                         source = wk.source match {
                                                           case Some(consumeEvent) =>
                                                             Consume(
                                                               channelsHashes =
                                                                 consumeEvent.channelHashes.map(
                                                                   bs =>
                                                                     Blake2b256Hash
                                                                       .fromByteArray(
                                                                         bs.toByteArray
                                                                       )
                                                                 ),
                                                               hash = Blake2b256Hash.fromByteArray(
                                                                 consumeEvent.hash.toByteArray
                                                               ),
                                                               persistent = consumeEvent.persistent
                                                             )
                                                           case None => {
                                                             println("ConsumeEvent is None");
                                                             throw new RuntimeException(
                                                               "ConsumeEvent is None"
                                                             )
                                                           }
                                                         }
                                                       )
                                                   )

                                               wks
                                             } catch {
                                               case e: Throwable =>
                                                 println(
                                                   "Error during scala getHistoryWaitingContinuations operation: " + e
                                                 )
                                                 throw e
                                             } finally {
                                               INSTANCE.deallocate_memory(
                                                 getHistoryWaitingContinuationResultPtr,
                                                 resultByteslength
                                               )
                                             }
                                           } else {
                                             println(
                                               "getHistoryWaitingContinuationResultPtr is null"
                                             )
                                             throw new RuntimeException(
                                               "getHistoryWaitingContinuationResultPtr is null"
                                             )
                                           }
                                         }
                              } yield result

                            override def getJoins(key: Blake2b256Hash): F[Seq[Seq[C]]] =
                              for {
                                result <- Sync[F].delay {
                                           val stateHashBytes       = stateHash.bytes.toArray
                                           val stateHashBytesLength = stateHashBytes.length
                                           val keyBytes             = key.bytes.toArray
                                           val keyBytesLength       = keyBytes.length

                                           val payloadSize   = stateHashBytesLength.toLong + keyBytesLength.toLong
                                           val payloadMemory = new Memory(payloadSize)

                                           payloadMemory
                                             .write(0, stateHashBytes, 0, stateHashBytesLength)
                                           payloadMemory.write(
                                             stateHashBytesLength.toLong,
                                             keyBytes,
                                             0,
                                             keyBytesLength
                                           )

                                           val getHistoryJoinsResultPtr =
                                             INSTANCE.get_history_joins(
                                               rspacePointer,
                                               payloadMemory,
                                               stateHashBytesLength,
                                               keyBytesLength
                                             )

                                           // Not sure is this line is needed
                                           // Need to figure out how to deallocate 'payloadMemory'
                                           payloadMemory.clear()

                                           if (getHistoryJoinsResultPtr != null) {
                                             val resultByteslength =
                                               getHistoryJoinsResultPtr.getInt(0)

                                             try {
                                               val resultBytes =
                                                 getHistoryJoinsResultPtr
                                                   .getByteArray(4, resultByteslength)
                                               val joinsProto  = JoinsProto.parseFrom(resultBytes)
                                               val joinsProtos = joinsProto.joins

                                               val joins: Seq[Seq[C]] =
                                                 joinsProtos.map(
                                                   join => join.join
                                                 )

                                               joins
                                             } catch {
                                               case e: Throwable =>
                                                 println(
                                                   "Error during scala getHistoryJoins operation: " + e
                                                 )
                                                 throw e
                                             } finally {
                                               INSTANCE.deallocate_memory(
                                                 getHistoryJoinsResultPtr,
                                                 resultByteslength
                                               )
                                             }
                                           } else {
                                             println("getHistoryJoinsResultPtr is null")
                                             throw new RuntimeException(
                                               "getHistoryJoinsResultPtr is null"
                                             )
                                           }
                                         }

                              } yield result

                            override def base: RSpacePlusPlusHistoryReaderBase[F, C, P, A, K] = {
                              println("base")
                              ???
                            }

                            // See rspace/src/main/scala/coop/rchain/rspace/history/syntax/HistoryReaderSyntax.scala
                            override def readerBinary
                                : RSpacePlusPlusHistoryReaderBinary[F, C, P, A, K] =
                              new RSpacePlusPlusHistoryReaderBinary[F, C, P, A, K] {
                                override def getData(key: Blake2b256Hash): F[Seq[DatumB[A]]] =
                                  for {
                                    result <- Sync[F].delay {
                                               val stateHashBytes       = stateHash.bytes.toArray
                                               val stateHashBytesLength = stateHashBytes.length
                                               val keyBytes             = key.bytes.toArray
                                               val keyBytesLength       = keyBytes.length

                                               val payloadSize   = stateHashBytesLength.toLong + keyBytesLength.toLong
                                               val payloadMemory = new Memory(payloadSize)

                                               payloadMemory
                                                 .write(0, stateHashBytes, 0, stateHashBytesLength)
                                               payloadMemory.write(
                                                 stateHashBytesLength.toLong,
                                                 keyBytes,
                                                 0,
                                                 keyBytesLength
                                               )

                                               val getHistoryDataResultPtr =
                                                 INSTANCE.get_history_data(
                                                   rspacePointer,
                                                   payloadMemory,
                                                   stateHashBytesLength,
                                                   keyBytesLength
                                                 )

                                               // Not sure is this line is needed
                                               // Need to figure out how to deallocate 'payloadMemory'
                                               payloadMemory.clear()

                                               if (getHistoryDataResultPtr != null) {
                                                 val resultByteslength =
                                                   getHistoryDataResultPtr.getInt(0)

                                                 try {
                                                   val resultBytes =
                                                     getHistoryDataResultPtr
                                                       .getByteArray(4, resultByteslength)
                                                   val datumsProto =
                                                     DatumsProto.parseFrom(resultBytes)
                                                   val datumsProtos = datumsProto.datums

                                                   val datums: Seq[DatumB[A]] =
                                                     datumsProtos.map(
                                                       datum => {
                                                         val d = Datum(
                                                           datum.a.get,
                                                           datum.persist,
                                                           source = datum.source match {
                                                             case Some(produceEvent) =>
                                                               Produce(
                                                                 channelsHash =
                                                                   Blake2b256Hash.fromByteArray(
                                                                     produceEvent.channelHash.toByteArray
                                                                   ),
                                                                 hash =
                                                                   Blake2b256Hash.fromByteArray(
                                                                     produceEvent.hash.toByteArray
                                                                   ),
                                                                 persistent =
                                                                   produceEvent.persistent
                                                               )
                                                             case None => {
                                                               println("ProduceEvent is None");
                                                               throw new RuntimeException(
                                                                 "ProduceEvent is None"
                                                               )
                                                             }
                                                           }
                                                         )
                                                         DatumB(
                                                           d,
                                                           encodeDatum(d)
                                                         )
                                                       }
                                                     )

                                                   datums
                                                 } catch {
                                                   case e: Throwable =>
                                                     println(
                                                       "Error during scala getHistoryBinaryData operation: " + e
                                                     )
                                                     throw e
                                                 } finally {
                                                   INSTANCE.deallocate_memory(
                                                     getHistoryDataResultPtr,
                                                     resultByteslength
                                                   )
                                                 }
                                               } else {
                                                 println("getHistoryBinaryDataResultPtr is null")
                                                 throw new RuntimeException(
                                                   "getHistoryBinaryDataResultPtr is null"
                                                 )
                                               }
                                             }

                                  } yield result

                                override def getContinuations(
                                    key: Blake2b256Hash
                                ): F[Seq[WaitingContinuationB[P, K]]] =
                                  for {
                                    result <- Sync[F].delay {
                                               val stateHashBytes       = stateHash.bytes.toArray
                                               val stateHashBytesLength = stateHashBytes.length
                                               val keyBytes             = key.bytes.toArray
                                               val keyBytesLength       = keyBytes.length

                                               val payloadSize   = stateHashBytesLength.toLong + keyBytesLength.toLong
                                               val payloadMemory = new Memory(payloadSize)

                                               payloadMemory
                                                 .write(0, stateHashBytes, 0, stateHashBytesLength)
                                               payloadMemory.write(
                                                 stateHashBytesLength.toLong,
                                                 keyBytes,
                                                 0,
                                                 keyBytesLength
                                               )

                                               val getHistoryWaitingContinuationResultPtr =
                                                 INSTANCE.get_history_waiting_continuations(
                                                   rspacePointer,
                                                   payloadMemory,
                                                   stateHashBytesLength,
                                                   keyBytesLength
                                                 )

                                               // Not sure if these lines are needed
                                               // Need to figure out how to deallocate each memory instance
                                               payloadMemory.clear()

                                               if (getHistoryWaitingContinuationResultPtr != null) {
                                                 val resultByteslength =
                                                   getHistoryWaitingContinuationResultPtr.getInt(0)

                                                 try {
                                                   val resultBytes =
                                                     getHistoryWaitingContinuationResultPtr
                                                       .getByteArray(4, resultByteslength)
                                                   val wksProto =
                                                     WaitingContinuationsProto.parseFrom(
                                                       resultBytes
                                                     )
                                                   val wksProtos = wksProto.wks

                                                   val wks: Seq[WaitingContinuationB[P, K]] =
                                                     wksProtos.map(
                                                       wk => {
                                                         val waitingContinuation =
                                                           WaitingContinuation(
                                                             patterns = wk.patterns,
                                                             continuation = wk.continuation.get,
                                                             persist = wk.persist,
                                                             peeks =
                                                               wk.peeks.map(_.value).to[SortedSet],
                                                             source = wk.source match {
                                                               case Some(consumeEvent) =>
                                                                 Consume(
                                                                   channelsHashes =
                                                                     consumeEvent.channelHashes.map(
                                                                       bs =>
                                                                         Blake2b256Hash
                                                                           .fromByteArray(
                                                                             bs.toByteArray
                                                                           )
                                                                     ),
                                                                   hash =
                                                                     Blake2b256Hash.fromByteArray(
                                                                       consumeEvent.hash.toByteArray
                                                                     ),
                                                                   persistent =
                                                                     consumeEvent.persistent
                                                                 )
                                                               case None => {
                                                                 println("ConsumeEvent is None");
                                                                 throw new RuntimeException(
                                                                   "ConsumeEvent is None"
                                                                 )
                                                               }
                                                             }
                                                           )
                                                         WaitingContinuationB(
                                                           waitingContinuation,
                                                           encodeContinuation(waitingContinuation)
                                                         )
                                                       }
                                                     )

                                                   wks
                                                 } catch {
                                                   case e: Throwable =>
                                                     println(
                                                       "Error during scala getHistoryBinaryWaitingContinuations operation: " + e
                                                     )
                                                     throw e
                                                 } finally {
                                                   INSTANCE.deallocate_memory(
                                                     getHistoryWaitingContinuationResultPtr,
                                                     resultByteslength
                                                   )
                                                 }
                                               } else {
                                                 println(
                                                   "getHistoryBinaryWaitingContinuationResultPtr is null"
                                                 )
                                                 throw new RuntimeException(
                                                   "getHistoryBinaryWaitingContinuationResultPtr is null"
                                                 )
                                               }
                                             }
                                  } yield result

                                override def getJoins(key: Blake2b256Hash): F[Seq[JoinsB[C]]] =
                                  for {
                                    result <- Sync[F].delay {
                                               val stateHashBytes       = stateHash.bytes.toArray
                                               val stateHashBytesLength = stateHashBytes.length
                                               val keyBytes             = key.bytes.toArray
                                               val keyBytesLength       = keyBytes.length

                                               val payloadSize   = stateHashBytesLength.toLong + keyBytesLength.toLong
                                               val payloadMemory = new Memory(payloadSize)

                                               payloadMemory
                                                 .write(0, stateHashBytes, 0, stateHashBytesLength)
                                               payloadMemory.write(
                                                 stateHashBytesLength.toLong,
                                                 keyBytes,
                                                 0,
                                                 keyBytesLength
                                               )

                                               val getHistoryJoinsResultPtr =
                                                 INSTANCE.get_history_joins(
                                                   rspacePointer,
                                                   payloadMemory,
                                                   stateHashBytesLength,
                                                   keyBytesLength
                                                 )

                                               // Not sure is this line is needed
                                               // Need to figure out how to deallocate 'payloadMemory'
                                               payloadMemory.clear()

                                               if (getHistoryJoinsResultPtr != null) {
                                                 val resultByteslength =
                                                   getHistoryJoinsResultPtr.getInt(0)

                                                 try {
                                                   val resultBytes =
                                                     getHistoryJoinsResultPtr
                                                       .getByteArray(4, resultByteslength)
                                                   val joinsProto =
                                                     JoinsProto.parseFrom(resultBytes)
                                                   val joinsProtos = joinsProto.joins

                                                   val joins: Seq[JoinsB[C]] =
                                                     joinsProtos.map(
                                                       join => {
                                                         val j = join.join
                                                         JoinsB(j, encodeJoin(j))
                                                       }
                                                     )

                                                   joins
                                                 } catch {
                                                   case e: Throwable =>
                                                     println(
                                                       "Error during scala getHistoryBinaryJoins operation: " + e
                                                     )
                                                     throw e
                                                 } finally {
                                                   INSTANCE.deallocate_memory(
                                                     getHistoryJoinsResultPtr,
                                                     resultByteslength
                                                   )
                                                 }
                                               } else {
                                                 println("getHistoryBinaryJoinsResultPtr is null")
                                                 throw new RuntimeException(
                                                   "getHistoryBinaryJoinsResultPtr is null"
                                                 )
                                               }
                                             }

                                  } yield result
                              }

                          }
                        }
      } yield historyReader

    override def getSerializeC: Serialize[C] = serializeC

    override def root: Blake2b256Hash = { println("\nroot"); ??? }

  }

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
                                 println("ProduceEvent is None");
                                 throw new RuntimeException("ProduceEvent is None")
                               }
                             }
                           )
                       )

                     datums
                   } catch {
                     case e: Throwable =>
                       println("Error during scala getData operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(getDataResultPtr, resultByteslength)
                   }
                 } else {
                   println("getDataResultPtr is null")
                   throw new RuntimeException("getDataResultPtr is null")
                 }
               }
    } yield result

  def getWaitingContinuations(channels: Seq[C]): F[Seq[WaitingContinuation[P, K]]] =
    for {
      result <- Sync[F].delay {
                 val channelsProto = ChannelsProto(channels)
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
                                 println("ConsumeEvent is None");
                                 throw new RuntimeException("ConsumeEvent is None")
                               }
                             }
                           )
                       )

                     wks
                   } catch {
                     case e: Throwable =>
                       println("Error during scala getWaitingContinuations operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(getWaitingContinuationResultPtr, resultByteslength)
                   }
                 } else {
                   println("getWaitingContinuationResultPtr is null")
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

                   } catch {
                     case e: Throwable =>
                       println("Error during scala getJoins operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(getJoinsResultPtr, resultByteslength)
                   }
                 } else {
                   println("getJoinsResultPtr is null")
                   throw new RuntimeException("getJoinsResultPtr is null")
                 }
               }
    } yield result

  override def consume(
      channels: Seq[C],
      patterns: Seq[P],
      continuation: K,
      persist: Boolean,
      peeks: SortedSet[Int] = SortedSet.empty
      // peeks: SortedSet[Int]
  ): F[MaybeActionResult] =
    ContextShift[F].evalOn(scheduler) {
      for {
        // result <- consumeLockF(channels) {
        consumeRef <- Sync[F].delay(Consume(channels, patterns, continuation, persist))
        result     <- lockedConsume(channels, patterns, continuation, persist, peeks, consumeRef)
        //  }
      } yield result
    }

  protected[this] def lockedConsume(
      channels: Seq[C],
      patterns: Seq[P],
      continuation: K,
      persist: Boolean,
      peeks: SortedSet[Int],
      consumeRef: Consume
  ): F[MaybeActionResult]

  override def produce(channel: C, data: A, persist: Boolean): F[MaybeActionResult] =
    ContextShift[F].evalOn(scheduler) {
      for {
        // result <- produceLockF(channel) {
        produceRef <- Sync[F].delay(Produce(channel, data, persist))
        result     <- lockedProduce(channel, data, persist, produceRef)
        //  }

      } yield result
    }

  protected[this] def lockedProduce(
      channel: C,
      data: A,
      persist: Boolean,
      produceRef: Produce
  ): F[MaybeActionResult]

  override def install(
      channels: Seq[C],
      patterns: Seq[P],
      continuation: K
  ): F[Option[(K, Seq[A])]] =
    for {
      // result <- installLockF(channels) {
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
                   println(
                     "Error during install operation: Installing can be done only on startup"
                   )
                   throw new RuntimeException("Installing can be done only on startup")
                 } else {
                   None
                 }
               }
      //  }

    } yield result

  def toMap: F[Map[Seq[C], Row[P, A, K]]] =
    for {
      result <- Sync[F].delay {
                 val toMapPtr = INSTANCE.to_map(rspacePointer)

                 if (toMapPtr != null) {
                   val length = toMapPtr.getInt(0)

                   try {
                     val resultBytes = toMapPtr.getByteArray(4, length)
                     val toMapResult = StoreToMapResult.parseFrom(resultBytes)

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
                                         channelsHash = Blake2b256Hash.fromByteArray(
                                           produceEvent.channelHash.toByteArray
                                         ),
                                         hash = Blake2b256Hash.fromByteArray(
                                           produceEvent.hash.toByteArray
                                         ),
                                         persistent = produceEvent.persistent
                                       )
                                     case None => {
                                       println("ProduceEvent is None");
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
                                         hash = Blake2b256Hash.fromByteArray(
                                           consumeEvent.hash.toByteArray
                                         ),
                                         persistent = consumeEvent.persistent
                                       )
                                     case None => {
                                       println("ConsumeEvent is None");
                                       throw new RuntimeException("ConsumeEvent is None")
                                     }
                                   }
                                 )
                             )
                           )
                         case None => {
                           println("Row is None")
                           Log[F].debug("Row is None"); throw new RuntimeException("Row is None")
                         }
                       }
                       (key, value)
                     }.toMap

                     map
                   } catch {
                     case e: Throwable =>
                       println("Error during scala toMap operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(toMapPtr, length)
                   }
                 } else {
                   println("toMapPtr is null")
                   throw new RuntimeException("toMapPtr is null")
                 }

               }
    } yield result

  override def reset(root: Blake2b256Hash): F[Unit] =
    for {
      _ <- Sync[F].delay {
            // println("\nhit scala reset, root: " + root)
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

  override def clear(): F[Unit] =
    Applicative[F].pure { INSTANCE.space_clear(rspacePointer) }

  override def createSoftCheckpoint(): F[SoftCheckpoint[C, P, A, K]] = {
    for {
      result <- Sync[F].delay {
                 val softCheckpointPtr = INSTANCE.create_soft_checkpoint(rspacePointer)

                 if (softCheckpointPtr != null) {
                   val length = softCheckpointPtr.getInt(0)

                   try {
                     val softCheckpointBytes = softCheckpointPtr.getByteArray(4, length)
                     val softCheckpointProto = SoftCheckpointProto.parseFrom(softCheckpointBytes)
                     val storeStateProto     = softCheckpointProto.cacheSnapshot.get

                     val continuationsMap = storeStateProto.continuations.map { mapEntry =>
                       val key = mapEntry.key
                       val value = mapEntry.value.map { wkProto =>
                         WaitingContinuation(
                           patterns = wkProto.patterns,
                           continuation = wkProto.continuation.get,
                           persist = wkProto.persist,
                           peeks = wkProto.peeks.map(_.value).to[SortedSet],
                           source = wkProto.source match {
                             case Some(consumeEvent) =>
                               Consume(
                                 channelsHashes = consumeEvent.channelHashes.map(
                                   bs => Blake2b256Hash.fromByteArray(bs.toByteArray)
                                 ),
                                 hash = Blake2b256Hash.fromByteArray(consumeEvent.hash.toByteArray),
                                 persistent = consumeEvent.persistent
                               )
                             case None => {
                               println("ConsumeEvent is None");
                               throw new RuntimeException("ConsumeEvent is None")
                             }
                           }
                         )
                       }

                       (key, value)
                     }.toMap

                     val installedContinuationsMap = storeStateProto.installedContinuations.map {
                       mapEntry =>
                         val key = mapEntry.key
                         val value = mapEntry.value match {
                           case Some(wkProto) => {
                             WaitingContinuation(
                               patterns = wkProto.patterns,
                               continuation = wkProto.continuation.get,
                               persist = wkProto.persist,
                               peeks = wkProto.peeks.map(_.value).to[SortedSet],
                               source = wkProto.source match {
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
                                   println("ConsumeEvent is None");
                                   throw new RuntimeException("ConsumeEvent is None")
                                 }
                               }
                             )
                           }
                           case None => {
                             println("wkProto is None");
                             throw new RuntimeException("wkProto is None")
                           }
                         }

                         (key, value)
                     }.toMap

                     val datumsMap = storeStateProto.data.map { mapEntry =>
                       val key = mapEntry.key.get
                       val value = mapEntry.value.map(
                         datum =>
                           Datum(
                             a = datum.a.get,
                             persist = datum.persist,
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
                                 println("ProduceEvent is None")
                                 Log[F].debug("ProduceEvent is None");
                                 throw new RuntimeException("ProduceEvent is None")
                               }
                             }
                           )
                       )

                       (key, value)
                     }.toMap

                     val joinsMap = storeStateProto.joins.map { mapEntry =>
                       val key = mapEntry.key.get
                       val value = mapEntry.value.map(
                         joinProto => joinProto.join
                       )

                       (key, value)
                     }.toMap

                     val installedJoinsMap = storeStateProto.installedJoins.map { mapEntry =>
                       val key = mapEntry.key.get
                       val value = mapEntry.value.map(
                         joinProto => joinProto.join
                       )

                       (key, value)
                     }.toMap

                     val checkpointLog: Seq[Event] = softCheckpointProto.log.map { eventProto =>
                       eventProto match {
                         case EventProto(EventProto.EventType.Comm(commProto)) => {
                           val consumeProto = commProto.consume.get
                           val produces = commProto.produces.map { produceProto =>
                             Produce(
                               channelsHash =
                                 Blake2b256Hash.fromByteArray(produceProto.channelHash.toByteArray),
                               hash = Blake2b256Hash.fromByteArray(produceProto.hash.toByteArray),
                               persistent = produceProto.persistent
                             )
                           }
                           val peeks = commProto.peeks.map(_.value).to[SortedSet]
                           val timesRepeated = commProto.timesRepeated.map { entry =>
                             val produceProto = entry.key.get
                             val produce = Produce(
                               channelsHash =
                                 Blake2b256Hash.fromByteArray(produceProto.channelHash.toByteArray),
                               hash = Blake2b256Hash.fromByteArray(produceProto.hash.toByteArray),
                               persistent = produceProto.persistent
                             )
                             produce -> entry.value
                           }.toMap
                           COMM(
                             consume = Consume(
                               channelsHashes = consumeProto.channelHashes
                                 .map(bs => Blake2b256Hash.fromByteArray(bs.toByteArray)),
                               hash = Blake2b256Hash.fromByteArray(consumeProto.hash.toByteArray),
                               persistent = consumeProto.persistent
                             ),
                             produces = produces,
                             peeks = peeks,
                             timesRepeated = timesRepeated
                           )
                         }

                         case EventProto(EventProto.EventType.IoEvent(ioEvent)) =>
                           ioEvent.ioEventType match {
                             case IOEventProto.IoEventType.Produce(produceProto) => {
                               Produce(
                                 channelsHash = Blake2b256Hash.fromByteArray(
                                   produceProto.channelHash.toByteArray
                                 ),
                                 hash = Blake2b256Hash.fromByteArray(produceProto.hash.toByteArray),
                                 persistent = produceProto.persistent
                               )
                             }

                             case IOEventProto.IoEventType.Consume(consumeProto) => {
                               Consume(
                                 channelsHashes = consumeProto.channelHashes
                                   .map(bs => Blake2b256Hash.fromByteArray(bs.toByteArray)),
                                 hash = Blake2b256Hash.fromByteArray(consumeProto.hash.toByteArray),
                                 persistent = consumeProto.persistent
                               )
                             }

                             case _ => throw new RuntimeException("unkown IOEventType")
                           }

                         case _ => throw new RuntimeException("unkown EventType")
                       }
                     }

                     val produceCounterMap = softCheckpointProto.produceCounter.map { mapEntry =>
                       val keyProto = mapEntry.key.get
                       val produce = Produce(
                         channelsHash =
                           Blake2b256Hash.fromByteArray(keyProto.channelHash.toByteArray),
                         hash = Blake2b256Hash.fromByteArray(keyProto.hash.toByteArray),
                         persistent = keyProto.persistent
                       )

                       val value = mapEntry.value
                       (produce, value)
                     }.toMap

                     val cacheSnapshot: HotStoreState[C, P, A, K] =
                       HotStoreState(
                         continuationsMap,
                         installedContinuationsMap,
                         datumsMap,
                         joinsMap,
                         installedJoinsMap
                       )

                     SoftCheckpoint(cacheSnapshot, checkpointLog, produceCounterMap)
                   } catch {
                     case e: Throwable =>
                       println("Error during scala createSoftCheckpoint operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(softCheckpointPtr, length)
                   }
                 } else {
                   println("softCheckpointPtr is null")
                   throw new RuntimeException("softCheckpointPtr is null")
                 }

               }
    } yield result
  }

  override def revertToSoftCheckpoint(checkpoint: SoftCheckpoint[C, P, A, K]): F[Unit] =
    for {
      _ <- Sync[F].delay {
            // println("\nhit scala revertToSoftCheckpoint")
            val cacheSnapshot = checkpoint.cacheSnapshot

            val continuationsMapEntries          = Seq.empty
            val installedContinuationsMapEntries = Seq.empty
            val datumsMapEntries                 = Seq.empty
            val joinsMapEntries                  = Seq.empty
            val installedJoinsMapEntries         = Seq.empty

            cacheSnapshot.continuations.map { mapEntry =>
              val key = mapEntry._1
              val value = mapEntry._2.map { wk =>
                WaitingContinuationProto(
                  patterns = wk.patterns,
                  continuation = Some(wk.continuation),
                  persist = wk.persist,
                  peeks = wk.peeks.map(elem => SortedSetElement(elem)).toSeq,
                  source = Some(
                    ConsumeProto(
                      channelHashes = wk.source.channelsHashes.map(
                        channelHash => channelHash.toByteString
                      ),
                      hash = wk.source.hash.toByteString,
                      persistent = wk.source.persistent
                    )
                  )
                )
              }

              continuationsMapEntries :+ StoreStateContMapEntry(key, value)
            }

            cacheSnapshot.installedContinuations.map { mapEntry =>
              val key = mapEntry._1
              val wk  = mapEntry._2
              val value =
                WaitingContinuationProto(
                  patterns = wk.patterns,
                  continuation = Some(wk.continuation),
                  persist = wk.persist,
                  peeks = wk.peeks.map(elem => SortedSetElement(elem)).toSeq,
                  source = Some(
                    ConsumeProto(
                      channelHashes = wk.source.channelsHashes.map(
                        channelHash => channelHash.toByteString
                      ),
                      hash = wk.source.hash.toByteString,
                      persistent = wk.source.persistent
                    )
                  )
                )

              installedContinuationsMapEntries :+ StoreStateInstalledContMapEntry(key, Some(value))
            }

            cacheSnapshot.data.map { mapEntry =>
              val key = mapEntry._1
              val value = mapEntry._2.map { datum =>
                DatumProto(
                  a = Some(datum.a),
                  persist = datum.persist,
                  source = Some(
                    ProduceProto(
                      channelHash = datum.source.channelsHash.toByteString,
                      hash = datum.source.hash.toByteString,
                      persistent = datum.source.persistent
                    )
                  )
                )
              }

              datumsMapEntries :+ StoreStateDataMapEntry(Some(key), value)
            }

            cacheSnapshot.joins.map { mapEntry =>
              val key = mapEntry._1
              val value = mapEntry._2.map(
                join => JoinProto(join)
              )

              joinsMapEntries :+ StoreStateJoinsMapEntry(Some(key), value)
            }

            cacheSnapshot.installedJoins.map { mapEntry =>
              val key = mapEntry._1
              val value = mapEntry._2.map(
                join => JoinProto(join)
              )

              installedJoinsMapEntries :+ StoreStateInstalledJoinsMapEntry(Some(key), value)
            }

            val hotStoreStateProto = HotStoreStateProto(
              continuationsMapEntries,
              installedContinuationsMapEntries,
              datumsMapEntries,
              joinsMapEntries,
              installedJoinsMapEntries
            )

            val produceCounterMapEntries = Seq.empty
            val produceCounterMap        = checkpoint.produceCounter

            produceCounterMap.map { mapEntry =>
              val produce = mapEntry._1
              val produceProto = ProduceProto(
                produce.channelsHash.toByteString,
                produce.hash.toByteString,
                produce.persistent
              )

              produceCounterMapEntries :+ ProduceCounterMapEntry(Some(produceProto), mapEntry._2)
            }

            val logProto = checkpoint.log.map {
              case comm: COMM =>
                val consumeProto = ConsumeProto(
                  channelHashes = comm.consume.channelsHashes.map(_.toByteString),
                  hash = comm.consume.hash.toByteString,
                  persistent = comm.consume.persistent
                )
                val producesProto = comm.produces.map { produce =>
                  ProduceProto(
                    channelHash = produce.channelsHash.toByteString,
                    hash = produce.hash.toByteString,
                    persistent = produce.persistent
                  )
                }
                val peeksProto = comm.peeks.map(SortedSetElement(_)).toSeq
                val timesRepeatedProto = comm.timesRepeated.map {
                  case (produce, count) =>
                    val produceProto = ProduceProto(
                      channelHash = produce.channelsHash.toByteString,
                      hash = produce.hash.toByteString,
                      persistent = produce.persistent
                    )
                    ProduceCounterMapEntry(Some(produceProto), count)
                }.toSeq
                EventProto(
                  eventType = EventProto.EventType.Comm(
                    CommProto(
                      consume = Some(consumeProto),
                      produces = producesProto,
                      peeks = peeksProto,
                      timesRepeated = timesRepeatedProto
                    )
                  )
                )
              case produce: Produce =>
                EventProto(
                  eventType = EventProto.EventType.IoEvent(
                    IOEventProto(
                      ioEventType = IOEventProto.IoEventType.Produce(
                        ProduceProto(
                          channelHash = produce.channelsHash.toByteString,
                          hash = produce.hash.toByteString,
                          persistent = produce.persistent
                        )
                      )
                    )
                  )
                )
              case consume: Consume =>
                EventProto(
                  eventType = EventProto.EventType.IoEvent(
                    IOEventProto(
                      ioEventType = IOEventProto.IoEventType.Consume(
                        ConsumeProto(
                          channelHashes = consume.channelsHashes.map(_.toByteString),
                          hash = consume.hash.toByteString,
                          persistent = consume.persistent
                        )
                      )
                    )
                  )
                )
            }

            val softCheckpointProto =
              SoftCheckpointProto(Some(hotStoreStateProto), logProto, produceCounterMapEntries)
            val softCheckpointProtoBytes = softCheckpointProto.toByteArray

            val payloadMemory = new Memory(softCheckpointProtoBytes.length.toLong)
            payloadMemory.write(0, softCheckpointProtoBytes, 0, softCheckpointProtoBytes.length)

            val _ = INSTANCE.revert_to_soft_checkpoint(
              rspacePointer,
              payloadMemory,
              softCheckpointProtoBytes.length
            )

            // Not sure if these lines are needed
            // Need to figure out how to deallocate each memory instance
            payloadMemory.clear()
          }
    } yield ()
}
