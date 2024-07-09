package rspacePlusPlus

import cats.effect.{Concurrent, ContextShift, Sync}
import cats.syntax.all._
import com.sun.jna.{Memory, Native, Pointer}
import coop.rchain.metrics.Metrics
import coop.rchain.models.{BindPattern, ListParWithRandom, Par, TaggedContinuation}
import coop.rchain.models.rspace_plus_plus_types._
import coop.rchain.rspace.{Checkpoint, ContResult, Result}
import coop.rchain.rspace.ReportingRspace.{
  ReportingComm,
  ReportingConsume,
  ReportingEvent,
  ReportingProduce
}
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.trace.{COMM, Consume, Produce}
import coop.rchain.shared.{Log, Serialize}
import rspacePlusPlus.JNAInterfaceLoader.INSTANCE

import scala.collection.SortedSet
import scala.concurrent.ExecutionContext

object ReportingRSpacePlusPlus {
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
      val INSTANCE: JNAInterface =
        Native
          .load("rspace_plus_plus_rhotypes", classOf[JNAInterface])
          .asInstanceOf[JNAInterface]

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
        x.eventType match {
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
      })
    } yield reports

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

  override def createCheckpoint(): F[Checkpoint] =
    for {
      result <- Sync[F].delay {

                 val checkpointResultPtr = INSTANCE.reporting_create_checkpoint(
                   rspacePointer
                 )

                 if (checkpointResultPtr != null) {
                   val resultByteslength = checkpointResultPtr.getInt(0)

                   try {
                     val resultBytes     = checkpointResultPtr.getByteArray(4, resultByteslength)
                     val checkpointProto = CheckpointProto.parseFrom(resultBytes)
                     val checkpointRoot  = checkpointProto.root

                     val checkpointLogProto = checkpointProto.log
                     val checkpointLog = checkpointLogProto.map {
                       case eventProto if eventProto.eventType.isComm =>
                         val commProto = eventProto.eventType.comm.get
                         val consume   = commProto.consume
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
                             channelsHashes = consume.get.channelHashes
                               .map(bs => Blake2b256Hash.fromByteArray(bs.toByteArray)),
                             hash = Blake2b256Hash.fromByteArray(consume.get.hash.toByteArray),
                             persistent = consume.get.persistent
                           ),
                           produces = produces,
                           peeks = peeks,
                           timesRepeated = timesRepeated
                         )
                       case eventProto if eventProto.eventType.isIoEvent =>
                         val ioEventProto = eventProto.eventType.ioEvent.get
                         ioEventProto.ioEventType match {
                           case IOEventProto.IoEventType.Produce(produceProto) =>
                             Produce(
                               channelsHash =
                                 Blake2b256Hash.fromByteArray(produceProto.channelHash.toByteArray),
                               hash = Blake2b256Hash.fromByteArray(produceProto.hash.toByteArray),
                               persistent = produceProto.persistent
                             )
                           case IOEventProto.IoEventType.Consume(consumeProto) =>
                             Consume(
                               channelsHashes = consumeProto.channelHashes
                                 .map(bs => Blake2b256Hash.fromByteArray(bs.toByteArray)),
                               hash = Blake2b256Hash.fromByteArray(consumeProto.hash.toByteArray),
                               persistent = consumeProto.persistent
                             )
                           case _ =>
                             throw new RuntimeException("Unknown IOEvent type")
                         }
                       case _ =>
                         throw new RuntimeException("Unknown Event type")
                     }

                     Checkpoint(
                       root = Blake2b256Hash.fromByteArray(checkpointRoot.toByteArray),
                       log = checkpointLog
                     )

                   } catch {
                     case e: Throwable =>
                       println("Error during scala createCheckpoint operation: " + e)
                       throw e
                   } finally {
                     INSTANCE.deallocate_memory(checkpointResultPtr, resultByteslength)
                   }
                 } else {
                   println(
                     "Error during createCheckpoint operation: Checkpoint pointer from rust was null"
                   )
                   throw new RuntimeException("Checkpoint pointer from rust was null")
                 }
               }
    } yield result

}
