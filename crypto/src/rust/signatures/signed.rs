use crate::rust::public_key::PublicKey;

use super::signatures_alg::SignaturesAlg;

// See crypto/src/main/scala/coop/rchain/crypto/signatures/Signed.scala
pub struct Signed<A> {
    pub data: A,
    pub pk: PublicKey,
    pub sig: Vec<u8>, // Type ByteString
    pub sig_algorithm: Box<dyn SignaturesAlg>,
}
