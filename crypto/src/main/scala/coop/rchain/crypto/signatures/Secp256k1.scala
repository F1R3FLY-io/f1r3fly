package coop.rchain.crypto.signatures

import java.io.FileReader
import java.nio.file.Path
import java.security.KeyPairGenerator
import java.security.interfaces.ECPrivateKey
import java.security.spec.ECGenParameterSpec
import cats.effect.Sync
import cats.syntax.applicative._
import cats.syntax.flatMap._
import cats.syntax.functor._
import cats.syntax.applicativeError._
import coop.rchain.crypto.util.SecureRandomUtil
import coop.rchain.crypto.{PrivateKey, PublicKey}
import org.bitcoin._
import com.google.common.base.Strings
import coop.rchain.shared.Base16
import org.bouncycastle.asn1.DLSequence
import org.bouncycastle.openssl.bc.BcPEMDecryptorProvider
import org.bouncycastle.openssl.{PEMEncryptedKeyPair, PEMParser}

// TODO: refactor Signature API to handle exceptions from `NativeSecp256k1` library

import scodec.bits.ByteVector
import org.bitcoins.crypto.{
  CryptoRuntimeFactory,
  ECDigitalSignature,
  ECPrivateKeyBytes,
  ECPublicKeyBytes
}

object Secp256k1 extends SignaturesAlg {

  private val cr              = CryptoRuntimeFactory.newCryptoRuntime
  val name                    = "secp256k1"
  override val sigLength: Int = 32

  /**
    * Verifies the given secp256k1 signature in native code.
    *
    * @return (private key, public key) pair
    *
    */
  @SuppressWarnings(Array("org.wartremover.warts.AsInstanceOf"))
  def newKeyPair: (PrivateKey, PublicKey) = {
    val priv = cr.freshPrivateKey
    val pub  = cr.toPublicKey(priv)

    (PrivateKey(priv.bytes.toArray), PublicKey(pub.decompressedBytes.toArray))
  }

  def parsePemFile[F[_]: Sync](path: Path, password: String): F[PrivateKey] = {
    import scala.collection.JavaConverters._

    def handleWith[A](f: => A, message: String): F[A] =
      Sync[F].delay(f).recoverWith { case t => Sync[F].raiseError(new Exception(message, t)) }

    val parser = new PEMParser(new FileReader(path.toFile))
    for {
      encryptedKeyPair <- handleWith(
                           parser.readObject().asInstanceOf[PEMEncryptedKeyPair],
                           "PEM file is not encrypted"
                         )
      decryptor = new BcPEMDecryptorProvider(password.toCharArray)
      keyPair <- handleWith(
                  encryptedKeyPair.decryptKeyPair(decryptor),
                  "Could not decrypt PEM file"
                )
      dlSequence <- handleWith(
                     keyPair.getPrivateKeyInfo.parsePrivateKey.asInstanceOf[DLSequence],
                     "Could not parse private key from PEM file"
                   )
      privateKeyOpt = dlSequence.iterator().asScala.collectFirst {
        case octet: org.bouncycastle.asn1.DEROctetString => PrivateKey(octet.getOctets)
      }
      privateKey <- privateKeyOpt match {
                     case Some(privateKey) => privateKey.pure[F]
                     case None =>
                       Sync[F].raiseError[PrivateKey](
                         new Exception("PEM file does not contain private key")
                       )
                   }
    } yield privateKey
  }

  //TODO: refactor to make use of strongly typed keys
  /**
    * Verifies the given secp256k1 signature in native code.
    *
    * Input values
    * @param data The data which was signed, must be exactly 32 bytes
    * @param signature The signature
    * @param pub The public key which did the signing
    *
    * Return value
    * boolean value of verification
    *
    */
  def verify(
      data: Array[Byte],
      signature: Array[Byte],
      pub: Array[Byte]
  ): Boolean = {
    // WARNING: this code throws Assertion exception if input is not correct length
    val pk    = ECPublicKeyBytes.fromBytes(ByteVector(pub))
    val sig   = ECDigitalSignature.fromBytes(ByteVector(signature))
    val stuff = ByteVector(data)
    cr.verify(pk, stuff, sig)
  }

  /**
    * libsecp256k1 Create an ECDSA signature.
    *
    * Input values
    * @param data Message hash, 32 bytes
    * @param sec Secret key, 32 bytes
    *
    * Return value
    * byte array of signature
    *
    */
  def sign(
      data: Array[Byte],
      sec: Array[Byte]
  ): Array[Byte] = {
    // WARNING: this code throws Assertion exception if input is not correct length
    val pk    = ECPrivateKeyBytes.fromBytes(ByteVector(sec)).toPrivateKey
    val stuff = ByteVector(data)
    cr.sign(pk, stuff).bytes.toArray
  }

  /**
    * libsecp256k1 Seckey Verify - returns true if valid, false if invalid
    *
    * Input value
    * @param seckey ECDSA Secret key, 32 bytes
    *
    * Return value
    * Boolean of secret key verification
    */
  def secKeyVerify(seckey: Array[Byte]): Boolean =
    // WARNING: this code throws Assertion exception if input is not correct length
    cr.secKeyVerify(ByteVector(seckey))

  /**
    * libsecp256k1 Compute Pubkey - computes public key from secret key
    *
    * @param seckey ECDSA Secret key, 32 bytes
    *
    * Return values
    * @param pubkey ECDSA Public key, 33 or 65 bytes
    */
  def toPublic(seckey: Array[Byte]): Array[Byte] = {
    // WARNING: this code throws Assertion exception if input is not correct length
    val pub = cr.publicKey(ECPrivateKeyBytes.fromBytes(ByteVector(seckey)).toPrivateKey)
    pub.decompressedBytes.toArray
  }

  override def toPublic(sec: PrivateKey): PublicKey = PublicKey(toPublic(sec.bytes))
}
