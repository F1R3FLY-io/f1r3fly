package coop.rchain.casper

import cats.data.EitherT
import cats.effect.{Concurrent, Sync, Timer}
import cats.syntax.all._
import com.google.protobuf.ByteString
import coop.rchain.blockstorage._
import coop.rchain.blockstorage.casperbuffer.CasperBufferStorage
import coop.rchain.blockstorage.dag.BlockDagStorage.DeployId
import coop.rchain.blockstorage.dag.{BlockDagRepresentation, BlockDagStorage}
import coop.rchain.blockstorage.deploy.DeployStorage
import coop.rchain.casper.bitcoin.{BitcoinAnchorService, F1r3flyStateCommitmentProto}
import coop.rchain.casper.engine.BlockRetriever
import coop.rchain.casper.finality.Finalizer
import coop.rchain.casper.merging.BlockIndex
import coop.rchain.casper.protocol._
import coop.rchain.casper.syntax._
import coop.rchain.casper.util.ProtoUtil._
import coop.rchain.casper.util._
import coop.rchain.casper.util.comm.CommUtil
import coop.rchain.casper.util.rholang._
import coop.rchain.catscontrib.Catscontrib.ToBooleanF
import coop.rchain.crypto.signatures.Signed
import coop.rchain.dag.DagOps
import coop.rchain.metrics.implicits._
import coop.rchain.metrics.{Metrics, Span}
import coop.rchain.models.BlockHash._
import coop.rchain.models.Validator.Validator
import coop.rchain.models.syntax._
import coop.rchain.models.{BlockHash => _, _}
import coop.rchain.rholang.interpreter.merging.RholangMergingLogic
// import coop.rchain.rspace.hashing.Blake2b256Hash
// import coop.rchain.rspace.internal
import coop.rchain.shared._
import coop.rchain.shared.syntax._

// format: off
class MultiParentCasperImpl[F[_]
  /* Execution */   : Concurrent: Time: Timer
  /* Transport */   : CommUtil: BlockRetriever: EventPublisher
  /* Rholang */     : RuntimeManager
  /* Casper */      : Estimator: SafetyOracle
  /* Storage */     : BlockStore: BlockDagStorage: DeployStorage: CasperBufferStorage
  /* Diagnostics */ : Log: Metrics: Span] // format: on
