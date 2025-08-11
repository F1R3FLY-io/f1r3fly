package coop.rchain.casper.bitcoin

import cats.effect.{Resource, Sync}
import cats.syntax.all._
import com.google.protobuf.ByteString
import com.sun.jna.{Memory, Pointer}
import coop.rchain.casper.{BitcoinAnchorConf, CasperConf}
import coop.rchain.casper.protocol.Bond
import coop.rchain.crypto.hash.Blake2b256
import coop.rchain.shared.Log
import java.nio.charset.StandardCharsets

/**
  * Service for anchoring F1r3fly state commitments to Bitcoin Layer 1.
  *
  * This service provides a type-safe, resource-managed wrapper around the
  * native Bitcoin anchor FFI library. It handles the lifecycle of the native
  * anchor instance and provides proper error handling and logging.
  */
trait BitcoinAnchorService[F[_]] {

  /**
    * Anchors a finalized F1r3fly state commitment to Bitcoin.
    *
    * @param commitment The state commitment containing LFB hash, RSpace root, etc.
    * @return Result containing transaction details or error information
    */
  def anchorFinalization(
      commitment: F1r3flyStateCommitmentProto
  ): F[BitcoinAnchorResultProto]

  /**
    * Shutdown the service and clean up resources.
    */
  def shutdown: F[Unit]
}

object BitcoinAnchorService {

  /**
    * Creates a resource-managed BitcoinAnchorService instance.
    *
    * The returned Resource ensures proper cleanup of native resources
    * when the service is no longer needed.
    */
  def apply[F[_]: Sync: Log](
      config: BitcoinAnchorConfigProto
  ): Resource[F, BitcoinAnchorService[F]] =
    Resource.make(
      create(config)
    )(
      _.shutdown
    )

  /**
    * Create a Resource-managed BitcoinAnchorService for engine use.
    *
    * This method handles the Option[Resource[F, T]] -> Resource[F, Option[T]] conversion
    * needed for safe engine lifecycle management.
    *
    * @param casperConf The Casper configuration
    * @return Resource that manages an optional Bitcoin anchor service
    */
  def createForEngine[F[_]: Sync: Log](
      casperConf: CasperConf
  ): Resource[F, Option[BitcoinAnchorService[F]]] =
    Resource.eval(fromCasperConf(casperConf)).flatMap {
      case Some(serviceResource) => serviceResource.map(Some(_))
      case None                  => Resource.pure[F, Option[BitcoinAnchorService[F]]](None)
    }

  /**
    * Create a Resource-managed BitcoinAnchorService from BitcoinAnchorConf.
    *
    * This is used in engine components that have access to Bitcoin anchor configuration
    * but not the full CasperConf.
    *
    * @param bitcoinConf The Bitcoin anchor configuration
    * @return Resource that manages an optional Bitcoin anchor service
    */
  def fromBitcoinAnchorConf[F[_]: Sync: Log](
      bitcoinConf: coop.rchain.casper.BitcoinAnchorConf
  ): Resource[F, Option[BitcoinAnchorService[F]]] =
    if (!bitcoinConf.enabled) {
      Resource.eval(Log[F].info("Bitcoin anchoring is disabled")) >>
        Resource.pure[F, Option[BitcoinAnchorService[F]]](None)
    } else {
      for {
        _ <- Resource.eval(
              Log[F].info(
                s"Creating Bitcoin anchor service from BitcoinAnchorConf: network=${bitcoinConf.network}"
              )
            )
        protoConfig = BitcoinAnchorConfigProto(
          network = bitcoinConf.network,
          enabled = bitcoinConf.enabled,
          esploraUrl = bitcoinConf.esploraUrl.getOrElse(""),
          feeRate = bitcoinConf.feeRate.getOrElse(0.0),
          maxFeeSats = bitcoinConf.maxFeeSats.getOrElse(0L)
        )
        service <- apply(protoConfig).map(Some(_))
      } yield service
    }

  /**
    * Create a disabled Bitcoin anchor service for engine use.
    *
    * This is used in engine components that don't have access to configuration
    * but need to maintain the Resource pattern for memory safety.
    *
    * @return Resource that manages a disabled (None) Bitcoin anchor service
    */
  def disabledForEngine[F[_]: Sync]: Resource[F, Option[BitcoinAnchorService[F]]] =
    Resource.pure[F, Option[BitcoinAnchorService[F]]](None)

