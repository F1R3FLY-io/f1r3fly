use crate::rust::{private_key::PrivateKey, public_key::PublicKey};

// See crypto/src/main/scala/coop/rchain/crypto/signatures/SignaturesAlg.scala
pub struct SignaturesAlg;

impl SignaturesAlg {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: &[u8]) -> bool {
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

    fn verify_with_public_key(&self, data: &[u8], signature: &[u8], pub_key: &PublicKey) -> bool {
        self.verify(data, signature, &pub_key.bytes)
    }

    fn sign_with_private_key(&self, data: &[u8], sec: &PrivateKey) -> Vec<u8> {
        self.sign(data, &sec.bytes)
    }

    fn sig_length(&self) -> usize {
        todo!()
    }
}
