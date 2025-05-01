// See casper/src/main/scala/coop/rchain/casper/merging/DeployChainIndex.scala

use prost::bytes::Bytes;
use std::collections::HashSet;

use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    history::history_repository::HistoryRepository,
    merger::{event_log_index::EventLogIndex, state_change::StateChange},
};

use super::deploy_index::DeployIndex;

#[derive(Debug, PartialEq, Eq, Hash)]
struct DeployIdWithCost {
    deploy_id: Bytes,
    cost: u64,
}

/** index of deploys depending on each other inside a single block (state transition) */
pub struct DeployChainIndex {
    deploys_with_cost: Vec<DeployIdWithCost>,
    pre_state_hash: Blake2b256Hash,
    post_state_hash: Blake2b256Hash,
    event_log_index: EventLogIndex,
    state_changes: StateChange,
    // caching hash code helps a lot to increase performance of computing rejection options
    // TODO mysterious speedup of merging benchmark when setting this to some fixed value - OLD
    hash_code: i32,
}

impl DeployChainIndex {
    pub fn new<C, P, A, K>(
        deploys: HashSet<DeployIndex>,
        pre_state_hash: Blake2b256Hash,
        post_state_hash: Blake2b256Hash,
        history_repository: impl HistoryRepository<C, P, A, K>,
    ) -> Self
    where
        C: std::clone::Clone,
        P: std::clone::Clone,
        A: std::clone::Clone,
        K: std::clone::Clone,
    {
        let deploys_with_cost: HashSet<DeployIdWithCost> = deploys
            .iter()
            .map(|deploy| DeployIdWithCost {
                deploy_id: deploy.deploy_id.clone(),
                cost: deploy.cost,
            })
            .collect();

        let event_log_index = deploys
            .into_iter()
            .fold(EventLogIndex::empty(), |acc, deploy| {
                EventLogIndex::combine(acc, deploy.event_log_index)
            });

        todo!()
    }
}

impl PartialEq for DeployChainIndex {
    fn eq(&self, other: &Self) -> bool {
        self.deploys_with_cost == other.deploys_with_cost
    }
}

impl Eq for DeployChainIndex {}
