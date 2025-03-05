// See casper/src/test/scala/coop/rchain/casper/util/GenesisBuilder.scala

use std::path::PathBuf;

use crypto::rust::{private_key::PrivateKey, public_key::PublicKey};
use models::rust::casper::protocol::casper_message::BlockMessage;

pub struct GenesisContext {
    pub genesis_block: BlockMessage,
    pub validator_key_pairs: Vec<(PrivateKey, PublicKey)>,
    pub genesis_vaults: Vec<(PrivateKey, PublicKey)>,
    pub storage_directory: PathBuf,
}
