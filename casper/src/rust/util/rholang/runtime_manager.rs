// See casper/src/main/scala/coop/rchain/casper/util/rholang/RuntimeManager.scala

use models::rhoapi::Par;
use models::rust::block::state_hash::StateHash;
use rholang::rust::interpreter::merging::rholang_merging_logic::DeployMergeableData;
use rholang::rust::interpreter::rho_runtime::{RhoHistoryRepository, RhoISpace, RhoReplayISpace};
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use shared::rust::store::key_value_store::KvStoreError;
use shared::rust::store::key_value_typed_store::KeyValueTypedStore;
use shared::rust::store::key_value_types_store_impl::KeyValueTypedStoreImpl;
use shared::rust::ByteVector;

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

/**
 * Creates connection to [[MergeableStore]] database.
 *
 * Mergeable (number) channels store is used in [[RuntimeManager]] implementation.
 * This function provides default instantiation.
 */
pub async fn mergeable_store(
    kvm: &mut impl KeyValueStoreManager,
) -> Result<impl KeyValueTypedStore<ByteVector, Vec<DeployMergeableData>>, KvStoreError> {
    let store = kvm.store("mergeable-channel-cache".to_string()).await?;

    Ok(KeyValueTypedStoreImpl::new(store))
}
