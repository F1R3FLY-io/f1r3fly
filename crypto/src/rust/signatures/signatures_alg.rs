use crate::rust::{private_key::PrivateKey, public_key::PublicKey};

// See crypto/src/main/scala/coop/rchain/crypto/signatures/SignaturesAlg.scala
pub trait SignaturesAlg {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: Vec<u8>) -> bool;

    fn sign(&self, data: &[u8], sec: &[u8]) -> Vec<u8>;

    fn to_public(&self, sec: &PrivateKey) -> PublicKey;

    fn new_key_pair(&self) -> (PrivateKey, PublicKey);

    fn name(&self) -> String;

    fn verify_with_public_key(&self, data: &[u8], signature: &[u8], pub_key: &PublicKey) -> bool {
        self.verify(data, signature, pub_key.bytes.clone())
    }

    fn sign_with_private_key(&self, data: &[u8], sec: &PrivateKey) -> Vec<u8> {
        self.sign(data, &sec.bytes)
    }

    fn sig_length(&self) -> usize;
}
