use crypto::rust::signatures::signed::Signed;

use crate::{rhoapi::PCost, ByteString};

// See models/src/main/scala/coop/rchain/casper/protocol/CasperMessage.scala
pub struct BlockMessage {
    pub block_hash: Vec<u8>,
    pub header: Header,
    pub body: Body,
    pub justifications: Vec<Justification>,
    pub sender: Vec<u8>,
    pub seq_num: i32,
    pub sig: Vec<u8>,
    pub sig_algorithm: String,
    pub shard_id: String,
    pub extra_bytes: Vec<u8>,
}

pub struct Header {
    pub parents_hash_list: Vec<ByteString>,
    pub timestamp: i64,
    pub version: i64,
    pub extra_bytes: ByteString,
}

pub struct Body {
    pub state: F1r3flyState,
    deploys: Vec<ProcessedDeploy>,
}

pub struct Justification {
    validator: ByteString,
    latest_block_hash: ByteString,
}

pub struct F1r3flyState {
    pub pre_state_hash: ByteString,
    pub post_state_hash: ByteString,
    pub bonds: Vec<Bond>,
    pub block_number: i64,
}

pub struct ProcessedDeploy {
    deploy: Signed<DeployData>,
    cost: PCost,
    deploy_log: Vec<Event>,
    is_failed: bool,
    system_deploy_error: Option<String>,
}

pub struct DeployData {
    term: String,
    time_stamp: i64,
    plo_price: i64,
    plo_limit: i64,
    valid_after_block_number: i64,
    shard_id: String,
}

pub struct Peek {
    channel_index: i32,
}

pub enum Event {
    Produce(ProduceEvent),
    Consume(ConsumeEvent),
    Comm(CommEvent),
}

pub struct ProduceEvent {
    pub channels_hash: ByteString,
    pub hash: ByteString,
    pub persistent: bool,
    pub times_repeated: i32,
}

pub struct ConsumeEvent {
    pub channels_hashes: Vec<ByteString>,
    pub hash: ByteString,
    pub persistent: bool,
}

pub struct CommEvent {
    pub consume: ConsumeEvent,
    pub produces: Vec<ProduceEvent>,
    pub peeks: Vec<Peek>,
}

pub struct Bond {
    validator: ByteString,
    stake: i64,
}
