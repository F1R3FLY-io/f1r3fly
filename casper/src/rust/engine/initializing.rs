// See casper/src/main/scala/coop/rchain/casper/engine/Initializing.scala

use std::sync::{Arc, Mutex};

use block_storage::rust::{
    dag::block_dag_key_value_storage::BlockDagKeyValueStorage,
    key_value_block_store::KeyValueBlockStore,
};
use comm::rust::rp::rp_conf::RPConf;
use models::rust::casper::protocol::casper_message::ApprovedBlock;

pub struct Initializing {
    rp_conf_ask: RPConf,
    last_approved_block: Arc<Mutex<Option<ApprovedBlock>>>,
    block_store: KeyValueBlockStore,
    block_dag_storage: BlockDagKeyValueStorage,
}
