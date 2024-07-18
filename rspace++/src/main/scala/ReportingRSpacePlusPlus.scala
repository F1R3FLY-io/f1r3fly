package rspacePlusPlus

import cats.effect.{Concurrent, ContextShift, Sync}
import cats.syntax.all._
import com.sun.jna.{Memory, Pointer}
import coop.rchain.metrics.Metrics
import coop.rchain.models.rspace_plus_plus_types._
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.rspace.internal.{ConsumeCandidate, WaitingContinuation}
import coop.rchain.rspace.{Checkpoint, ContResult, HotStoreState, Result, SoftCheckpoint}
import rspacePlusPlus.ReportingRSpacePlusPlus.{
  ReportingComm,
  ReportingConsume,
  ReportingEvent,
  ReportingProduce
}
//import coop.rchain.rspace.ReportingRspace.{
//  ReportingComm,
//  ReportingConsume,
//  ReportingEvent,
//  ReportingProduce
//}
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.trace.{COMM, Consume, Produce}
import coop.rchain.shared.{Log, Serialize}
import rspacePlusPlus.JNAInterfaceLoader.INSTANCE

import scala.collection.SortedSet
import scala.concurrent.ExecutionContext

object ReportingRSpacePlusPlus {
  sealed trait ReportingEvent

  final case class ReportingProduce[C, A](channel: C, data: A) extends ReportingEvent

  final case class ReportingConsume[C, P, K](
      channels: Seq[C],
      patterns: Seq[P],
      continuation: K,
      peeks: Seq[Int]
  ) extends ReportingEvent

  final case class ReportingComm[C, P, A, K](
      consume: ReportingConsume[C, P, K],
      produces: Seq[ReportingProduce[C, A]]
  ) extends ReportingEvent

  def create[F[_]: Concurrent: ContextShift: Log: Metrics, C, P, A, K](
      storePath: String
  )(
      implicit
      serializeC: Serialize[Par],
      serializeP: Serialize[BindPattern],
      serializeA: Serialize[ListParWithRandom],
      serializeK: Serialize[TaggedContinuation],
      scheduler: ExecutionContext
  ): F[ReportingRSpacePlusPlus[F, C, P, A, K]] =
    Sync[F].delay {
      val rspacePointer = INSTANCE.space_new(storePath);
      new ReportingRSpacePlusPlus[F, C, P, A, K](rspacePointer)
    }
}