  /**
    * Factory method to create a BitcoinAnchorService from CasperConf.
    *
    * Returns None if Bitcoin anchoring is disabled, or Some(service) if enabled.
    * Validates configuration and provides appropriate error messages.
    */
  def fromCasperConf[F[_]: Sync: Log](
      casperConf: CasperConf
  ): F[Option[Resource[F, BitcoinAnchorService[F]]]] = {
    val bitcoinConf = casperConf.bitcoinAnchor

    if (!bitcoinConf.enabled) {
      Log[F].info("Bitcoin anchoring is disabled") >>
        (None: Option[Resource[F, BitcoinAnchorService[F]]]).pure[F]
    } else {
      for {
        _ <- Log[F].info(s"Initializing Bitcoin anchoring for network: ${bitcoinConf.network}")

        // Validate network
        _ <- bitcoinConf.network match {
              case "mainnet" | "signet" | "regtest" => Sync[F].unit
              case invalid =>
                Sync[F].raiseError(
                  new IllegalArgumentException(
                    s"Invalid Bitcoin network: $invalid. Must be 'mainnet', 'signet', or 'regtest'"
                  )
                )
            }

        // Validate fee rate if specified
        _ <- bitcoinConf.feeRate.fold(Sync[F].unit) { rate =>
              if (rate > 0) Sync[F].unit
              else
                Sync[F].raiseError(
                  new IllegalArgumentException(
                    s"Fee rate must be positive, got: $rate"
                  )
                )
            }

        // Validate max fee if specified
        _ <- bitcoinConf.maxFeeSats.fold(Sync[F].unit) { maxFee =>
              if (maxFee > 0) Sync[F].unit
              else
                Sync[F].raiseError(
                  new IllegalArgumentException(
                    s"Max fee must be positive, got: $maxFee"
                  )
                )
            }

        // Create protobuf config
        protoConfig = BitcoinAnchorConfigProto(
          network = bitcoinConf.network,
          enabled = bitcoinConf.enabled,
          esploraUrl = bitcoinConf.esploraUrl.getOrElse(""),
          feeRate = bitcoinConf.feeRate.getOrElse(0.0),
          maxFeeSats = bitcoinConf.maxFeeSats.getOrElse(0L)
        )

        // Create service resource
        serviceResource = apply(protoConfig)

        _ <- Log[F].info("Bitcoin anchor service factory created successfully")

      } yield Some(serviceResource)
    }
  }

  /**
    * Computes a deterministic hash of the validator set from their bonds.
    *
    * This function creates a stable hash representing the active validator set
    * by sorting validators by their public key and hashing the concatenated
    * validator-stake pairs. This ensures consistent hashing across different
    * nodes and over time.
    *
    * @param bonds The validator bonds containing public keys and stakes
    * @return 32-byte hash representing the validator set
    */
  def computeValidatorSetHash(bonds: Seq[Bond]): ByteString = {
    // Sort bonds by validator public key (as hex string) for deterministic ordering
    val sortedBonds = bonds.sortBy(_.validator.toStringUtf8)

    // Create concatenated bytes: validator1 + stake1 + validator2 + stake2 + ...
    val concatenated = sortedBonds.foldLeft(Array.empty[Byte]) { (acc, bond) =>
      val validatorBytes = bond.validator.toByteArray
      val stakeBytes     = BigInt(bond.stake).toByteArray
      acc ++ validatorBytes ++ stakeBytes
    }

    // Hash the concatenated data
    val hash = Blake2b256.hash(concatenated)
    ByteString.copyFrom(hash)
  }

  private def create[F[_]: Sync: Log](
      config: BitcoinAnchorConfigProto
  ): F[BitcoinAnchorService[F]] =
    for {
      _ <- Log[F].info(
            s"Initializing Bitcoin Anchor Service: " +
              s"network=${config.network}, " +
              s"esploraUrl=${if (config.esploraUrl.nonEmpty) config.esploraUrl else "default"}, " +
              s"feeRate=${if (config.feeRate > 0) config.feeRate.toString + " sat/vB" else "auto"}, " +
              s"maxFeeSats=${if (config.maxFeeSats > 0) config.maxFeeSats.toString else "unlimited"}"
          )
      anchorPtr <- createAnchorInstance(config)
      _ <- Log[F].info(
            s"Bitcoin Anchor Service initialized successfully for network: ${config.network}"
          )
    } yield new BitcoinAnchorServiceImpl[F](anchorPtr)

