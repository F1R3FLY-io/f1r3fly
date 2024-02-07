import $ivy.`org.typelevel::cats-effect:2.5.4`
import $ivy.`org.bitcoin-s::bitcoin-s-crypto:1.9.3`
import $ivy.`org.scorexfoundation::scrypto:2.2.1`
import $ivy.`com.lihaoyi::os-lib:0.9.2`
import $ivy.`com.beachape::enumeratum:1.5.13`
import $ivy.`com.thesamet.scalapb::scalapb-json4s:0.11.0`
import $ivy.`io.circe::circe-literal:0.14.6`
import $ivy.`io.circe::circe-parser:0.14.6`
import $ivy.`org.typelevel::jawn-parser:1.5.1`
import $ivy.`org.scodec::scodec-core:1.11.7`

@

/* bitcoin-s-crypto for the Secp256k1 elliptic curve cipher
 * scrypto for Blake2b
 * os-lib for filesystem manipulation
 * scalapb-json4s hauls in json4s, which we use for to-ing and fro-ing JSON
 * circe-literal is for making it easy to use JSON in the Ammonite REPL
 */

