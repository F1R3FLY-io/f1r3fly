import $ivy.`org.bitcoin-s::bitcoin-s-crypto:1.9.3`
import $ivy.`org.scodec::scodec-core:1.11.7`
import $ivy.`org.bouncycastle:bcprov-jdk15on:1.68`
import $cp.^.shared.target.`scala-2.12`.classes
import $cp.^.crypto.target.`scala-2.12`.classes

@

import java.io.File
import java.security.{ AlgorithmParameters, PublicKey => JPublicKey }
import java.security.interfaces.{ ECPublicKey => JECPublicKey }
import java.security.spec.{ AlgorithmParameterSpec, ECParameterSpec, ECGenParameterSpec }
import scodec.bits._
import coop.rchain.crypto.util.ParameterSpec
import coop.rchain.crypto.util.CertificateHelper._

@main
def main(path: os.Path) = {
  val EllipticCurveName = "secp256r1"

  lazy val EllipticCurveParameterSpec: ParameterSpec = {
    val ap = AlgorithmParameters.getInstance("EC")
    ap.init(new ECGenParameterSpec(EllipticCurveName))
    ParameterSpec(ap.getParameterSpec(classOf[ECParameterSpec]))
  }

  def isExpectedEllipticCurve(publicKey: JPublicKey): Boolean =
    publicKey match {
      case p: JECPublicKey =>
        ParameterSpec(p.getParams) == EllipticCurveParameterSpec
      case _ => false
    }

  def publicKey(publicKey: JPublicKey): ByteVector =
    publicKey match {
      case p: JECPublicKey if isExpectedEllipticCurve(publicKey) =>
        val publicKey = Array.ofDim[Byte](64)
        val x         = p.getW.getAffineX.toByteArray.takeRight(32)
        val y         = p.getW.getAffineY.toByteArray.takeRight(32)
        x.copyToArray(publicKey, 32 - x.length)
        y.copyToArray(publicKey, 64 - y.length)
        hex"04" ++ ByteVector(publicKey)
    }

  val keyFile: File = path.toNIO.toFile
  val keyPair       = readKeyPair(keyFile)
  publicKey(keyPair.getPublic()).toHex
}