(
    validatorId: Option[ValidatorIdentity],
    // todo this should be read from chain, for now read from startup options
    casperShardConf: CasperShardConf,
    approvedBlock: BlockMessage,
    bitcoinAnchorService: Option[BitcoinAnchorService[F]] = None
) extends MultiParentCasper[F] {
  import MultiParentCasperImpl._

  implicit private val logSource: LogSource = LogSource(this.getClass)

  // TODO: Extract hardcoded version from shard config
  private val version = 1L

  def getValidator: F[Option[ValidatorIdentity]] = validatorId.pure[F]

  def getVersion: F[Long] = version.pure[F]

  def getApprovedBlock: F[BlockMessage] = approvedBlock.pure[F]

  private def updateLastFinalizedBlock(newBlock: BlockMessage): F[Unit] =
    lastFinalizedBlock.whenA(
      newBlock.body.state.blockNumber % casperShardConf.finalizationRate == 0
    )

  /**
    * Check if there are blocks in CasperBuffer available with all dependencies met.
    * @return First from the set of available blocks
    */
  override def getDependencyFreeFromBuffer: F[List[BlockMessage]] = {
    import cats.instances.list._
    for {
      pendants       <- CasperBufferStorage[F].getPendants
      pendantsStored <- pendants.toList.filterA(BlockStore[F].contains)
      depFreePendants <- pendantsStored.filterA { pendant =>
                          for {
                            pendantBlock   <- BlockStore[F].get(pendant)
                            justifications = pendantBlock.get.justifications
                            // If even one of justifications is not in DAG - block is not dependency free
                            missingDep <- justifications
                                           .map(_.latestBlockHash)
                                           .existsM(dagContains(_).not)
                          } yield !missingDep
                        }
      r <- depFreePendants.traverse(BlockStore[F].getUnsafe)
    } yield r
  }

  def dagContains(hash: BlockHash): F[Boolean] = blockDag.flatMap(_.contains(hash))

  def bufferContains(hash: BlockHash): F[Boolean] = CasperBufferStorage[F].contains(hash)

  def contains(hash: BlockHash): F[Boolean] = bufferContains(hash) ||^ dagContains(hash)

  def deploy(d: Signed[DeployData]): F[Either[DeployError, DeployId]] = {
    import coop.rchain.models.rholang.implicits._

    InterpreterUtil
      .mkTerm(d.data.term, NormalizerEnv(d))
      .bitraverse(
        err => DeployError.parsingError(s"Error in parsing term: \n$err").pure[F],
        _ => addDeploy(d)
      )
  }

  def addDeploy(deploy: Signed[DeployData]): F[DeployId] =
    for {
      _ <- DeployStorage[F].add(List(deploy))
      _ <- Log[F].info(s"Received ${PrettyPrinter.buildString(deploy)}")
    } yield deploy.sig

  /**
    * Calculates the deterministic index of a validator in the sorted bonds list.
    * Returns -1 if the validator is not found in the bonds.
    *
    * @param validatorId The validator identity to find
    * @param bonds The current bonds list from consensus
    * @return The validator's index in the sorted list, or -1 if not found
    */
  private def getValidatorIndex(
      validatorId: ValidatorIdentity,
      bonds: Seq[Bond]
  ): Int =
    try {
      // Validate inputs
      if (validatorId == null || validatorId.publicKey == null) {
        return -1
      }

      if (bonds == null || bonds.isEmpty) {
        return -1
      }

      // Sort bonds by validator public key for deterministic ordering
      val sortedBonds = bonds.sortBy(_.validator.toStringUtf8)
      val myPubKey    = ByteString.copyFrom(validatorId.publicKey.bytes)
      sortedBonds.indexWhere(_.validator == myPubKey)
    } catch {
      case _: Exception =>
        // Any error in validator index calculation should result in -1 (not found)
        -1
    }

  /**
    * Determines if this validator should perform Bitcoin anchoring for the given block.
    * Uses deterministic selection based on block height and validator bonds.
    *
    * @param blockHeight The height of the block being finalized
    * @param validatorId The optional validator identity of this node
    * @param bonds The current validator bonds from consensus
    * @return Either an error string or boolean indicating if this validator should anchor
    */
  private def shouldPerformAnchor(
      blockHeight: Long,
      validatorId: Option[ValidatorIdentity],
      bonds: Seq[Bond]
  ): Either[String, Boolean] = {
    // Validate block height
    if (blockHeight < 0) {
      return Left(s"Invalid block height: $blockHeight (must be >= 0)")
    }

    validatorId match {
      case None =>
        // Bootstrap/observer nodes never anchor (no ValidatorIdentity)
        Right(false)
      case Some(id) =>
        if (bonds.isEmpty) {
          // No validators in bonds, no anchoring possible
          Left("Empty bonds list - no validators available for anchoring")
        } else {
          // Validate bonds list
          val invalidBonds = bonds.filter(bond => bond.validator == null || bond.validator.isEmpty)
          if (invalidBonds.nonEmpty) {
            return Left(
              s"Invalid bonds detected: ${invalidBonds.length} bonds with null/empty validator keys"
            )
          }

          val myIndex = getValidatorIndex(id, bonds)
          if (myIndex < 0) {
            // This validator is not in the bonds list
            Right(false)
          } else {
            try {
              // Deterministic selection: validator at (blockHeight % bondsCount) anchors
              val selectedIndex = (blockHeight % bonds.length).toInt
              Right(selectedIndex == myIndex)
            } catch {
              case e: ArithmeticException =>
                Left(s"Arithmetic error in validator selection: ${e.getMessage}")
              case e: Exception =>
                Left(s"Unexpected error in validator selection: ${e.getMessage}")
            }
          }
        }
    }
  }

  /**
    * Anchors the finalized block state to Bitcoin.
    * Gathers the required commitment data and calls the Bitcoin anchor service.
    * Uses distributed coordination to ensure only one validator anchors per block.
    */
  private def anchorFinalizationToBitcoin(
      service: BitcoinAnchorService[F],
      lfbHash: BlockHash
  ): F[Unit] =
    for {
      // Get the finalized block
      lfbBlock <- BlockStore[F].getUnsafe(lfbHash)

      // Get post-state hash (RSpace root)
      rspaceRoot = ProtoUtil.postStateHash(lfbBlock)

      // Get block height and timestamp
      blockHeight = ProtoUtil.blockNumber(lfbBlock)
      timestamp   = lfbBlock.header.timestamp

      // Get current validator bonds to compute validator set hash
      bonds            <- RuntimeManager[F].computeBonds(rspaceRoot)
      validatorSetHash = BitcoinAnchorService.computeValidatorSetHash(bonds)

      // Distributed anchoring: check if this validator should anchor
      distributedMode = casperShardConf.bitcoinAnchorDistributed
      anchorResult = if (distributedMode) {
        shouldPerformAnchor(blockHeight, validatorId, bonds)
      } else {
        Right(true) // All-nodes behavior (current)
      }

      _ <- anchorResult match {
            case Left(error) =>
              Log[F].error(s"Bitcoin anchor validation failed for block $blockHeight: $error")
            case Right(_) =>
              ().pure[F]
          }

      shouldAnchor = anchorResult.getOrElse(false)

      _ <- if (shouldAnchor) {
            // This validator is selected - perform anchoring
            {
              val myIndex       = validatorId.map(id => getValidatorIndex(id, bonds)).getOrElse(-1)
              val selectedIndex = if (bonds.nonEmpty) (blockHeight % bonds.size).toInt else -1

              for {
                _ <- Log[F].info(
                      s"Bitcoin anchor: SELECTED validator for block $blockHeight " +
                        s"(my index: $myIndex, selected index: $selectedIndex, distributed: $distributedMode)"
                    )

                // Create state commitment
                commitment = F1r3flyStateCommitmentProto(
                  lfbHash = lfbHash,
                  rspaceRoot = rspaceRoot,
                  blockHeight = blockHeight,
                  timestamp = timestamp,
                  validatorSetHash = validatorSetHash
                )

                // Anchor to Bitcoin (fire-and-forget, with comprehensive error logging)
                _ <- service.anchorFinalization(commitment).attempt.flatMap {
                      case Left(error) =>
                        // Log detailed error information for troubleshooting
                        error match {
                          case timeout: java.util.concurrent.TimeoutException =>
                            Log[F].warn(
                              s"Bitcoin anchoring timeout for block ${lfbHash.toHexString}: ${error.getMessage}"
                            )
                          case network: java.net.ConnectException =>
                            Log[F].warn(
                              s"Bitcoin network connection failed for block ${lfbHash.toHexString}: ${error.getMessage}"
                            )
                          case _ =>
                            Log[F].warn(
                              s"Bitcoin anchoring failed for block ${lfbHash.toHexString}: ${error.getClass.getSimpleName}: ${error.getMessage}"
                            )
                        }
                      case Right(_) =>
                        Log[F].info(
                          s"Successfully anchored block ${lfbHash.toHexString} to Bitcoin (validator $myIndex)"
                        )
                    }
              } yield ()
            }
          } else {
            // Not selected - skip anchoring
            val reason = if (!distributedMode) {
              "distributed mode disabled"
            } else {
              validatorId match {
                case None => "not a validator (bootstrap/observer node)"
                case Some(id) =>
                  val myIndex = getValidatorIndex(id, bonds)
                  if (myIndex < 0) "validator not found in bonds"
                  else {
                    val selectedIndex = if (bonds.nonEmpty) (blockHeight % bonds.size).toInt else -1
                    s"not selected (my index: $myIndex, selected index: $selectedIndex)"
                  }
              }
            }
            Log[F].info(s"Bitcoin anchor: SKIPPED for block $blockHeight - $reason")
          }

    } yield ()

  def estimator(dag: BlockDagRepresentation[F]): F[IndexedSeq[BlockHash]] =
    Estimator[F].tips(dag, approvedBlock).map(_.tips)

  def lastFinalizedBlock: F[BlockMessage] = {

    def processFinalised(finalizedSet: Set[BlockHash]): F[Unit] =
      finalizedSet.toList.traverse { h =>
        for {
          block   <- BlockStore[F].getUnsafe(h)
          deploys = block.body.deploys.map(_.deploy)

          // Remove block deploys from persistent store
          deploysRemoved   <- DeployStorage[F].remove(deploys)
          finalizedSetStr  = PrettyPrinter.buildString(finalizedSet)
          removedDeployMsg = s"Removed $deploysRemoved deploys from deploy history as we finalized block $finalizedSetStr."
          _                <- Log[F].info(removedDeployMsg)

          // Remove block index from cache
          _ <- BlockIndex.cache.remove(h).pure

          // Remove block post-state mergeable channels from persistent store
          stateHash = block.body.state.postStateHash.toBlake2b256Hash.bytes
          _         <- RuntimeManager[F].getMergeableStore.delete(stateHash)
        } yield ()
      }.void

    def newLfbFoundEffect(newLfb: BlockHash): F[Unit] =
      for {
        // Record finalization and publish event (original logic)
        _ <- BlockDagStorage[F].recordDirectlyFinalized(newLfb, processFinalised)
        _ <- EventPublisher[F].publish(RChainEvent.blockFinalised(newLfb.toHexString))

        // Bitcoin anchoring (if enabled)
        _ <- bitcoinAnchorService.fold(Sync[F].unit) { service =>
              anchorFinalizationToBitcoin(service, newLfb)
            }
      } yield ()

    implicit val ms = CasperMetricsSource

    for {
      dag                      <- blockDag
      lastFinalizedBlockHash   = dag.lastFinalizedBlock
      lastFinalizedBlockHeight <- dag.lookupUnsafe(lastFinalizedBlockHash).map(_.blockNum)
      work = Finalizer
        .run[F](
          dag,
          casperShardConf.faultToleranceThreshold,
          lastFinalizedBlockHeight,
          newLfbFoundEffect
        )
      newFinalisedHashOpt <- Span[F].traceI("finalizer-run")(work)
      blockMessage        <- BlockStore[F].getUnsafe(newFinalisedHashOpt.getOrElse(lastFinalizedBlockHash))
    } yield blockMessage
  }

  def blockDag: F[BlockDagRepresentation[F]] =
    BlockDagStorage[F].getRepresentation

  def normalizedInitialFault(weights: Map[Validator, Long]): F[Float] =
    BlockDagStorage[F].accessEquivocationsTracker { tracker =>
      tracker.equivocationRecords.map { equivocations =>
        equivocations
          .map(_.equivocator)
          .flatMap(weights.get)
          .sum
          .toFloat / weightMapTotal(weights)
      }
    }

  def getRuntimeManager: F[RuntimeManager[F]] = Sync[F].delay(RuntimeManager[F])

  def fetchDependencies: F[Unit] = {
    import cats.instances.list._
    for {
      pendants       <- CasperBufferStorage[F].getPendants
      pendantsUnseen <- pendants.toList.filterA(BlockStore[F].contains(_).not)
      _ <- Log[F].debug(s"Requesting CasperBuffer pendant hashes, ${pendantsUnseen.size} items.") >>
            pendantsUnseen.toList.traverse_(
              dependency =>
                Log[F]
                  .debug(
                    s"Sending dependency ${PrettyPrinter.buildString(dependency)} to BlockRetriever"
                  ) >>
                  BlockRetriever[F].admitHash(
                    dependency,
                    admitHashReason = BlockRetriever.MissingDependencyRequested
                  )
            )
    } yield ()
  }

  override def getSnapshot: F[CasperSnapshot[F]] = {
    import cats.instances.list._

    def getOnChainState(b: BlockMessage): F[OnChainCasperState] =
      for {
        av <- RuntimeManager[F].getActiveValidators(b.body.state.postStateHash)
        // bonds are available in block message, but please remember this is just a cache, source of truth is RSpace.
        bm          = b.body.state.bonds
        shardConfig = casperShardConf
      } yield OnChainCasperState(shardConfig, bm.map(v => v.validator -> v.stake).toMap, av)

    for {
      dag         <- BlockDagStorage[F].getRepresentation
      r           <- Estimator[F].tips(dag, approvedBlock)
      (lca, tips) = (r.lca, r.tips)

      /**
        * Before block merge, `EstimatorHelper.chooseNonConflicting` were used to filter parents, as we could not
        * have conflicting parents. With introducing block merge, all parents that share the same bonds map
        * should be parents. Parents that have different bond maps are only one that cannot be merged in any way.
        */
      parents <- for {
                  // For now main parent bonds map taken as a reference, but might be we want to pick a subset with equal
                  // bond maps that has biggest cumulative stake.
                  blocks  <- tips.toList.traverse(BlockStore[F].getUnsafe)
                  parents = blocks.filter(b => b.body.state.bonds == blocks.head.body.state.bonds)
                } yield parents
      onChainState <- getOnChainState(parents.head)

      /**
        * We ensure that only the justifications given in the block are those
        * which are bonded validators in the chosen parent. This is safe because
        * any latest message not from a bonded validator will not change the
        * final fork-choice.
        */
      justifications <- {
        for {
          lms <- dag.latestMessages
          r = lms.toList
            .map {
              case (validator, blockMetadata) => Justification(validator, blockMetadata.blockHash)
            }
            .filter(j => onChainState.bondsMap.keySet.contains(j.validator))
        } yield r.toSet
      }
      parentMetas <- parents.traverse(b => dag.lookupUnsafe(b.blockHash))
      maxBlockNum = ProtoUtil.maxBlockNumberMetadata(parentMetas)
      maxSeqNums  <- dag.latestMessages.map(m => m.map { case (k, v) => k -> v.seqNum })
      deploysInScope <- {
        val currentBlockNumber  = maxBlockNum + 1
        val earliestBlockNumber = currentBlockNumber - onChainState.shardConf.deployLifespan
        for {
          result <- DagOps
                     .bfTraverseF[F, BlockMetadata](parentMetas)(
                       b =>
                         ProtoUtil
                           .getParentMetadatasAboveBlockNumber(
                             b,
                             earliestBlockNumber,
                             dag
                           )
                     )
                     .foldLeftF(Set.empty[Signed[DeployData]]) { (deploys, blockMetadata) =>
                       for {
                         block        <- BlockStore[F].getUnsafe(blockMetadata.blockHash)
                         blockDeploys = ProtoUtil.deploys(block).map(_.deploy)
                       } yield deploys ++ blockDeploys
                     }
        } yield result
      }
      invalidBlocks <- dag.invalidBlocksMap
      lfb           = dag.lastFinalizedBlock
    } yield CasperSnapshot(
      dag,
      lfb,
      lca,
      tips,
      parents,
      justifications,
      invalidBlocks,
      deploysInScope,
      maxBlockNum,
      maxSeqNums,
      onChainState
    )
  }

  override def validate(
      b: BlockMessage,
      s: CasperSnapshot[F]
  ): F[Either[BlockError, ValidBlock]] = {
    val validationProcess: EitherT[F, BlockError, ValidBlock] =
      for {
        _ <- EitherT(
              Validate
                .blockSummary(b, approvedBlock, s, casperShardConf.shardName, deployLifespan)
            )
        _ <- EitherT.liftF(Span[F].mark("post-validation-block-summary"))
        _ <- EitherT(
              InterpreterUtil
                .validateBlockCheckpoint(b, s, RuntimeManager[F])
                .map {
                  case Left(ex)       => Left(ex)
                  case Right(Some(_)) => Right(BlockStatus.valid)
                  case Right(None)    => Left(BlockStatus.invalidTransaction)
                }
            )
        _ <- EitherT.liftF(Span[F].mark("transactions-validated"))
        _ <- EitherT(Validate.bondsCache(b, RuntimeManager[F]))
        _ <- EitherT.liftF(Span[F].mark("bonds-cache-validated"))
        _ <- EitherT(Validate.neglectedInvalidBlock(b, s))
        _ <- EitherT.liftF(Span[F].mark("neglected-invalid-block-validated"))
        _ <- EitherT(
              EquivocationDetector.checkNeglectedEquivocationsWithUpdate(b, s.dag, approvedBlock)
            )
        _ <- EitherT.liftF(Span[F].mark("neglected-equivocation-validated"))

        // This validation is only to punish validator which accepted lower price deploys.
        // And this can happen if not configured correctly.
        minPhloPrice = casperShardConf.minPhloPrice
        _ <- EitherT(Validate.phloPrice(b, minPhloPrice)).recoverWith {
              case _ =>
                val warnToLog = EitherT.liftF[F, BlockError, Unit](
                  Log[F].warn(s"One or more deploys has phloPrice lower than $minPhloPrice")
                )
                val asValid = EitherT.rightT[F, BlockError](BlockStatus.valid)
                warnToLog *> asValid
            }
        _ <- EitherT.liftF(Span[F].mark("phlogiston-price-validated"))

        depDag <- EitherT.liftF(CasperBufferStorage[F].toDoublyLinkedDag)
        status <- EitherT(EquivocationDetector.checkEquivocations(depDag, b, s.dag))
        _      <- EitherT.liftF(Span[F].mark("equivocation-validated"))
      } yield status

    val blockPreState  = b.body.state.preStateHash
    val blockPostState = b.body.state.postStateHash
    val blockSender    = b.sender.toByteArray
    val indexBlock = for {
      mergeableChs <- RuntimeManager[F].loadMergeableChannels(blockPostState, blockSender, b.seqNum)

      index <- BlockIndex(
                b.blockHash,
                b.body.deploys,
                b.body.systemDeploys,
                blockPreState.toBlake2b256Hash,
                blockPostState.toBlake2b256Hash,
                RuntimeManager[F].getHistoryRepo,
                mergeableChs
              )
      _ = BlockIndex.cache.putIfAbsent(b.blockHash, index)
    } yield ()

    val validationProcessDiag = for {
      // Create block and measure duration
      r                    <- Stopwatch.duration(validationProcess.value)
      (valResult, elapsed) = r
      _ <- valResult
            .map { status =>
              val blockInfo   = PrettyPrinter.buildString(b, short = true)
              val deployCount = b.body.deploys.size
              Log[F].info(s"Block replayed: $blockInfo (${deployCount}d) ($status) [$elapsed]") <*
                indexBlock.whenA(casperShardConf.maxNumberOfParents > 1)
            }
            .getOrElse(().pure[F])
    } yield valResult

    Log[F].info(s"Validating block ${PrettyPrinter.buildString(b, short = true)}.") *> validationProcessDiag
  }

  override def handleValidBlock(block: BlockMessage): F[BlockDagRepresentation[F]] =
    for {
      updatedDag <- BlockDagStorage[F].insert(block, invalid = false)
      _          <- CasperBufferStorage[F].remove(block.blockHash)
      _          <- updateLastFinalizedBlock(block)
    } yield updatedDag

  override def handleInvalidBlock(
      block: BlockMessage,
      status: InvalidBlock,
      dag: BlockDagRepresentation[F]
  ): F[BlockDagRepresentation[F]] = {
    // TODO: Slash block for status except InvalidUnslashableBlock
    def handleInvalidBlockEffect(
        status: BlockError,
        block: BlockMessage
    ): F[BlockDagRepresentation[F]] =
      for {
        _ <- Log[F].warn(
              s"Recording invalid block ${PrettyPrinter.buildString(block.blockHash)} for ${status.toString}."
            )
        // TODO should be nice to have this transition of a block from casper buffer to dag storage atomic
        r <- BlockDagStorage[F].insert(block, invalid = true)
        _ <- CasperBufferStorage[F].remove(block.blockHash)
      } yield r

    status match {
      case InvalidBlock.AdmissibleEquivocation =>
        val baseEquivocationBlockSeqNum = block.seqNum - 1
        for {
          _ <- BlockDagStorage[F].accessEquivocationsTracker { tracker =>
                for {
                  equivocations <- tracker.equivocationRecords
                  _ <- Sync[F].unlessA(equivocations.exists {
                        case EquivocationRecord(validator, seqNum, _) =>
                          block.sender == validator && baseEquivocationBlockSeqNum == seqNum
                        // More than 2 equivocating children from base equivocation block and base block has already been recorded
                      }) {
                        val newEquivocationRecord =
                          EquivocationRecord(
                            block.sender,
                            baseEquivocationBlockSeqNum,
                            Set.empty[BlockHash]
                          )
                        tracker.insertEquivocationRecord(newEquivocationRecord)
                      }
                } yield ()
              }
          // We can only treat admissible equivocations as invalid blocks if
          // casper is single threaded.
          updatedDag <- handleInvalidBlockEffect(InvalidBlock.AdmissibleEquivocation, block)
        } yield updatedDag

      case InvalidBlock.IgnorableEquivocation =>
        /*
         * We don't have to include these blocks to the equivocation tracker because if any validator
         * will build off this side of the equivocation, we will get another attempt to add this block
         * through the admissible equivocations.
         */
        Log[F]
          .info(
            s"Did not add block ${PrettyPrinter.buildString(block.blockHash)} as that would add an equivocation to the BlockDAG"
          )
          .as(dag)

      case ib: InvalidBlock if InvalidBlock.isSlashable(ib) =>
        handleInvalidBlockEffect(ib, block)

      case ib: InvalidBlock =>
        CasperBufferStorage[F].remove(block.blockHash) >> Log[F]
          .warn(
            s"Recording invalid block ${PrettyPrinter.buildString(block.blockHash)} for $ib."
          )
          .as(dag)
    }
  }
}

