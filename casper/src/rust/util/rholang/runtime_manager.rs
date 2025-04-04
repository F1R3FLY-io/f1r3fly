// See casper/src/main/scala/coop/rchain/casper/util/rholang/RuntimeManager.scala
// See casper/src/main/scala/coop/rchain/casper/util/rholang/RuntimeManagerSyntax.scala

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crypto::rust::signatures::signed::Signed;
use hex::ToHex;
use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use models::rust::block::state_hash::{StateHash, StateHashSerde};
use models::rust::block_hash::BlockHash;
use models::rust::casper::protocol::casper_message::{
    Bond, DeployData, ProcessedDeploy, ProcessedSystemDeploy,
};
use models::rust::validator::Validator;
use rholang::rust::interpreter::matcher::r#match::Matcher;
use rholang::rust::interpreter::merging::rholang_merging_logic::{
    DeployMergeableData, NumberChannel, RholangMergingLogic,
};
use rholang::rust::interpreter::rho_runtime::{
    self, RhoHistoryRepository, RhoRuntime, RhoRuntimeImpl,
};
use rholang::rust::interpreter::system_processes::BlockData;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::merger::merging_logic::{NumberChannelsDiff, NumberChannelsEndVal};
use rspace_plus_plus::rspace::replay_rspace::ReplayRSpace;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceStore};
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use shared::rust::store::key_value_store::KvStoreError;
use shared::rust::store::key_value_typed_store_impl::KeyValueTypedStoreImpl;
use shared::rust::ByteVector;

use crate::rust::errors::CasperError;
use crate::rust::rholang::replay_runtime::ReplayRuntimeOps;
use crate::rust::rholang::runtime::RuntimeOps;

use super::system_deploy::SystemDeployTrait;

use retry::{delay::NoDelay, retry, OperationResult};

type MergeableStore = KeyValueTypedStoreImpl<ByteVector, Vec<DeployMergeableData>>;

#[derive(serde::Serialize, serde::Deserialize)]
struct MergeableKey {
    state_hash: StateHashSerde,
    #[serde(with = "shared::rust::serde_bytes")]
    creator: prost::bytes::Bytes,
    seq_num: i32,
}

pub struct RuntimeManager {
    pub space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    pub replay_space: ReplayRSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    pub history_repo: RhoHistoryRepository,
    pub mergeable_store: MergeableStore,
    pub mergeable_tag_name: Par,
}

impl RuntimeManager {
    pub async fn spawn_runtime(&self) -> Arc<Mutex<RhoRuntimeImpl>> {
        let new_space = self.space.spawn().expect("Failed to spawn RSpace");
        let runtime = rho_runtime::create_rho_runtime(
            new_space,
            self.mergeable_tag_name.clone(),
            false,
            &mut Vec::new(),
        )
        .await;

        runtime
    }

    pub async fn spawn_replay_runtime(&self) -> Arc<Mutex<RhoRuntimeImpl>> {
        let new_replay_space = self
            .replay_space
            .spawn()
            .expect("Failed to spawn ReplayRSpace");

        let runtime = rho_runtime::create_replay_rho_runtime(
            new_replay_space,
            self.mergeable_tag_name.clone(),
            false,
            &mut Vec::new(),
        )
        .await;

        runtime
    }

    pub async fn compute_state(
        &mut self,
        start_hash: StateHash,
        terms: Vec<Signed<DeployData>>,
        system_deploys: Vec<impl SystemDeployTrait>,
        block_data: BlockData,
        invalid_blocks: Option<HashMap<BlockHash, Validator>>,
    ) -> Result<(StateHash, Vec<ProcessedDeploy>, Vec<ProcessedSystemDeploy>), CasperError> {
        let invalid_blocks = invalid_blocks.unwrap_or_default();
        let runtime = self.spawn_runtime().await;

        // Block data used for mergeable key
        let sender = block_data.sender.clone();
        let seq_num = block_data.seq_num;

        let computed = RuntimeOps::compute_state(
            runtime,
            start_hash.clone(),
            terms,
            system_deploys,
            block_data,
            invalid_blocks,
        )?;

        let (state_hash, usr_deploy_res, sys_deploy_res) = computed;
        let (usr_processed, usr_mergeable): (Vec<ProcessedDeploy>, Vec<NumberChannelsEndVal>) =
            usr_deploy_res.into_iter().unzip();
        let (sys_processed, sys_mergeable): (
            Vec<ProcessedSystemDeploy>,
            Vec<NumberChannelsEndVal>,
        ) = sys_deploy_res.into_iter().unzip();

        // Concat user and system deploys mergeable channel maps
        let mergeable_chs = usr_mergeable
            .into_iter()
            .chain(sys_mergeable.into_iter())
            .collect();

        // Convert from final to diff values and persist mergeable (number) channels for post-state hash
        let pre_state_hash = Blake2b256Hash::from_bytes_prost(&start_hash);
        let post_state_hash = Blake2b256Hash::from_bytes_prost(&state_hash);

        // Save mergeable channels to store
        self.save_mergeable_channels(
            post_state_hash,
            sender.bytes,
            seq_num,
            mergeable_chs,
            pre_state_hash,
        )?;

        Ok((state_hash, usr_processed, sys_processed))
    }

