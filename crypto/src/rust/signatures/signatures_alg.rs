// See crypto/src/main/scala/coop/rchain/crypto/signatures/SignaturesAlg.scala

use crate::rust::{private_key::PrivateKey, public_key::PublicKey};

use super::{secp256k1::Secp256k1, secp256k1_eth::Secp256k1Eth};

// TODO: refactor to use PublicKey and PrivateKey instead of [u8]
pub trait SignaturesAlg {
    fn verify(&self, data: &[u8], signature: &[u8], pub_key: &[u8]) -> bool;

    fn sign(&self, data: &[u8], sec: &[u8]) -> Vec<u8>;

    fn to_public(&self, sec: &PrivateKey) -> PublicKey;

    fn new_key_pair(&self) -> (PrivateKey, PublicKey);

    fn name(&self) -> String;

    fn verify_with_public_key(&self, data: &[u8], signature: &[u8], pub_key: &PublicKey) -> bool {
        self.verify(data, signature, &pub_key.bytes)
    }

    fn sign_with_private_key(&self, data: &[u8], sec: &PrivateKey) -> Vec<u8> {
        self.sign(data, &sec.bytes)
    }

    fn sig_length(&self) -> usize;
}

pub struct SignaturesAlgFactory;

impl SignaturesAlgFactory {
    pub fn apply(name: &str) -> Option<Box<dyn SignaturesAlg>> {
        match name {
            // ed25519 signature algorithm is disabled
            // TODO: quick way to prevent use of ed25519 to sign deploys - OLD
            // https://rchain.atlassian.net/browse/RCHAIN-3560
            // case Ed25519.name => Some(Ed25519)
            "secp256k1" => Some(Box::new(Secp256k1)),
            "secp256k1-eth" => Some(Box::new(Secp256k1Eth)),
            _ => None,
        }
    }
}
