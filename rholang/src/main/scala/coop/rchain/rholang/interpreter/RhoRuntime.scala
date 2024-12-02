package coop.rchain.rholang.interpreter

import cats.{Monad, Parallel}
import cats.data.Chain
import cats.effect._
import cats.effect.concurrent.Ref
import cats.mtl.FunctorTell
import cats.syntax.all._
import coop.rchain.crypto.hash.Blake2b512Random
import coop.rchain.metrics.Metrics.Source
import coop.rchain.metrics.{Metrics, Span}
import coop.rchain.models.BlockHash.BlockHash
import coop.rchain.models.TaggedContinuation.TaggedCont.ScalaBodyRef
import coop.rchain.models.Validator.Validator
import coop.rchain.models.Var.VarInstance.FreeVar
import coop.rchain.models._
import coop.rchain.models.rholang.implicits._
import coop.rchain.rholang.RholangMetricsSource
import coop.rchain.rholang.interpreter.RhoRuntime.{RhoISpace, RhoReplayISpace}
import coop.rchain.rholang.interpreter.RholangAndScalaDispatcher.RhoDispatch
import coop.rchain.rholang.interpreter.SystemProcesses._
import coop.rchain.rholang.interpreter.accounting.{_cost, Cost, CostAccounting, HasCost}
import coop.rchain.rholang.interpreter.registry.RegistryBootstrap
import coop.rchain.rholang.interpreter.storage.ChargingRSpace
// import coop.rchain.rspace.RSpace.RSpaceStore
import coop.rchain.rspace.hashing.Blake2b256Hash
// import coop.rchain.rspace.history.HistoryRepository
import rspacePlusPlus.HistoryRepository
import coop.rchain.rspace.internal.{Datum, Row, WaitingContinuation}
import coop.rchain.rspace.util.unpackOption
import coop.rchain.rspace._
import coop.rchain.shared.Log
import monix.execution.Scheduler

import rspacePlusPlus.{
  IReplaySpacePlusPlus,
  ISpacePlusPlus,
  RSpacePlusPlus_RhoTypes,
  TuplespacePlusPlus
}

import com.sun.jna.{Memory, Pointer}
import coop.rchain.rholang.JNAInterfaceLoader.RHOLANG_RUST_INSTANCE
import coop.rchain.models.rholang_scala_rust_types._
import com.google.protobuf.ByteString
import coop.rchain.rholang.interpreter.errors.BugFoundError
import coop.rchain.rholang.interpreter.errors.RustError
import coop.rchain.models.rspace_plus_plus_types._
import coop.rchain.rspace.trace.Produce
import scala.collection.SortedSet
import coop.rchain.rspace.trace._

// trait RhoRuntime[F[_]] extends HasCost[F] {
trait RhoRuntime[F[_]] {

  /**
    * Parse the rholang term into [[coop.rchain.models.Par]] and execute it with provided initial phlo.
    *
    * This function would change the state in the runtime.
    * @param term The rholang contract which would run on the runtime
    * @param initialPhlo initial cost for the this evaluation. If the phlo is not enough,
    *                    [[coop.rchain.rholang.interpreter.errors.OutOfPhlogistonsError]] would return.
    * @param normalizerEnv additional env for Par when parsing term into Par
    * @param rand random seed for rholang execution
    * @return
    */
  def evaluate(term: String, initialPhlo: Cost, normalizerEnv: Map[String, Par])(
      implicit rand: Blake2b512Random
  ): F[EvaluateResult]

  /**
    * The function would execute the par regardless setting cost which would possibly cause
    * [[coop.rchain.rholang.interpreter.errors.OutOfPhlogistonsError]]. Because of that, use this
    * function in some situation which is not cost sensitive.
    *
    * This function would change the state in the runtime.
    *
    * Ideally, this function should be removed or hack the runtime without cost accounting in the future .
    * @param par [[coop.rchain.models.Par]] for the execution
    * @param env additional env for execution
    * @param rand random seed for rholang execution
    * @return
    */
  def inj(par: Par, env: Env[Par] = Env[Par]())(
      implicit rand: Blake2b512Random
  ): F[Unit]

  /**
    * After some executions([[evaluate]]) on the runtime, you can create a soft checkpoint which is the changes
    * for the current state of the runtime. You can revert the changes by [[revertToSoftCheckpoint]]
    * @return
    */
  def createSoftCheckpoint
      : F[SoftCheckpoint[Par, BindPattern, ListParWithRandom, TaggedContinuation]]

  def revertToSoftCheckpoint(
      softCheckpoint: SoftCheckpoint[Par, BindPattern, ListParWithRandom, TaggedContinuation]
  ): F[Unit]

  /**
    * Create a checkpoint for the runtime. All the changes which happened in the runtime would persistent in the disk
    * and result in a new stateHash for the new state.
    * @return
    */
  def createCheckpoint: F[Checkpoint]

  /**
    * Reset the runtime to the specific state. Then you can operate some execution on the state.
    * @param root the target state hash to reset
    * @return
    */
  def reset(root: Blake2b256Hash): F[Unit]