    pub async fn compute_genesis(
        &mut self,
        terms: Vec<Signed<DeployData>>,
        block_time: i64,
        block_number: i64,
    ) -> Result<(StateHash, StateHash, Vec<ProcessedDeploy>), CasperError> {
        let runtime = self.spawn_runtime().await;
        let computed =
            RuntimeOps::compute_genesis(runtime, terms, block_time, block_number).await?;
        let (pre_state, state_hash, processed) = computed;
        let (processed_deploys, mergeable_chs) = processed.into_iter().unzip();

        // Convert from final to diff values and persist mergeable (number) channels for post-state hash
        let pre_state_hash = Blake2b256Hash::from_bytes_prost(&pre_state);
        let post_state_hash = Blake2b256Hash::from_bytes_prost(&state_hash);

        // Save mergeable channels to store
        self.save_mergeable_channels(
            post_state_hash,
            prost::bytes::Bytes::new(),
            0,
            mergeable_chs,
            pre_state_hash,
        )?;

        Ok((pre_state, state_hash, processed_deploys))
    }

    pub async fn replay_compute_state(
        &mut self,
        start_hash: &StateHash,
        terms: Vec<ProcessedDeploy>,
        system_deploys: Vec<ProcessedSystemDeploy>,
        block_data: &BlockData,
        invalid_blocks: Option<HashMap<BlockHash, Validator>>,
        is_genesis: bool, // FIXME have a better way of knowing this. Pass the replayDeploy function maybe? - OLD
    ) -> Result<StateHash, CasperError> {
        let invalid_blocks = invalid_blocks.unwrap_or_default();
        let replay_runtime = self.spawn_replay_runtime().await;

        let sender = block_data.sender.clone();
        let seq_num = block_data.seq_num;

        let replay_op = ReplayRuntimeOps::replay_compute_state(
            replay_runtime,
            start_hash,
            terms,
            system_deploys,
            block_data,
            Some(invalid_blocks),
            is_genesis,
        )?;

        let (state_hash, mergeable_chs) = replay_op;
        // Convert from final to diff values and persist mergeable (number) channels for post-state hash
        let pre_state_hash = Blake2b256Hash::from_bytes_prost(&start_hash);

        self.save_mergeable_channels(
            state_hash.clone(),
            sender.bytes,
            seq_num,
            mergeable_chs,
            pre_state_hash,
        )
        .unwrap_or_else(|e| panic!("Failed to save mergeable channels: {:?}", e));

        Ok(state_hash.to_bytes_prost())
    }

    pub async fn capture_results(
        &self,
        start: &StateHash,
        deploy: &Signed<DeployData>,
    ) -> Result<Vec<Par>, CasperError> {
        let runtime = self.spawn_runtime().await;
        let computed = RuntimeOps::capture_results(runtime, start, deploy)?;
        Ok(computed)
    }

    pub async fn get_active_validators(
        &self,
        start_hash: &StateHash,
    ) -> Result<Vec<Validator>, CasperError> {
        let runtime = self.spawn_runtime().await;
        let computed = RuntimeOps::get_active_validators(runtime, start_hash)?;
        Ok(computed)
    }

    pub async fn compute_bonds(&self, hash: StateHash) -> Result<Vec<Bond>, CasperError> {
        let runtime = self.spawn_runtime().await;
        let mut retries = 0;
        const MAX_RETRIES: usize = 5;

        // TODO: this retry is a temp solution for debugging why this throws `IllegalArgumentException` - OLD
        let result = retry(NoDelay.take(MAX_RETRIES), || {
            retries += 1;
            match RuntimeOps::compute_bonds(runtime.clone(), &hash) {
                Ok(bonds) => OperationResult::Ok(bonds),
                Err(e) => {
                    // Match Scala's error message format
                    eprintln!(
                        "Unexpected exception {} during computeBonds. {} {}.",
                        e,
                        if retries >= MAX_RETRIES {
                            format!("Giving up after {} retries", retries)
                        } else {
                            format!("Retrying {} time", retries)
                        },
                        if retries >= MAX_RETRIES { "." } else { "" }
                    );

                    if retries >= MAX_RETRIES {
                        OperationResult::Err(e)
                    } else {
                        OperationResult::Retry(e)
                    }
                }
            }
        });

        result.map_err(|e| {
            CasperError::RuntimeError(format!(
                "Unexpected exception {} during computeBonds. Giving up after {} retries.",
                e, MAX_RETRIES
            ))
        })
    }

