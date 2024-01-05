import $ivy.`org.scodec::scodec-core:1.11.7`
import $ivy.`org.bouncycastle:bcprov-jdk15on:1.68`
import $cp.^.shared.target.`scala-2.12`.classes
import $cp.^.crypto.target.`scala-2.12`.classes

@

import java.io.File
import scodec.bits.ByteVector
import coop.rchain.crypto.util.CertificateHelper._

@main
def main(path: os.Path) = {
  val keyFile: File = path.toNIO.toFile
  val keyPair       = readKeyPair(keyFile)
  ByteVector(publicAddress(keyPair.getPublic()).get).toHex
}
