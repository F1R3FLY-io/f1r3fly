package rspacePlusPlus

import cats.effect.Sync
import cats.syntax.all._
import coop.rchain.rspace.hashing.Blake2b256Hash
import coop.rchain.rspace.internal._
import coop.rchain.rspace.util.ReplayException
import coop.rchain.shared.Log
import coop.rchain.rspace.trace.{COMM, Consume, Produce, ReplayData}
import com.sun.jna.{Memory, Native, Pointer}
import rspacePlusPlus.JNAInterfaceLoader.{INSTANCE => INSTANCE}
import coop.rchain.models.rspace_plus_plus_types.{
  CommProto,
  ConsumeProto,
  EventProto,
  IOEventProto,
  ProduceCounterMapEntry,
  ProduceProto,
  SortedSetElement
}
import coop.rchain.models.rspace_plus_plus_types.LogProto

trait IReplaySpacePlusPlus[F[_], C, P, A, K] extends ISpacePlusPlus[F, C, P, A, K] {

  protected def logF: Log[F]

  // private[rspacePlusPlus] val replayData: ReplayData = ReplayData.empty

  /** Rigs this ReplaySpace with the initial state and a log of permitted operations.
    * During replay, whenever a COMM event that is not available in the log occurs, an error is going to be raised.
    *
    * This method is not thread safe.
    *
    *  @param startRoot A [Blake2b256Hash] representing the intial state
    *  @param log A [Log] with permitted operations
    */
  def rigAndReset(startRoot: Blake2b256Hash, log: trace.Log)(implicit syncF: Sync[F]): F[Unit] =
    rig(log) >> reset(startRoot)

  def rig(log: trace.Log)(implicit syncF: Sync[F]): F[Unit] =
    for {
      _ <- Sync[F].delay {
            val rspacePointer = getRspacePointer

            val eventProtos = log.map {
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

            val logProto      = LogProto(eventProtos)
            val logProtoBytes = logProto.toByteArray

            if (!logProtoBytes.isEmpty) {
              val payloadMemory = new Memory(logProtoBytes.length.toLong)
              payloadMemory.write(0, logProtoBytes, 0, logProtoBytes.length)

              val _ = INSTANCE.rig(
                rspacePointer,
                payloadMemory,
                logProtoBytes.length
              )

              // Not sure if these lines are needed
              // Need to figure out how to deallocate each memory instance
              payloadMemory.clear()
            } else {
              println("\nlog is empty in rig")
            }
          }
    } yield ()

  def checkReplayData()(implicit syncF: Sync[F]): F[Unit] =
    for {
      _ <- Sync[F].delay {
            val rspacePointer = getRspacePointer
            val _ = INSTANCE.check_replay_data(
              rspacePointer
            )
          }
    } yield ()

  def getRspacePointer: Pointer
}