    // Executes deploy as user deploy with immediate rollback
    pub async fn play_exploratory_deploy(
        &self,
        term: String,
        hash: &StateHash,
    ) -> Result<Vec<Par>, CasperError> {
        let runtime = self.spawn_runtime().await;
        let computed = RuntimeOps::play_exploratory_deploy(runtime, term, hash)?;
        Ok(computed)
    }

    pub async fn get_data(&self, hash: StateHash, channel: &Par) -> Result<Vec<Par>, CasperError> {
        let runtime = self.spawn_runtime().await;

        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(Blake2b256Hash::from_bytes_prost(&hash));
        drop(runtime_lock);

        let computed = RuntimeOps::get_data_par(runtime, channel);
        Ok(computed)
    }

    pub async fn get_continuation(
        &self,
        hash: StateHash,
        channels: Vec<Par>,
    ) -> Result<Vec<(Vec<BindPattern>, Par)>, CasperError> {
        let runtime = self.spawn_runtime().await;

        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(Blake2b256Hash::from_bytes_prost(&hash));
        drop(runtime_lock);

        let computed = RuntimeOps::get_continuation_par(runtime, channels);
        Ok(computed)
    }

    pub fn get_history_repo(self) -> RhoHistoryRepository {
        self.history_repo
    }

    pub fn get_mergeable_store(self) -> MergeableStore {
        self.mergeable_store
    }

    /**
     * Load mergeable channels from store
     */
    pub fn load_mergeable_channels(
        &self,
        state_hash_bs: StateHash,
        creator: prost::bytes::Bytes,
        seq_num: i32,
    ) -> Result<Vec<NumberChannelsDiff>, CasperError> {
        let state_hash = Blake2b256Hash::from_bytes_prost(&state_hash_bs);
        let mergeable_key = MergeableKey {
            state_hash: StateHashSerde(state_hash.to_bytes_prost()),
            creator,
            seq_num,
        };

        let get_key =
            bincode::serialize(&mergeable_key).expect("Failed to serialize mergeable key");

        let res = self.mergeable_store.get_one(&get_key)?;

        match res {
            Some(res) => {
                let res_map = res
                    .into_iter()
                    .map(|x| {
                        x.channels
                            .into_iter()
                            .map(|y| (y.hash, y.diff))
                            .collect::<HashMap<_, _>>()
                    })
                    .collect::<Vec<_>>();
                Ok(res_map)
            }

            None => Err(CasperError::RuntimeError(format!(
                "Mergeable store invalid state hash {}",
                state_hash.bytes().encode_hex::<String>()
            ))),
        }
    }

    /**
     * Converts final mergeable (number) channel values and save to mergeable store.
     *
     * Tuple (postStateHash, creator, seqNum) is used as a key, preStateHash is used to
     * read initial value to get the difference.
     */
    fn save_mergeable_channels(
        &mut self,
        post_state_hash: Blake2b256Hash,
        creator: prost::bytes::Bytes,
        seq_num: i32,
        channels_data: Vec<NumberChannelsEndVal>,
        // Used to calculate value difference from final values
        pre_state_hash: Blake2b256Hash,
    ) -> Result<(), CasperError> {
        // Calculate difference values from final values on number channels
        let diffs = self.convert_number_channels_to_diff(channels_data, pre_state_hash);

        // Convert to storage types
        let deploy_channels = diffs
            .into_iter()
            .map(|data| {
                let channels: Vec<NumberChannel> = data
                    .into_iter()
                    .map(|(hash, diff)| NumberChannel { hash, diff })
                    .collect::<Vec<_>>();

                DeployMergeableData { channels }
            })
            .collect();

        // Key is composed from post-state hash and block creator with seq number
        let mergeable_key = MergeableKey {
            state_hash: StateHashSerde(post_state_hash.to_bytes_prost()),
            creator,
            seq_num,
        };

        let key_encoded =
            bincode::serialize(&mergeable_key).expect("Failed to serialize mergeable key");

        // Save to mergeable channels store
        self.mergeable_store.put_one(key_encoded, deploy_channels)?;

        Ok(())
    }

    /**
     * Converts number channels final values to difference values. Excludes channels without an initial value.
     *
     * @param channelsData Final values
     * @param preStateHash Inital state
     * @return Map with values as difference on number channel
     */
    fn convert_number_channels_to_diff(
        &self,
        channels_data: Vec<NumberChannelsEndVal>,
        // Used to calculate value difference from final values
        pre_state_hash: Blake2b256Hash,
    ) -> Vec<NumberChannelsDiff> {
        let history_repo = self.history_repo.clone();
        let get_data_func = move |ch: &Blake2b256Hash| {
            history_repo
                .get_history_reader(pre_state_hash.clone())
                .and_then(|reader| reader.get_data(ch))
        };

        let get_num_func = RholangMergingLogic::convert_to_read_number(get_data_func);

        // Calculate difference values from final values on number channels
        RholangMergingLogic::calculate_num_channel_diff(channels_data, get_num_func)
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