  /**
    * Consume the result in the rspace.
    *
    * This function would change the state in the runtime.
    * @param channel target channel for the consume
    * @param pattern pattern for the consume
    * @return
    */
  def consumeResult(
      channel: Seq[Par],
      pattern: Seq[BindPattern]
  ): F[Option[(TaggedContinuation, Seq[ListParWithRandom])]]

  /**
    * get data directly from history repository
    *
    * This function would not change the state in the runtime
    */
  def getData(channel: Par): F[Seq[Datum[ListParWithRandom]]]

  def getJoins(channel: Par): F[Seq[Seq[Par]]]

  /**
    * get data directly from history repository
    *
    * This function would not change the state in the runtime
    */
  def getContinuation(
      channels: Seq[Par]
  ): F[Seq[WaitingContinuation[BindPattern, TaggedContinuation]]]

  /**
    * Set the runtime block data environment.
    */
  def setBlockData(blockData: BlockData): F[Unit]

  /**
    * Set the runtime invalid blocks environment.
    */
  def setInvalidBlocks(invalidBlocks: Map[BlockHash, Validator]): F[Unit]

  /**
    * Get the hot changes after some executions for the runtime.
    * Currently this is only for debug info mostly.
    */
  def getHotChanges: F[Map[Seq[Par], Row[BindPattern, ListParWithRandom, TaggedContinuation]]]

  /* Additional methods for Rholang Rust Integration */

  def setCostToMax: F[Unit]

  def getRuntimePtr: Pointer
}

trait ReplayRhoRuntime[F[_]] extends RhoRuntime[F] {
  def rig(log: trace.Log): F[Unit]

  def checkReplayData: F[Unit]
}

