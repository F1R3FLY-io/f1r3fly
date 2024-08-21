use super::signatures_alg::SignaturesAlg;

// See crypto/src/main/scala/coop/rchain/crypto/signatures/Ed25519.scala
pub struct Ed25519;

impl SignaturesAlg for Ed25519 {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: Vec<u8>) -> bool {
        todo!()
    }

    fn sign(&self, data: &[u8], sec: &[u8]) -> Vec<u8> {
        todo!()
    }

    fn to_public(
        &self,
        sec: &crate::rust::private_key::PrivateKey,
    ) -> crate::rust::public_key::PublicKey {
        todo!()
    }

    fn new_key_pair(
        &self,
    ) -> (
        crate::rust::private_key::PrivateKey,
        crate::rust::public_key::PublicKey,
    ) {
        todo!()
    }

    fn name(&self) -> String {
        todo!()
    }

    fn sig_length(&self) -> usize {
        todo!()
    }
}
