// See casper/src/main/scala/coop/rchain/casper/util/rholang/RuntimeManager.scala

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crypto::rust::signatures::signed::Signed;
use models::casper::system_deploy_data_proto::SystemDeploy;
use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use models::rust::block::state_hash::StateHash;
use models::rust::block_hash::BlockHash;
use models::rust::casper::protocol::casper_message::{
    Bond, DeployData, ProcessedDeploy, ProcessedSystemDeploy,
};
use models::rust::validator::Validator;
use rholang::rust::interpreter::matcher::r#match::Matcher;
use rholang::rust::interpreter::merging::rholang_merging_logic::DeployMergeableData;
use rholang::rust::interpreter::rho_runtime::{self, RhoHistoryRepository, RhoRuntimeImpl};
use rholang::rust::interpreter::system_processes::BlockData;
use rspace_plus_plus::rspace::replay_rspace::ReplayRSpace;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceStore};
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use shared::rust::store::key_value_store::KvStoreError;
use shared::rust::store::key_value_typed_store_impl::KeyValueTypedStoreImpl;
use shared::rust::ByteVector;

use crate::rust::errors::CasperError;

use super::replay_failure::ReplayFailure;

type MergeableStore = KeyValueTypedStoreImpl<ByteVector, Vec<DeployMergeableData>>;

pub struct RuntimeManager {
    pub space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    pub replay_space: ReplayRSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    pub history_repo: RhoHistoryRepository,
    pub mergeable_store: MergeableStore,
    pub mergeable_tag_name: Par,
}

impl RuntimeManager {
    pub async fn spawn_runtime(self) -> Arc<Mutex<RhoRuntimeImpl>> {
        let new_space = self.space.spawn().expect("Failed to spawn RSpace");
        let runtime = rho_runtime::create_rho_runtime(
            new_space,
            self.mergeable_tag_name,
            false,
            &mut Vec::new(),
        )
        .await;

        runtime
    }

    pub async fn spawn_replay_runtime(self) -> Arc<Mutex<RhoRuntimeImpl>> {
        let new_replay_space = self
            .replay_space
            .spawn()
            .expect("Failed to spawn ReplayRSpace");

        let runtime = rho_runtime::create_replay_rho_runtime(
            new_replay_space,
            self.mergeable_tag_name,
            false,
            &mut Vec::new(),
        )
        .await;

        runtime
    }

    pub fn compute_state(
        &self,
        start_hash: StateHash,
        terms: Vec<Signed<DeployData>>,
        system_deploys: Vec<SystemDeploy>,
        block_data: BlockData,
        invalid_blocks: Option<HashMap<BlockHash, Validator>>,
    ) -> Result<(StateHash, Vec<ProcessedDeploy>, Vec<ProcessedSystemDeploy>), CasperError> {
        let invalid_blocks = invalid_blocks.unwrap_or_default();
        todo!()
    }

    pub fn compute_genesis(
        &self,
        terms: Vec<Signed<DeployData>>,
        block_time: i64,
        block_number: i64,
    ) -> Result<(StateHash, StateHash, Vec<ProcessedDeploy>), CasperError> {
        todo!()
    }

    pub fn replay_compute_state(
        &self,
        start_hash: StateHash,
        terms: Vec<ProcessedDeploy>,
        system_deploys: Vec<ProcessedSystemDeploy>,
        block_data: BlockData,
    ) -> Result<StateHash, ReplayFailure> {
        todo!()
    }

    pub fn capture_results(
        &self,
        start: StateHash,
        deploy: Signed<DeployData>,
    ) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    pub fn get_active_validators(start_hash: StateHash) -> Result<Vec<Validator>, CasperError> {
        todo!()
    }

    pub fn compute_bonds(hash: StateHash) -> Result<Vec<Bond>, CasperError> {
        todo!()
    }

    pub fn play_exploratory_deploy(term: String, hash: StateHash) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    pub fn get_data(hash: StateHash) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    pub fn get_continuation(hash: StateHash) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    pub fn get_history_repo(self) -> RhoHistoryRepository {
        self.history_repo
    }

    pub fn get_mergeable_store(self) -> MergeableStore {
        self.mergeable_store
    }

    /**
     * This is a hard-coded value for `emptyStateHash` which is calculated by
     * [[coop.rchain.casper.rholang.RuntimeOps.emptyStateHash]].
     * Because of the value is actually the same all
     * the time. For some situations, we can just use the value directly for better performance.
     */
    pub fn empty_state_hash_fixed() -> StateHash {
        hex::decode("cb75e7f94e8eac21f95c524a07590f2583fbdaba6fb59291cf52fa16a14c784d")
            .unwrap()
            .into()
    }

    pub fn create_with_space(
        rspace: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
        replay_rspace: ReplayRSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
        history_repo: RhoHistoryRepository,
        mergeable_store: MergeableStore,
        mergeable_tag_name: Par,
    ) -> RuntimeManager {
        RuntimeManager {
            space: rspace,
            replay_space: replay_rspace,
            history_repo,
            mergeable_store,
            mergeable_tag_name,
        }
    }

    pub fn create_with_store(
        store: RSpaceStore,
        mergeable_store: MergeableStore,
        mergeable_tag_name: Par,
    ) -> RuntimeManager {
        let (rt_manager, _) = Self::create_with_history(store, mergeable_store, mergeable_tag_name);
        rt_manager
    }

    pub fn create_with_history(
        store: RSpaceStore,
        mergeable_store: MergeableStore,
        mergeable_tag_name: Par,
    ) -> (RuntimeManager, RhoHistoryRepository) {
        let (rspace, replay_rspace) =
            RSpace::create_with_replay(store, Arc::new(Box::new(Matcher)))
                .expect("Failed to create RSpaceWithReplay");

        let history_repo = rspace.history_repository.clone();

        let runtime_manager = RuntimeManager::create_with_space(
            rspace,
            replay_rspace,
            history_repo.clone(),
            mergeable_store,
            mergeable_tag_name,
        );

        (runtime_manager, history_repo)
    }

    /**
     * Creates connection to [[MergeableStore]] database.
     *
     * Mergeable (number) channels store is used in [[RuntimeManager]] implementation.
     * This function provides default instantiation.
     */
    pub async fn mergeable_store(
        kvm: &mut impl KeyValueStoreManager,
    ) -> Result<MergeableStore, KvStoreError> {
        let store = kvm.store("mergeable-channel-cache".to_string()).await?;

        Ok(KeyValueTypedStoreImpl::new(store))
    }
}