object MultiParentCasperImpl {

  // TODO: Extract hardcoded deployLifespan from shard config
  // Size of deploy safety range.
  // Validators will try to put deploy in a block only for next `deployLifespan` blocks.
  // Required to enable protection from re-submitting duplicate deploys
  val deployLifespan = 50

  def addedEvent(block: BlockMessage): RChainEvent = {
    val (blockHash, parents, justifications, deployIds, creator, seqNum) = blockEvent(block)
    RChainEvent.blockAdded(
      blockHash,
      parents,
      justifications,
      deployIds,
      creator,
      seqNum
    )
  }

  def createdEvent(b: BlockMessage): RChainEvent = {
    val (blockHash, parents, justifications, deployIds, creator, seqNum) = blockEvent(b)
    RChainEvent.blockCreated(
      blockHash,
      parents,
      justifications,
      deployIds,
      creator,
      seqNum
    )
  }

  private def blockEvent(block: BlockMessage) = {

    val blockHash = block.blockHash.toHexString
    val parentHashes =
      block.header.parentsHashList.map(_.toHexString)
    val justificationHashes =
      block.justifications.toList
        .map(j => (j.validator.toHexString, j.latestBlockHash.toHexString))
    val deployIds: List[String] =
      block.body.deploys.map(pd => PrettyPrinter.buildStringNoLimit(pd.deploy.sig))
    val creator = block.sender.toHexString
    val seqNum  = block.seqNum
    (blockHash, parentHashes, justificationHashes, deployIds, creator, seqNum)
  }
}
