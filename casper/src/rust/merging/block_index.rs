// See casper/src/main/scala/coop/rchain/casper/merging/BlockIndex.scala

use models::rust::casper::protocol::casper_message::Event;
use rholang::rust::interpreter::rho_runtime::RhoHistoryRepository;
use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    merger::{event_log_index::EventLogIndex, merging_logic::NumberChannelsDiff},
    trace::event::Produce,
};

use crate::rust::util::event_converter;

pub fn create_event_log_index(
    events: Vec<Event>,
    history_repository: RhoHistoryRepository,
    pre_state_hash: Blake2b256Hash,
    mergeable_chs: NumberChannelsDiff,
) -> EventLogIndex {
    let pre_state_reader = history_repository
        .get_history_reader(pre_state_hash)
        .unwrap();

    let produce_exists_in_pre_state = |p: &Produce| {
        pre_state_reader
            .get_data(&p.channel_hash)
            .map_or(false, |data| data.iter().any(|d| d.source == *p))
    };

    let produce_touches_pre_state_join = |p: &Produce| {
        pre_state_reader
            .get_joins(&p.channel_hash)
            .map_or(false, |joins| joins.iter().any(|j| j.len() > 1))
    };

    EventLogIndex::new(
        events
            .iter()
            .map(event_converter::to_rspace_event)
            .collect(),
        produce_exists_in_pre_state,
        produce_touches_pre_state_join,
        mergeable_chs,
    )
}
