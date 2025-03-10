// See casper/src/main/scala/coop/rchain/casper/genesis/contracts/Validator.scala

use crypto::rust::public_key::PublicKey;

#[derive(PartialEq, Eq, Hash)]
pub struct Validator {
    pub pk: PublicKey,
    pub stake: i64,
}
