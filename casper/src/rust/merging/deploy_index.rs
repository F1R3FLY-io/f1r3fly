// See casper/src/main/scala/coop/rchain/casper/merging/DeployIndex.scala

use models::rust::casper::protocol::casper_message::Event;
use rspace_plus_plus::rspace::merger::event_log_index::EventLogIndex;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd)]
pub struct DeployIndex {
    pub deploy_id: prost::bytes::Bytes,
    pub cost: u64,
    pub event_log_index: EventLogIndex,
}

impl DeployIndex {
    pub fn new(
        sig: prost::bytes::Bytes,
        cost: u64,
        events: Vec<Event>,
        create_event_log_index: impl Fn(Vec<Event>) -> EventLogIndex,
    ) -> Self {
        let event_log_index = create_event_log_index(events);

        Self {
            deploy_id: sig,
            cost,
            event_log_index,
        }
    }
}