  private def createAnchorInstance[F[_]: Sync](
      config: BitcoinAnchorConfigProto
  ): F[Pointer] =
    for {
      configBytes <- Sync[F].delay(config.toByteArray)
      configMem   <- Sync[F].delay(new Memory(configBytes.length))
      _           <- Sync[F].delay(configMem.write(0, configBytes, 0, configBytes.length))
      anchorPtr <- Sync[F].delay {
                    val ptr = BitcoinAnchorJNAInterface.instance.create_bitcoin_anchor(
                      configMem,
                      configBytes.length
                    )
                    if (ptr == Pointer.NULL) {
                      throw new RuntimeException(
                        s"Failed to create Bitcoin anchor instance. " +
                          s"This may be due to invalid configuration or missing native library. " +
                          s"Ensure libbitcoin_anchor_ffi.so is in the library path and " +
                          s"the configuration is valid."
                      )
                    }
                    ptr
                  }
    } yield anchorPtr
}

/**
  * Implementation of BitcoinAnchorService that wraps the native FFI calls.
  */
private class BitcoinAnchorServiceImpl[F[_]: Sync: Log](
    private val anchorPtr: Pointer
) extends BitcoinAnchorService[F] {

  def anchorFinalization(
      commitment: F1r3flyStateCommitmentProto
  ): F[BitcoinAnchorResultProto] =
    for {
      _ <- Log[F].debug(s"Anchoring finalization for block height ${commitment.blockHeight}")

      // Serialize commitment to protobuf bytes
      stateBytes <- Sync[F].delay(commitment.toByteArray)
      stateMem   <- Sync[F].delay(new Memory(stateBytes.length))
      _          <- Sync[F].delay(stateMem.write(0, stateBytes, 0, stateBytes.length))

      // Call native function
      resultPtr <- Sync[F].delay {
                    BitcoinAnchorJNAInterface.instance.anchor_finalization(
                      anchorPtr,
                      stateMem,
                      stateBytes.length
                    )
                  }

      // Handle null result
      _ <- if (resultPtr == Pointer.NULL) {
            Log[F].error(
              s"Bitcoin anchor finalization returned null for commitment: " +
                s"lfbHash=${commitment.lfbHash.toStringUtf8}, " +
                s"blockHeight=${commitment.blockHeight}, " +
                s"timestamp=${commitment.timestamp}. " +
                s"This may indicate network issues or invalid configuration."
            ) >> Sync[F].raiseError(
              new RuntimeException(
                "Bitcoin anchor finalization failed. " +
                  "Check that the Bitcoin network is accessible and configuration is correct."
              )
            )
          } else {
            Log[F].debug(
              s"Bitcoin anchor finalization succeeded for block height ${commitment.blockHeight}"
            )
          }

      // Read result length (first 4 bytes)
      resultLen <- Sync[F].delay(resultPtr.getInt(0))

      // Read result data (skip first 4 bytes which contain length)
      resultBytes <- Sync[F].delay {
                      val bytes = new Array[Byte](resultLen)
                      resultPtr.read(4, bytes, 0, resultLen)
                      bytes
                    }

      // Parse protobuf result
      result <- Sync[F].delay(BitcoinAnchorResultProto.parseFrom(resultBytes))

      // Deallocate native memory
      _ <- Sync[F].delay {
            BitcoinAnchorJNAInterface.instance.deallocate_memory(
              resultPtr,
              resultLen + 4 // Include the 4-byte length prefix
            )
          }

      _ <- if (result.success) {
            Log[F].info(
              s"Successfully anchored finalization to Bitcoin. " +
                s"Transaction: ${result.transactionId}, Fee: ${result.feeSats} sats"
            )
          } else {
            Log[F].warn(
              s"Bitcoin anchor finalization failed: ${result.errorMessage}"
            )
          }

    } yield result

  def shutdown: F[Unit] =
    for {
      _ <- Log[F].info("Shutting down Bitcoin Anchor Service")
      _ <- Sync[F].delay {
            BitcoinAnchorJNAInterface.instance.destroy_bitcoin_anchor(anchorPtr)
          }
    } yield ()
}
