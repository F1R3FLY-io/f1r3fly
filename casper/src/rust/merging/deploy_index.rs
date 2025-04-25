// See casper/src/main/scala/coop/rchain/casper/merging/DeployIndex.scala

use rspace_plus_plus::rspace::merger::event_log_index::EventLogIndex;

pub struct DeployIndex {
    pub deploy_id: prost::bytes::Bytes,
    pub cost: u64,
    pub event_log_index: EventLogIndex,
}
