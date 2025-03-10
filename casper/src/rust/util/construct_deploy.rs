// See casper/src/main/scala/coop/rchain/casper/util/ConstructDeploy.scala

use lazy_static::lazy_static;

use crypto::rust::{
    private_key::PrivateKey,
    public_key::PublicKey,
    signatures::{secp256k1::Secp256k1, signatures_alg::SignaturesAlg},
};

lazy_static! {
    pub static ref DEFAULT_SEC: PrivateKey = PrivateKey::from_bytes(
        &hex::decode("a68a6e6cca30f81bd24a719f3145d20e8424bd7b396309b0708a16c7d8000b76")
            .expect("ConstructDeploy: Failed to decode default private key")
    );
    pub static ref DEFAULT_PUB: PublicKey = {
        let secp = Secp256k1;
        secp.to_public(&DEFAULT_SEC)
    };
    pub static ref DEFAULT_KEY_PAIR: (&'static PrivateKey, &'static PublicKey) =
        (&DEFAULT_SEC, &DEFAULT_PUB);
    pub static ref DEFAULT_SEC2: PrivateKey = PrivateKey::from_bytes(
        &hex::decode("5a0bde2f5857124b1379c78535b07a278e3b9cefbcacc02e62ab3294c02765a1")
            .expect("ConstructDeploy: Failed to decode default private key")
    );
    pub static ref DEFAULT_PUB2: PublicKey = {
        let secp = Secp256k1;
        secp.to_public(&DEFAULT_SEC2)
    };
}