class RhoRuntimeImpl[F[_]: Sync: Span](
    // val reducer: Reduce[F],
    // val space: RhoISpace[F],
    // val cost: _cost[F],
    // val blockDataRef: Ref[F, BlockData],
    // val invalidBlocksParam: InvalidBlocks[F],
    // val mergeChs: Ref[F, Set[Par]]
    runtimePtr: Pointer
) extends RhoRuntime[F] {
  private val emptyContinuation = TaggedContinuation()

  override def evaluate(term: String, initialPhlo: Cost, normalizerEnv: Map[String, Name])(
      implicit rand: Blake2b512Random
  ): F[EvaluateResult] =
    Sync[F].delay {
      // println("\nterm in evaluate: " + term)
      // println("\nrand in scala evaluate: " + Blake2b512Random.debugStr(rand))
      val pathPosition = rand.pathView.position()
      val blake2b512BlockProto = Blake2b512BlockProto(
        chainValue = rand.digest.chainValue.map(v => Int64Proto(v)).toSeq,
        t0 = rand.digest.t0,
        t1 = rand.digest.t1
      )

      val blake2b512RandomProto = Blake2b512RandomProto(
        digest = Some(blake2b512BlockProto),
        lastBlock = ByteString.copyFrom(rand.lastBlock),
        pathView = ByteString.copyFrom(rand.pathView),
        countView = {
          val buffer    = rand.countView
          val countList = (0 until buffer.limit()).map(buffer.get(_)).map(UInt64Proto(_))
          countList
        },
        hashArray = ByteString.copyFrom(rand.hashArray),
        position = rand.position.toLong,
        pathPosition = pathPosition
      )

      val evalParams = EvaluateParams(
        term,
        Some(CostProto(initialPhlo.value.toLong, initialPhlo.operation)),
        normalizerEnv,
        Some(blake2b512RandomProto)
      )

      val paramsBytes = evalParams.toByteArray
      val paramsPtr   = new Memory(paramsBytes.length.toLong)
      paramsPtr.write(0, paramsBytes, 0, paramsBytes.length)

      val evalResultPtr = RHOLANG_RUST_INSTANCE.evaluate(runtimePtr, paramsPtr, paramsBytes.length)
      assert(evalResultPtr != null)

      try {
        val resultByteslength = evalResultPtr.getInt(0)
        val resultBytes       = evalResultPtr.getByteArray(4, resultByteslength)
        val evalResultProto   = EvaluateResultProto.parseFrom(resultBytes)

        EvaluateResult(
          Cost(evalResultProto.getCost.value.toLong, evalResultProto.getCost.operation),
          evalResultProto.errors
            .map(
              err =>
                RustError(
                  err
                )
            )
            .toVector,
          evalResultProto.mergeable.toSet
        )

      } catch {
        case e: Throwable =>
          println("Error parsing EvaluateResultProto: " + e)
          throw e
      }
    }

  override def inj(par: Par, env: Env[Par] = Env[Par]())(implicit rand: Blake2b512Random): F[Unit] =
    // reducer.inj(par)
    ???

  override def createSoftCheckpoint
      : F[SoftCheckpoint[Par, BindPattern, ListParWithRandom, TaggedContinuation]] =
    // Span[F].withMarks("create-soft-heckpoint") {
    //   space.createSoftCheckpoint()
    // }
    for {
      result <- Sync[F].delay {
                 //  println("\nhit scala createSoftCheckpoint")
                 val softCheckpointPtr = RHOLANG_RUST_INSTANCE.create_soft_checkpoint(runtimePtr)

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
                                   //  Log[F].debug("ConsumeEvent is None");
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
                                 //  Log[F].debug("ProduceEvent is None");
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

                     val cacheSnapshot
                         : HotStoreState[Par, BindPattern, ListParWithRandom, TaggedContinuation] =
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
                   }
                 } else {
                   println("softCheckpointPtr is null")
                   throw new RuntimeException("softCheckpointPtr is null")
                 }

               }
    } yield result

  override def revertToSoftCheckpoint(
      softCheckpoint: SoftCheckpoint[Name, BindPattern, ListParWithRandom, TaggedContinuation]
  ): F[Unit] =
    // space.revertToSoftCheckpoint(softCheckpoint)
    for {
      _ <- Sync[F].delay {
            // println("\nhit scala revertToSoftCheckpoint")
            val cacheSnapshot = softCheckpoint.cacheSnapshot

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
            val produceCounterMap        = softCheckpoint.produceCounter

            produceCounterMap.map { mapEntry =>
              val produce = mapEntry._1
              val produceProto = ProduceProto(
                produce.channelsHash.toByteString,
                produce.hash.toByteString,
                produce.persistent
              )

              produceCounterMapEntries :+ ProduceCounterMapEntry(Some(produceProto), mapEntry._2)
            }

            val logProto = softCheckpoint.log.map {
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

            val _ = RHOLANG_RUST_INSTANCE.revert_to_soft_checkpoint(
              runtimePtr,
              payloadMemory,
              softCheckpointProtoBytes.length
            )

            // Not sure if these lines are needed
            // Need to figure out how to deallocate each memory instance
            payloadMemory.clear()
          }
    } yield ()

  override def createCheckpoint: F[Checkpoint] =
    // 	Span[F].withMarks("create-checkpoint") {
    //   space.createCheckpoint()
    // }
    for {
      result <- Sync[F].delay {
                 //  println("\nhit scala createCheckpoint")

                 val checkpointResultPtr = RHOLANG_RUST_INSTANCE.create_checkpoint(
                   runtimePtr
                 )

                 if (checkpointResultPtr != null) {
                   val resultByteslength = checkpointResultPtr.getInt(0)

                   try {
                     val resultBytes     = checkpointResultPtr.getByteArray(4, resultByteslength)
                     val checkpointProto = CheckpointProto.parseFrom(resultBytes)
                     val checkpointRoot  = checkpointProto.root

                     //  println(
                     //    "\nscala createCheckpoint root: " + Blake2b256Hash
                     //      .fromByteArray(checkpointRoot.toByteArray)
                     //  )

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

                     //  println("\n log: " + checkpointLog)

                     Checkpoint(
                       root = Blake2b256Hash.fromByteArray(checkpointRoot.toByteArray),
                       log = checkpointLog
                     )
                     //  Checkpoint(
                     //    root = Blake2b256Hash.fromByteArray(checkpointRoot.toByteArray),
                     //    log = Seq.empty[Event]
                     //  )

                   } catch {
                     case e: Throwable =>
                       println("Error during scala createCheckpoint operation: " + e)
                       throw e
                   }
                 } else {
                   println(
                     "Error during createCheckpoint operation: Checkpoint pointer from rust was null"
                   )
                   throw new RuntimeException("Checkpoint pointer from rust was null")
                 }
               }
    } yield result

  override def reset(root: Blake2b256Hash): F[Unit] =
    // space.reset(root)
    for {
      _ <- Sync[F].delay {
            // println("\nhit scala reset, root: " + root)
            val rootBytes = root.bytes.toArray

            val rootMemory = new Memory(rootBytes.length.toLong)
            rootMemory.write(0, rootBytes, 0, rootBytes.length)

            val _ = RHOLANG_RUST_INSTANCE.reset(
              runtimePtr,
              rootMemory,
              rootBytes.length
            )

            // Not sure if these lines are needed
            // Need to figure out how to deallocate each memory instance
            rootMemory.clear()
          }
    } yield ()

  override def consumeResult(
      channel: Seq[Par],
      pattern: Seq[BindPattern]
  ): F[Option[(TaggedContinuation, Seq[ListParWithRandom])]] =
    // space.consume(channel, pattern, emptyContinuation, persist = false).map(unpackOption)
    ???

  override def getData(channel: Par): F[Seq[Datum[ListParWithRandom]]] =
    // space.getData(channel)
    for {
      result <- Sync[F].delay {
                 val channelBytes = channel.toByteArray

                 val payloadMemory = new Memory(channelBytes.length.toLong)
                 payloadMemory.write(0, channelBytes, 0, channelBytes.length)

                 val getDataResultPtr = RHOLANG_RUST_INSTANCE.get_data(
                   runtimePtr,
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

                     val datums: Seq[Datum[ListParWithRandom]] =
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
                   }
                 } else {
                   println("getDataResultPtr is null")
                   throw new RuntimeException("getDataResultPtr is null")
                 }
               }
    } yield result

  override def getJoins(channel: Par): F[Seq[Seq[Par]]] =
    // space.getJoins(channel)
    for {
      result <- Sync[F].delay {
                 val channelBytes = channel.toByteArray

                 val payloadMemory = new Memory(channelBytes.length.toLong)
                 payloadMemory.write(0, channelBytes, 0, channelBytes.length)

                 val getJoinsResultPtr = RHOLANG_RUST_INSTANCE.get_joins(
                   runtimePtr,
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

                     val joins: Seq[Seq[Par]] =
                       joinsProtos.map(
                         join => join.join
                       )

                     joins

                   } catch {
                     case e: Throwable =>
                       println("Error during scala getJoins operation: " + e)
                       throw e
                   }
                 } else {
                   println("getJoinsResultPtr is null")
                   throw new RuntimeException("getJoinsResultPtr is null")
                 }
               }
    } yield result

  override def getContinuation(
      channels: Seq[Name]
  ): F[Seq[WaitingContinuation[BindPattern, TaggedContinuation]]] =
    // space.getWaitingContinuations(channels)
    for {
      result <- Sync[F].delay {
                 val channelsProto = ChannelsProto(channels)
                 val channelsBytes = channelsProto.toByteArray

                 val payloadMemory = new Memory(channelsBytes.length.toLong)
                 payloadMemory.write(0, channelsBytes, 0, channelsBytes.length)

                 val getWaitingContinuationResultPtr =
                   RHOLANG_RUST_INSTANCE.get_waiting_continuations(
                     runtimePtr,
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

                     val wks: Seq[WaitingContinuation[BindPattern, TaggedContinuation]] =
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
                   }
                 } else {
                   println("getWaitingContinuationResultPtr is null")
                   throw new RuntimeException("getWaitingContinuationResultPtr is null")
                 }
               }
    } yield result

  override def setBlockData(blockData: BlockData): F[Unit] =
    // blockDataRef.set(blockData)
    Sync[F].delay {
      val setBlockDataParams = BlockDataProto(
        blockData.timeStamp.toLong,
        blockData.blockNumber.toLong,
        ByteString.copyFrom(blockData.sender.bytes),
        blockData.seqNum
      )

      // println("\nSetBlockDataParams: " + setBlockDataParams)

      val paramsBytes = setBlockDataParams.toByteArray
      // println("\nparamsBytes: " + paramsBytes.length)

      // NOTE: Here, if 'blockData' fields are empty, then 'paramsBytes.length' will be 0 and throw an error.
      // So in this case, I skip calling Rust 'set_block_data'
      if (paramsBytes.length != 0) {
        val paramsPtr = new Memory(paramsBytes.length.toLong)
        paramsPtr.write(0, paramsBytes, 0, paramsBytes.length)

        RHOLANG_RUST_INSTANCE.set_block_data(runtimePtr, paramsPtr, paramsBytes.length)
      }
    }

  override def setInvalidBlocks(invalidBlocks: Map[BlockHash, Validator]): F[Unit] =
    Sync[F].delay {
      val invalidBlocksProto = InvalidBlocksProto(
        invalidBlocks.map {
          case (blockHash, validator) => {
            println("\nblockHash: " + blockHash.toByteArray())
            println("\nvalidator: " + validator.toByteArray())
            BlockHashValidator(
              blockHash,
              validator
            )
          }
        }.toSeq
      )

      val paramsBytes = invalidBlocksProto.toByteArray
      // NOTE: Here, if 'blockData' fields are empty, then 'paramsBytes.length' will be 0 and throw an error.
      // So in this case, I skip calling Rust 'set_block_data'
      if (paramsBytes.length != 0) {
        val paramsPtr = new Memory(paramsBytes.length.toLong)
        paramsPtr.write(0, paramsBytes, 0, paramsBytes.length)

        RHOLANG_RUST_INSTANCE.set_invalid_blocks(runtimePtr, paramsPtr, paramsBytes.length)
      } else {
        println("\nparamsBytes.length is 0, not calling Rust replay_set_invalid_blocks")
      }
    }

  override def getHotChanges
      : F[Map[Seq[Par], Row[BindPattern, ListParWithRandom, TaggedContinuation]]] =
    // space.toMap
    for {
      result <- Sync[F].delay {
                 val toMapPtr = RHOLANG_RUST_INSTANCE.get_hot_changes(runtimePtr)

                 if (toMapPtr != null) {
                   val length = toMapPtr.getInt(0)

                   try {
                     val resultBytes = toMapPtr.getByteArray(4, length)
                     val toMapResult = StoreToMapResult.parseFrom(resultBytes)

                     val map = toMapResult.mapEntries.flatMap { mapEntry =>
                       val key = mapEntry.key
                       mapEntry.value match {
                         case Some(row) =>
                           val value = Row(
                             data = row.data.map(
                               datum =>
                                 Datum[ListParWithRandom](
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
                                       println("ProduceEvent is None")
                                       throw new RuntimeException("ProduceEvent is None")
                                     }
                                   }
                                 )
                             ),
                             wks = row.wks.map(
                               wk =>
                                 WaitingContinuation[BindPattern, TaggedContinuation](
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
                                       println("ConsumeEvent is None")
                                       throw new RuntimeException("ConsumeEvent is None")
                                     }
                                   }
                                 )
                             )
                           )
                           Some(key -> value)
                         case None =>
                           println("Row is None")
                           None
                       }
                     }.toMap

                     map
                   } catch {
                     case e: Throwable =>
                       println("Error during scala toMap operation: " + e)
                       throw e
                   }
                 } else {
                   println("toMapPtr is null")
                   throw new RuntimeException("toMapPtr is null")
                 }

               }
    } yield result

  override def setCostToMax: F[Unit] = ???

  override def getRuntimePtr: Pointer = runtimePtr
}