class ReportingRSpacePlusPlus[F[_]: Concurrent: ContextShift: Log: Metrics, C, P, A, K](
    rspacePointer: Pointer
)(
    implicit
    serializeC: Serialize[Par],
    serializeP: Serialize[BindPattern],
    serializeA: Serialize[ListParWithRandom],
    serializeK: Serialize[TaggedContinuation],
    scheduler: ExecutionContext
) extends ReplayRSpacePlusPlus[F, C, P, A, K](rspacePointer) {

  override def getRspacePointer: Pointer = rspacePointer

  protected override def logF: Log[F] = Log[F]

  def getReport: F[Seq[Seq[ReportingEvent]]] =
    for {
      pointer <- Sync[F].delay(INSTANCE.reporting_get_report(rspacePointer))
      reportProto <- Sync[F].delay(
                      ReportingEventNestedListProto
                        .parseFrom(pointer.getByteArray(4, pointer.getInt(0)))
                        .events
                        .map(_.events)
                    )
      reports = reportProto.map(_.map { x =>
        mapReportinEventProto(x.eventType)
      })
    } yield reports

  private def mapReportinEventProto(x: ReportingEventProto.EventType): ReportingEvent =
    x match {
      case ReportingEventProto.EventType.Consume(consume) =>
        ReportingConsume(
          consume.channels,
          consume.patterns,
          consume.continuation.getOrElse(throw new RuntimeException("Continuation is None")),
          consume.peeks.map(_.value)
        )
      case ReportingEventProto.EventType.Produce(produce) =>
        ReportingProduce(
          produce.channel,
          produce.data
        )
      case ReportingEventProto.EventType.Comm(ReportingCommProto(Some(consume), produces)) =>
        ReportingComm(
          ReportingConsume(
            consume.channels,
            consume.patterns,
            consume.continuation.getOrElse(throw new RuntimeException("Continuation is None")),
            consume.peeks.map(_.value)
          ),
          produces.map(
            p =>
              ReportingProduce(
                p.channel.getOrElse(throw new RuntimeException("channel is None")),
                p.data.getOrElse(throw new RuntimeException("data is None"))
              )
          )
        )
    }

  protected[this] override def lockedConsume(
      channels: Seq[ReportingRSpacePlusPlus.this.C],
      patterns: Seq[ReportingRSpacePlusPlus.this.P],
      continuation: ReportingRSpacePlusPlus.this.K,
      persist: Boolean,
      peeks: SortedSet[Int],
      consumeRef: Consume
  ): F[MaybeActionResult] =
    for {
      result <- Sync[F].delay {
                 // println(s"\nhit consume in scala ${}");
                 //  dataLogger.debug("hit consume in scala")

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

                 val consumeResultPtr = INSTANCE.reporting_log_consume(
                   rspacePointer,
                   payloadMemory,
                   consumeParamsBytes.length
                 )

                 // Not sure if these lines are needed
                 // Need to figure out how to deallocate each memory instance
                 payloadMemory.clear()

                 //  val jsonString = consumeResultPtr.getString(0)
                 //  println("\njsonString: " + jsonString)

                 //  if (jsonString != "") {
                 if (consumeResultPtr != null) {
                   val resultByteslength = consumeResultPtr.getInt(0)
                   try {
                     // println("\nresultByteslength: " + resultByteslength)
                     val resultBytes = consumeResultPtr.getByteArray(4, resultByteslength)
                     // println("resultBytes length: " + resultBytes.length)
                     val actionResult = ActionResult.parseFrom(resultBytes)

                     //  val actionResult = ActionResult.parseFrom(jsonString.getBytes())

                     val contResult = actionResult.contResult.get
                     val results    = actionResult.results

                     Some(
                       (
                         ContResult(
                           continuation = contResult.continuation.get,
                           persistent = contResult.persistent,
                           channels = contResult.channels,
                           patterns = contResult.patterns,
                           peek = contResult.peek
                         ),
                         results.map(
                           r =>
                             Result(
                               channel = r.channel.get,
                               matchedDatum = r.matchedDatum.get,
                               removedDatum = r.removedDatum.get,
                               persistent = r.persistent
                             )
                         )
                       )
                     )
                   } catch {
                     case e: Throwable =>
                       println("Error during scala consume operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(consumeResultPtr, resultByteslength)
                   }
                 } else {
                   None
                 }
               }
    } yield result

  protected override def logComm(
      dataCandidates: Seq[
        ConsumeCandidate[ReportingRSpacePlusPlus.this.C, ReportingRSpacePlusPlus.this.A]
      ],
      channels: Seq[ReportingRSpacePlusPlus.this.C],
      wk: WaitingContinuation[ReportingRSpacePlusPlus.this.P, ReportingRSpacePlusPlus.this.K],
      comm: COMM,
      label: String
  ): F[COMM] =
    for {
      result <- Sync[F].delay {

                 val commParams = CommParams(
                   Some(
                     CommProto(
                       Some(
                         ConsumeProto(
                           comm.consume.channelsHashes.map(_.toByteString),
                           comm.consume.hash.toByteString,
                           comm.consume.persistent
                         )
                       ),
                       comm.produces.map(
                         p =>
                           ProduceProto(
                             p.channelsHash.toByteString,
                             p.hash.toByteString,
                             p.persistent
                           )
                       ),
                       comm.peeks.map(p => new SortedSetElement(p)).toSeq,
                       comm.timesRepeated.map[ProduceCounterMapEntry, Seq[ProduceCounterMapEntry]] {
                         case (produce, times) =>
                           val produceProto = ProduceProto(
                             channelHash = produce.channelsHash.toByteString,
                             hash = produce.hash.toByteString,
                             persistent = produce.persistent
                           )
                           ProduceCounterMapEntry(Some(produceProto), times)
                       }(collection.breakOut)
                     )
                   ),
                   dataCandidates = dataCandidates.map(
                     dc =>
                       ConsumeCandidateProto(
                         channels = Some(dc.channel),
                         datum = Some(
                           DatumProto(
                             Some(dc.datum.a),
                             dc.datum.persist,
                             Some(
                               ProduceProto(
                                 channelHash = dc.datum.source.channelsHash.toByteString,
                                 hash = dc.datum.source.hash.toByteString,
                                 persistent = dc.datum.source.persistent
                               )
                             )
                           )
                         ),
                         removedDatum = Some(dc.removedDatum),
                         dataIndex = dc.datumIndex
                       )
                   ),
                   channels = channels,
                   continuation = Some(
                     WaitingContinuationProto(
                       patterns = wk.patterns,
                       continuation = Some(wk.continuation),
                       persist = wk.persist,
                       peeks = wk.peeks.map(p => new SortedSetElement(p)).toSeq
                     )
                   ),
                   label = label
                 )

                 val commParamsBytes = commParams.toByteArray
                 val payloadMemory   = new Memory(commParamsBytes.length.toLong)
                 payloadMemory.write(0, commParamsBytes, 0, commParamsBytes.length)

                 INSTANCE.reporting_log_comm(
                   rspacePointer,
                   payloadMemory,
                   commParamsBytes.length
                 )
               }
    } yield comm

  protected[this] override def lockedProduce(
      channel: ReportingRSpacePlusPlus.this.C,
      data: ReportingRSpacePlusPlus.this.A,
      persist: Boolean,
      produceRef: Produce
  ): F[MaybeActionResult] =
    for {
      result <- Sync[F].delay {
                 //  println("\nHit scala produce, data: " + data)
                 val channelBytes       = channel.toByteArray
                 val channelBytesLength = channelBytes.length
                 val dataBytes          = data.toByteArray
                 val dataBytesLength    = dataBytes.length

                 val payloadSize   = channelBytesLength.toLong + dataBytesLength.toLong
                 val payloadMemory = new Memory(payloadSize)

                 payloadMemory.write(0, channelBytes, 0, channelBytesLength)
                 payloadMemory.write(channelBytesLength.toLong, dataBytes, 0, dataBytesLength)

                 val produceResultPtr = INSTANCE.reporting_log_produce(
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
                     val contResult   = actionResult.contResult.get
                     val results      = actionResult.results

                     Some(
                       (
                         ContResult(
                           continuation = contResult.continuation.get,
                           persistent = contResult.persistent,
                           channels = contResult.channels,
                           patterns = contResult.patterns,
                           peek = contResult.peek
                         ),
                         results.map(
                           r =>
                             Result(
                               channel = r.channel.get,
                               matchedDatum = r.matchedDatum.get,
                               removedDatum = r.removedDatum.get,
                               persistent = r.persistent
                             )
                         )
                       )
                     )
                   } catch {
                     case e: Throwable =>
                       println("Error during scala produce operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(produceResultPtr, resultByteslength)
                   }
                 } else {
                   None
                 }
               }
    } yield result

  // overrides and changes the behaviour of this.createCheckpoint()
  protected override def internalCallCreateCheckpoint(): Pointer =
    INSTANCE.reporting_create_checkpoint(
      rspacePointer
    )

  // overrides and changes the behaviour of this.createSoftCheckpoint()
  protected override def internalCallCreateSoftCheckpoint(): Pointer =
    INSTANCE.reporting_create_soft_checkpoint(
      rspacePointer
    )
}
