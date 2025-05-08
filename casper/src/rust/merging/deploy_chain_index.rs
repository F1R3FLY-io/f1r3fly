// See casper/src/main/scala/coop/rchain/casper/merging/DeployChainIndex.scala

use prost::bytes::Bytes;
use std::collections::HashSet;

use rspace_plus_plus::rspace::{
    errors::HistoryError,
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
    deploys_with_cost: HashSet<DeployIdWithCost>,
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
    ) -> Result<Self, HistoryError>
    where
        C: std::clone::Clone
            + serde::Serialize
            + for<'de> serde::Deserialize<'de>
            + Send
            + Sync
            + 'static,
        P: std::clone::Clone + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
        A: std::clone::Clone + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
        K: std::clone::Clone + for<'de> serde::Deserialize<'de> + Send + Sync + 'static,
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

        let pre_history_reader =
            history_repository.get_history_reader_struct(&pre_state_hash)?;
        let post_history_reader =
            history_repository.get_history_reader_struct(&post_state_hash)?;

        let state_changes = StateChange::new(
            pre_history_reader,
            post_history_reader,
            event_log_index.clone(),
        )?;

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for deploy in &deploys_with_cost {
            std::hash::Hash::hash(&deploy.deploy_id, &mut hasher);
        }
        let hash_code = std::hash::Hasher::finish(&hasher) as i32;

        Ok(Self {
            deploys_with_cost,
            pre_state_hash,
            post_state_hash,
            event_log_index,
            state_changes,
            hash_code,
        })
    }
}

impl PartialEq for DeployChainIndex {
    fn eq(&self, other: &Self) -> bool {
        self.deploys_with_cost == other.deploys_with_cost
    }
}

impl Eq for DeployChainIndex {}
