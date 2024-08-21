use crate::rust::{private_key::PrivateKey, public_key::PublicKey};

use super::signatures_alg::SignaturesAlg;

// See crypto/src/main/scala/coop/rchain/crypto/signatures/Secp256k1.scala
pub struct Secp256k1;

impl SignaturesAlg for Secp256k1 {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: Vec<u8>) -> bool {
        todo!()
    }

    fn sign(&self, data: &[u8], sec: &[u8]) -> Vec<u8> {
        todo!()
    }

    fn to_public(&self, sec: &PrivateKey) -> PublicKey {
        todo!()
    }

    fn new_key_pair(&self) -> (PrivateKey, PublicKey) {
        todo!()
    }

    fn name(&self) -> String {
        todo!()
    }

    fn sig_length(&self) -> usize {
        todo!()
    }
}