class ReplayRhoRuntimeImpl[F[_]: Sync: Span](
    // override val reducer: Reduce[F],
    // override val space: RhoReplayISpace[F],
    // override val cost: _cost[F],
    // // TODO: Runtime must be immutable. Block data and invalid blocks should be supplied when Runtime is created.
    // //  This also means to unify all special names necessary to spawn a new Runtime.
    // override val blockDataRef: Ref[F, BlockData],
    // override val invalidBlocksParam: InvalidBlocks[F],
    // override val mergeChs: Ref[F, Set[Par]]
    // runtimePtr: Pointer,
    runtimePtr: Pointer
) extends RhoRuntimeImpl[F](runtimePtr)
    with ReplayRhoRuntime[F] {
  override def checkReplayData: F[Unit] =
    // space.checkReplayData()
    for {
      _ <- Sync[F].delay {
            val _ = RHOLANG_RUST_INSTANCE.check_replay_data(
              runtimePtr
            )
          }
    } yield ()

  override def rig(log: trace.Log): F[Unit] =
    for {
      _ <- Sync[F].delay {
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

              val _ = RHOLANG_RUST_INSTANCE.rig(
                runtimePtr,
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

  override def getRuntimePtr: Pointer = runtimePtr
}

object ReplayRhoRuntime {
  def apply[F[_]: Sync: Span](
      reducer: Reduce[F],
      space: RhoReplayISpace[F],
      cost: _cost[F],
      blockDataRef: Ref[F, BlockData],
      invalidBlocksParam: InvalidBlocks[F],
      mergeChs: Ref[F, Set[Par]]
  ) =
    // new ReplayRhoRuntimeImpl[F](reducer, space, cost, blockDataRef, invalidBlocksParam, mergeChs)
    ???
}

object RhoRuntime {

  val currentDir = java.nio.file.Paths.get("").toAbsolutePath.toString
  println(s"Current directory: $currentDir")

  val jnaLibraryPath = System.getProperty("jna.library.path")
  println(s"Current jna.library.path: $jnaLibraryPath")

  val maxHeapSize         = Runtime.getRuntime.maxMemory() / (1024 * 1024)
  val initialHeapSize     = Runtime.getRuntime.totalMemory() / (1024 * 1024)
  val freeMemory          = Runtime.getRuntime.freeMemory() / (1024 * 1024)
  val availableProcessors = Runtime.getRuntime.availableProcessors()

  println(s"Max Heap Size: ${maxHeapSize}MB")
  println(s"Initial Heap Size: ${initialHeapSize}MB")
  println(s"Free Memory: ${freeMemory}MB")
  println(s"Available Processors: ${availableProcessors}")

  implicit val RuntimeMetricsSource: Source = Metrics.Source(RholangMetricsSource, "runtime")
  private[this] val createReplayRuntime     = Metrics.Source(RuntimeMetricsSource, "create-replay")
  private[this] val createPlayRuntime       = Metrics.Source(RuntimeMetricsSource, "create-play")

  def apply[F[_]: Sync: Span](
      reducer: Reduce[F],
      space: RhoISpace[F],
      cost: _cost[F],
      blockDataRef: Ref[F, BlockData],
      invalidBlocksParam: InvalidBlocks[F],
      mergeChs: Ref[F, Set[Par]]
  ) =
    // new RhoRuntimeImpl[F](reducer, space, cost, blockDataRef, invalidBlocksParam, mergeChs)
    ???

  type RhoTuplespace[F[_]]   = TCPAK[F, TuplespacePlusPlus]
  type RhoISpace[F[_]]       = TCPAK[F, ISpacePlusPlus]
  type RhoReplayISpace[F[_]] = TCPAK[F, IReplaySpacePlusPlus]

  // type RhoTuplespace[F[_]]   = TCPAK[F, Tuplespace]
  // type RhoISpace[F[_]]       = TCPAK[F, ISpace]
  // type RhoReplayISpace[F[_]] = TCPAK[F, IReplaySpace]
  type ISpaceAndReplay[F[_]] = (RhoISpace[F], RhoReplayISpace[F])

  type RhoHistoryRepository[F[_]] =
    HistoryRepository[F, Par, BindPattern, ListParWithRandom, TaggedContinuation]

  type TCPAK[M[_], F[_[_], _, _, _, _]] =
    F[M, Par, BindPattern, ListParWithRandom, TaggedContinuation]

  def introduceSystemProcesses[F[_]: Sync: _cost: Span](
      spaces: List[RhoTuplespace[F]],
      processes: List[(Name, Arity, Remainder, BodyRef)]
  ): F[List[Option[(TaggedContinuation, Seq[ListParWithRandom])]]] =
    // println("\nhit introduceSystemProcesses")
    // println("spaces: " + spaces)
    // println("processes: " + processes)
    // processes.flatMap {
    //   case (name, arity, remainder, ref) =>
    //     val channels = List(name)
    //     val patterns = List(
    //       BindPattern(
    //         (0 until arity).map[Par, Seq[Par]](i => EVar(FreeVar(i))),
    //         remainder,
    //         freeCount = arity
    //       )
    //     )
    //     val continuation = TaggedContinuation(ScalaBodyRef(ref))
    //     spaces.map(_.install(channels, patterns, continuation))
    // }.sequence
    ???

  def stdSystemProcesses[F[_]]: Seq[Definition[F]] = Seq(
    Definition[F]("rho:io:stdout", FixedChannels.STDOUT, 1, BodyRefs.STDOUT, {
      ctx: ProcessContext[F] =>
        ctx.systemProcesses.stdOut
    }),
    Definition[F]("rho:io:stdoutAck", FixedChannels.STDOUT_ACK, 2, BodyRefs.STDOUT_ACK, {
      ctx: ProcessContext[F] =>
        ctx.systemProcesses.stdOutAck
    }),
    Definition[F]("rho:io:stderr", FixedChannels.STDERR, 1, BodyRefs.STDERR, {
      ctx: ProcessContext[F] =>
        ctx.systemProcesses.stdErr
    }),
    Definition[F]("rho:io:stderrAck", FixedChannels.STDERR_ACK, 2, BodyRefs.STDERR_ACK, {
      ctx: ProcessContext[F] =>
        ctx.systemProcesses.stdErrAck
    }),
    Definition[F](
      "rho:block:data",
      FixedChannels.GET_BLOCK_DATA,
      1,
      BodyRefs.GET_BLOCK_DATA, { ctx =>
        ctx.systemProcesses.getBlockData(ctx.blockData)
      }
    ),
    Definition[F](
      "rho:casper:invalidBlocks",
      FixedChannels.GET_INVALID_BLOCKS,
      1,
      BodyRefs.GET_INVALID_BLOCKS, { ctx =>
        ctx.systemProcesses.invalidBlocks(ctx.invalidBlocks)
      }
    ),
    Definition[F](
      "rho:rev:address",
      FixedChannels.REV_ADDRESS,
      3,
      BodyRefs.REV_ADDRESS, { ctx =>
        ctx.systemProcesses.revAddress
      }
    ),
    Definition[F](
      "rho:rchain:deployerId:ops",
      FixedChannels.DEPLOYER_ID_OPS,
      3,
      BodyRefs.DEPLOYER_ID_OPS, { ctx =>
        ctx.systemProcesses.deployerIdOps
      }
    ),
    Definition[F](
      "rho:registry:ops",
      FixedChannels.REG_OPS,
      3,
      BodyRefs.REG_OPS, { ctx =>
        ctx.systemProcesses.registryOps
      }
    ),
    Definition[F](
      "sys:authToken:ops",
      FixedChannels.SYS_AUTHTOKEN_OPS,
      3,
      BodyRefs.SYS_AUTHTOKEN_OPS, { ctx =>
        ctx.systemProcesses.sysAuthTokenOps
      }
    )
  )

  def stdRhoCryptoProcesses[F[_]]: Seq[Definition[F]] = Seq(
    Definition[F](
      "rho:crypto:secp256k1Verify",
      FixedChannels.SECP256K1_VERIFY,
      4,
      BodyRefs.SECP256K1_VERIFY, { ctx =>
        ctx.systemProcesses.secp256k1Verify
      }
    ),
    Definition[F](
      "rho:crypto:blake2b256Hash",
      FixedChannels.BLAKE2B256_HASH,
      2,
      BodyRefs.BLAKE2B256_HASH, { ctx =>
        ctx.systemProcesses.blake2b256Hash
      }
    ),
    Definition[F](
      "rho:crypto:keccak256Hash",
      FixedChannels.KECCAK256_HASH,
      2,
      BodyRefs.KECCAK256_HASH, { ctx =>
        ctx.systemProcesses.keccak256Hash
      }
    ),
    Definition[F](
      "rho:crypto:sha256Hash",
      FixedChannels.SHA256_HASH,
      2,
      BodyRefs.SHA256_HASH, { ctx =>
        ctx.systemProcesses.sha256Hash
      }
    ),
    Definition[F](
      "rho:crypto:ed25519Verify",
      FixedChannels.ED25519_VERIFY,
      4,
      BodyRefs.ED25519_VERIFY, { ctx =>
        ctx.systemProcesses.ed25519Verify
      }
    )
  )

  def dispatchTableCreator[F[_]: Concurrent: Span](
      space: RhoTuplespace[F],
      dispatcher: RhoDispatch[F],
      blockData: Ref[F, BlockData],
      invalidBlocks: InvalidBlocks[F],
      extraSystemProcesses: Seq[Definition[F]]
  ): RhoDispatchMap[F] =
    // (stdSystemProcesses[F] ++ stdRhoCryptoProcesses[F] ++ extraSystemProcesses)
    //   .map(
    //     _.toDispatchTable(
    //       ProcessContext(space, dispatcher, blockData, invalidBlocks)
    //     )
    //   )
    //   .toMap
    ???

  val basicProcesses: Map[String, Par] = Map[String, Par](
    "rho:registry:lookup"          -> Bundle(FixedChannels.REG_LOOKUP, writeFlag = true),
    "rho:registry:insertArbitrary" -> Bundle(FixedChannels.REG_INSERT_RANDOM, writeFlag = true),
    "rho:registry:insertSigned:secp256k1" -> Bundle(
      FixedChannels.REG_INSERT_SIGNED,
      writeFlag = true
    )
  )

  def setupReducer[F[_]: Concurrent: Parallel: _cost: Log: Metrics: Span](
      chargingRSpace: RhoTuplespace[F],
      blockDataRef: Ref[F, BlockData],
      invalidBlocks: InvalidBlocks[F],
      extraSystemProcesses: Seq[Definition[F]],
      urnMap: Map[String, Par],
      mergeChs: Ref[F, Set[Par]],
      mergeableTagName: Par
  ): Reduce[F] =
    // lazy val replayDispatchTable: RhoDispatchMap[F] =
    //   dispatchTableCreator(
    //     chargingRSpace,
    //     replayDispatcher,
    //     blockDataRef,
    //     invalidBlocks,
    //     extraSystemProcesses
    //   )

    // lazy val (replayDispatcher, replayReducer) =
    //   RholangAndScalaDispatcher(
    //     chargingRSpace,
    //     replayDispatchTable,
    //     urnMap,
    //     mergeChs,
    //     mergeableTagName
    //   )
    // replayReducer
    ???

  def setupMapsAndRefs[F[_]: Sync](
      extraSystemProcesses: Seq[Definition[F]] = Seq.empty
  ): F[
    (Ref[F, BlockData], InvalidBlocks[F], Map[String, Name], Seq[(Name, Arity, Remainder, BodyRef)])
  ] =
    //   for {
    //     blockDataRef  <- Ref.of(BlockData.empty)
    //     invalidBlocks = InvalidBlocks.unsafe[F]()
    //     urnMap = basicProcesses ++ (stdSystemProcesses[F] ++ stdRhoCryptoProcesses[F] ++ extraSystemProcesses)
    //       .map(_.toUrnMap)
    //     procDefs = (stdSystemProcesses[F] ++ stdRhoCryptoProcesses[F] ++ extraSystemProcesses)
    //       .map(_.toProcDefs)
    //   } yield (blockDataRef, invalidBlocks, urnMap, procDefs)
    ???

  def createRhoEnv[F[_]: Concurrent: Parallel: _cost: Log: Metrics: Span](
      rspace: RhoISpace[F],
      mergeChs: Ref[F, Set[Par]],
      mergeableTagName: Par,
      extraSystemProcesses: Seq[Definition[F]] = Seq.empty
  ): F[(Reduce[F], Ref[F, BlockData], InvalidBlocks[F])] =
    //   for {
    //     mapsAndRefs                                     <- setupMapsAndRefs(extraSystemProcesses)
    //     (blockDataRef, invalidBlocks, urnMap, procDefs) = mapsAndRefs
    //     reducer = setupReducer(
    //       ChargingRSpace.chargingRSpace[F](rspace),
    //       blockDataRef,
    //       invalidBlocks,
    //       extraSystemProcesses,
    //       urnMap,
    //       mergeChs,
    //       mergeableTagName
    //     )
    //     res <- introduceSystemProcesses(rspace :: Nil, procDefs.toList)
    //     _   = assert(res.forall(_.isEmpty))
    //   } yield (reducer, blockDataRef, invalidBlocks)
    ???

  def bootstrapRegistry[F[_]: Sync](runtime: RhoRuntime[F]): F[Unit] =
    Sync[F].delay {
      RHOLANG_RUST_INSTANCE.bootstrap_registry(runtime.getRuntimePtr)
    }

  private def createRuntime[F[_]: Concurrent: Log: Metrics: Span: Parallel](
      rspace: RhoISpace[F],
      extraSystemProcesses: Seq[Definition[F]],
      initRegistry: Boolean,
      mergeableTagName: Par
      // removed 'implicit costLog: FunctorTell[F, Chain[Cost]]'
  )(): F[RhoRuntime[F]] =
    Span[F].trace(createPlayRuntime) {
      // println("\nscala createRuntime")
      if (extraSystemProcesses.nonEmpty) {
        println(s"extra system processes was nonEmpty, size: ${extraSystemProcesses.size}")
      }

      Sync[F].delay {
        val runtimeParams = CreateRuntimeParams(
          Some(mergeableTagName),
          initRegistry
        )

        val runtimeParamsBytes = runtimeParams.toByteArray
        val paramsPtr          = new Memory(runtimeParamsBytes.length.toLong)
        paramsPtr.write(0, runtimeParamsBytes, 0, runtimeParamsBytes.length)

        val spacePtr = rspace.getRspacePointer
        val runtimePtr =
          RHOLANG_RUST_INSTANCE.create_runtime(spacePtr, paramsPtr, runtimeParamsBytes.length)
        assert(runtimePtr != null)
        new RhoRuntimeImpl[F](runtimePtr)
      }
    }

  /**
    *
    * @param rspace the rspace which the runtime would operate on it
    * @param extraSystemProcesses extra system rholang processes exposed to the runtime
    *                             which you can execute function on it
    * @param initRegistry For a newly created rspace, you might need to bootstrap registry
    *                     in the runtime to use rholang registry normally. Actually this initRegistry
    *                     is not the only thing you need for rholang registry, after the bootstrap
    *                     registry, you still need to insert registry contract on the rspace.
    *                     For a exist rspace which bootstrap registry before, you can skip this.
    *                     For some test cases, you don't need the registry then you can skip this
    *                     init process which can be faster.
    * @param costLog currently only the testcases needs a special costLog for test information.
    *                Normally you can just
    *                use [[coop.rchain.rholang.interpreter.accounting.noOpCostLog]]
    * @return
    */
  def createRhoRuntime[F[_]: Concurrent: Log: Metrics: Span: Parallel](
      rspace: RhoISpace[F],
      mergeableTagName: Par,
      initRegistry: Boolean = true,
      extraSystemProcesses: Seq[Definition[F]] = Seq.empty
      // removed 'implicit costLog: FunctorTell[F, Chain[Cost]]'
  )(): F[RhoRuntime[F]] =
    createRuntime[F](rspace, extraSystemProcesses, initRegistry, mergeableTagName)

  /**
    *
    * @param rspace the replay rspace which the runtime operate on it
    * @param extraSystemProcesses same as [[coop.rchain.rholang.interpreter.RhoRuntime.createRhoRuntime]]
    * @param initRegistry same as [[coop.rchain.rholang.interpreter.RhoRuntime.createRhoRuntime]]
    * @param costLog same as [[coop.rchain.rholang.interpreter.RhoRuntime.createRhoRuntime]]
    * @return
    */
  def createReplayRhoRuntime[F[_]: Concurrent: Log: Metrics: Span: Parallel](
      replaySpace: RhoReplayISpace[F],
      mergeableTagName: Par,
      extraSystemProcesses: Seq[Definition[F]] = Seq.empty,
      initRegistry: Boolean = true
      // removed 'implicit costLog: FunctorTell[F, Chain[Cost]]'
  )(): F[ReplayRhoRuntime[F]] =
    Span[F].trace(createReplayRuntime) {
      // println("\nscala createReplayRuntime")
      if (extraSystemProcesses.nonEmpty) {
        println(s"extra system processes was nonEmpty, size: ${extraSystemProcesses.size}")
      }

      Sync[F].delay {
        val runtimeParams = CreateRuntimeParams(
          Some(mergeableTagName),
          initRegistry
        )

        val runtimeParamsBytes = runtimeParams.toByteArray
        val paramsPtr          = new Memory(runtimeParamsBytes.length.toLong)
        paramsPtr.write(0, runtimeParamsBytes, 0, runtimeParamsBytes.length)

        val replaySpacePtr = replaySpace.getRspacePointer
        val runtimePtr =
          RHOLANG_RUST_INSTANCE.create_replay_runtime(
            replaySpacePtr,
            paramsPtr,
            runtimeParamsBytes.length
          )
        assert(runtimePtr != null)
        new ReplayRhoRuntimeImpl[F](runtimePtr)
      }
    }

  def createRuntimes[F[_]: Concurrent: ContextShift: Parallel: Log: Metrics: Span](
      space: RhoISpace[F],
      replaySpace: RhoReplayISpace[F],
      initRegistry: Boolean,
      additionalSystemProcesses: Seq[Definition[F]],
      mergeableTagName: Par
  ): F[(RhoRuntime[F], ReplayRhoRuntime[F])] =
    ???

  /*
   * Create from KeyValueStore's
	 *
	 * NOTE: NOT passing 'additionalSystemProcesses' parameter to rust side
   */
  def createRuntime[F[_]: Concurrent: ContextShift: Parallel: Log: Metrics: Span](
      // stores: RSpaceStore[F],
      storePath: String,
      mergeableTagName: Par,
      initRegistry: Boolean = false,
      additionalSystemProcesses: Seq[Definition[F]] = Seq.empty
  )(
      implicit scheduler: Scheduler
  ): F[RhoRuntime[F]] = {
    import coop.rchain.rholang.interpreter.storage._
    // implicit val m: Match[F, BindPattern, ListParWithRandom] = matchListPar[F]
    // for {
    //   // space <- RSpace
    //   //           .create[F, Par, BindPattern, ListParWithRandom, TaggedContinuation](
    //   //             stores
    //   //           )
    //   space <- RSpacePlusPlus_RhoTypes.create[F](storePath)
    //   runtime <- createRhoRuntime[F](
    //               space,
    //               mergeableTagName,
    //               initRegistry,
    //               additionalSystemProcesses
    //             )
    // } yield runtime
    ???
  }
}
