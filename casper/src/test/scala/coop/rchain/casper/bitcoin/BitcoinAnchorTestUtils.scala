package coop.rchain.casper.bitcoin

import java.nio.file.Paths
import scala.concurrent.duration._
import com.google.protobuf.ByteString
import coop.rchain.casper.{
  BitcoinAnchorConf,
  CasperConf,
  GenesisBlockData,
  GenesisCeremonyConf,
  RoundRobinDispatcher
}
import coop.rchain.casper.util.GenesisBuilder

object BitcoinAnchorTestUtils {

  def validConfig: BitcoinAnchorConfigProto = BitcoinAnchorConfigProto(
    network = "regtest",
    enabled = true,
    esploraUrl = "",
    feeRate = 1.0,
    maxFeeSats = 100000L
  )

  def invalidConfig: BitcoinAnchorConfigProto = BitcoinAnchorConfigProto(
    network = "invalid_network",
    enabled = true,
    esploraUrl = "",
    feeRate = -1.0,
    maxFeeSats = -1L
  )

  def enabledCasperConf: CasperConf = CasperConf(
    faultToleranceThreshold = 0.0f,
    validatorPublicKey = None,
    validatorPrivateKey = None,
    validatorPrivateKeyPath = None,
    shardName = "test",
    parentShardId = "/",
    casperLoopInterval = 30.seconds,
    requestedBlocksTimeout = 240.seconds,
    finalizationRate = 1,
    maxNumberOfParents = 2,
    maxParentDepth = Some(2147483647),
    forkChoiceStaleThreshold = 10.minutes,
    forkChoiceCheckIfStaleInterval = 11.minutes,
    synchronyConstraintThreshold = 0.67,
    heightConstraintThreshold = 1000L,
    roundRobinDispatcher = RoundRobinDispatcher(
      maxPeerQueueSize = 100,
      giveUpAfterSkipped = 0,
      dropPeerAfterRetries = 0
    ),
    genesisBlockData = GenesisBlockData(
      genesisDataDir = Paths.get("/tmp/test"),
      bondsFile = "/tmp/test/bonds.txt",
      walletsFile = "/tmp/test/wallets.txt",
      bondMaximum = 9223372036854775807L,
      bondMinimum = 1,
      epochLength = 10000,
      quarantineLength = 50000,
      numberOfActiveValidators = 100,
      deployTimestamp = None,
      genesisBlockNumber = 0,
      posMultiSigPublicKeys = GenesisBuilder.defaultPosMultiSigPublicKeys,
      posMultiSigQuorum = GenesisBuilder.defaultPosMultiSigPublicKeys.length - 1
    ),
    genesisCeremony = GenesisCeremonyConf(
      requiredSignatures = 0,
      approveDuration = 5.minutes,
      approveInterval = 5.minute,
      autogenShardSize = 5,
      genesisValidatorMode = false,
      ceremonyMasterMode = false
    ),
    minPhloPrice = 1L,
    bitcoinAnchor = BitcoinAnchorConf(
      enabled = true,
      network = "regtest",
      esploraUrl = None,
      feeRate = Some(1.0),
      maxFeeSats = Some(100000L)
    )
  )

  def disabledCasperConf: CasperConf = enabledCasperConf.copy(
    bitcoinAnchor = BitcoinAnchorConf(
      enabled = false,
      network = "regtest",
      esploraUrl = None,
      feeRate = None,
      maxFeeSats = None
    )
  )

  def create32ByteHash(prefix: String): ByteString = {
    val bytes  = prefix.getBytes("UTF-8")
    val padded = bytes ++ Array.fill(32 - bytes.length)(0.toByte)
    ByteString.copyFrom(padded.take(32))
  }

  def createValidCommitment(): F1r3flyStateCommitmentProto =
    F1r3flyStateCommitmentProto(
      lfbHash = create32ByteHash("test_lfb_hash"),
      rspaceRoot = create32ByteHash("test_rspace_root"),
      blockHeight = 12345L,
      timestamp = System.currentTimeMillis(),
      validatorSetHash = create32ByteHash("test_validator_set")
    )
}
