// See casper/src/main/scala/coop/rchain/casper/util/rholang/RuntimeManager.scala

use models::rust::block::state_hash::StateHash;
use models::{rhoapi::Par, ByteVector};
use rholang::rust::interpreter::merging::rholang_merging_logic::DeployMergeableData;
use rholang::rust::interpreter::rho_runtime::{RhoHistoryRepository, RhoISpace, RhoReplayISpace};
use shared::rust::key_value_typed_store::KeyValueTypedStore;

pub struct RuntimeManager<M>
where
    M: KeyValueTypedStore<ByteVector, Vec<DeployMergeableData>>,
{
    pub space: RhoISpace,
    pub replay_space: RhoReplayISpace,
    pub history_repo: RhoHistoryRepository,
    pub mergeable_store: M,
    pub mergeable_tag_name: Par,
}

impl<M> RuntimeManager<M> where M: KeyValueTypedStore<ByteVector, Vec<DeployMergeableData>> {}

pub fn empty_state_hash_fixed() -> StateHash {
    hex::decode("cb75e7f94e8eac21f95c524a07590f2583fbdaba6fb59291cf52fa16a14c784d")
        .unwrap()
        .into()
}
