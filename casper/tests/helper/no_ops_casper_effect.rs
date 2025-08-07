// See casper/src/test/scala/coop/rchain/casper/helper/NoOpsCasperEffect.scala

use crate::util::test_mocks::MockKeyValueStore;
use async_trait::async_trait;
use casper::rust::validator_identity::ValidatorIdentity;
use rspace_plus_plus::rspace::state::rspace_state_manager::RSpaceStateManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use block_storage::rust::{
    dag::block_dag_key_value_storage::{DeployId, KeyValueDagRepresentation},
    key_value_block_store::KeyValueBlockStore,
};
use casper::rust::{
    block_status::{BlockError, InvalidBlock, ValidBlock},
    casper::{Casper, CasperSnapshot, DeployError, MultiParentCasper},
    errors::CasperError,
    util::rholang::runtime_manager::RuntimeManager,
};
use crypto::rust::signatures::signed::Signed;
use models::rust::{
    block_hash::BlockHash,
    block_implicits::get_random_block_default,
    casper::protocol::casper_message::{BlockMessage, DeployData},
    validator::Validator,
};
use rspace_plus_plus::rspace::history::Either;

pub struct NoOpsCasperEffect {
    store: HashMap<BlockHash, BlockMessage>,
    estimator_func: Vec<BlockHash>,
    runtime_manager: Arc<RuntimeManager>,
    block_store: KeyValueBlockStore,
    // Shared data for block store to ensure clones can access the same blocks
    shared_block_data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    shared_approved_block_data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
    block_dag_storage: KeyValueDagRepresentation,
}

unsafe impl Send for NoOpsCasperEffect {}
unsafe impl Sync for NoOpsCasperEffect {}

// For testing purposes, we'll implement Clone manually by creating stub instances
impl Clone for NoOpsCasperEffect {
    fn clone(&self) -> Self {
        // Create a clone that shares the same underlying storage so that blocks added to one instance
        // are visible to cloned instances (which is necessary for the engine tests to work)

        // Create new KeyValueBlockStore with shared underlying storage
        // Note: We need to share the underlying data between clones for tests to work
        let cloned_block_store = KeyValueBlockStore::new(
            Box::new(MockKeyValueStore::with_shared_data(
                self.shared_block_data.clone(),
            )),
            Box::new(MockKeyValueStore::with_shared_data(
                self.shared_approved_block_data.clone(),
            )),
        );

        Self {
            store: self.store.clone(),
            estimator_func: self.estimator_func.clone(),
            runtime_manager: self.runtime_manager.clone(), // Arc clone is cheap
            block_store: cloned_block_store,
            shared_block_data: self.shared_block_data.clone(),
            shared_approved_block_data: self.shared_approved_block_data.clone(),
            block_dag_storage: self.block_dag_storage.clone(),
        }
    }
}

// Using shared MockKeyValueStore from test_mocks module

impl NoOpsCasperEffect {
    pub fn new(
        blocks: Option<HashMap<BlockHash, BlockMessage>>,
        estimator_func: Option<Vec<BlockHash>>,
        runtime_manager: RuntimeManager,
        _block_store: KeyValueBlockStore, // We'll ignore this and create our own with shared data
        block_dag_storage: KeyValueDagRepresentation,
    ) -> Self {
        // Create shared storage that will be used by all clones
        let shared_block_data = Arc::new(Mutex::new(HashMap::new()));
        let shared_approved_block_data = Arc::new(Mutex::new(HashMap::new()));

        // Create block store with shared underlying storage
        let block_store = KeyValueBlockStore::new(
            Box::new(MockKeyValueStore::with_shared_data(
                shared_block_data.clone(),
            )),
            Box::new(MockKeyValueStore::with_shared_data(
                shared_approved_block_data.clone(),
            )),
        );

        Self {
            store: blocks.unwrap_or_default(),
            estimator_func: estimator_func.unwrap_or_default(),
            runtime_manager: Arc::new(runtime_manager),
            block_store,
            shared_block_data,
            shared_approved_block_data,
            block_dag_storage,
        }
    }

    /// Create a simple test instance with minimal dependencies
    pub fn new_for_test(
        approved_block: models::rust::casper::protocol::casper_message::ApprovedBlock,
    ) -> Self {
        use shared::rust::store::key_value_typed_store_impl::KeyValueTypedStoreImpl;

        // Using shared MockKeyValueStore from test_mocks module

        // Create minimal test dependencies
        let genesis_block = approved_block.candidate.block.clone();
        let mut blocks = HashMap::new();
        blocks.insert(genesis_block.block_hash.clone(), genesis_block.clone());

        // Create test block store
        let store = Box::new(MockKeyValueStore::new());
        let store_approved_block = Box::new(MockKeyValueStore::new());
        let block_store = KeyValueBlockStore::new(store, store_approved_block);

        // Create test DAG storage
        use block_storage::rust::dag::block_metadata_store::BlockMetadataStore;
        use models::rust::block_hash::BlockHashSerde;
        use models::rust::block_metadata::BlockMetadata;

        let metadata_store = Box::new(MockKeyValueStore::new());
        let metadata_typed_store =
            KeyValueTypedStoreImpl::<BlockHashSerde, BlockMetadata>::new(metadata_store);
        let block_metadata_store = BlockMetadataStore::new(metadata_typed_store);

        let deploy_store = Box::new(MockKeyValueStore::new());
        let deploy_typed_store =
            KeyValueTypedStoreImpl::<DeployId, BlockHashSerde>::new(deploy_store);

        let block_dag_storage = KeyValueDagRepresentation {
            dag_set: Default::default(),
            latest_messages_map: Default::default(),
            child_map: Default::default(),
            height_map: Default::default(),
            invalid_blocks_set: Default::default(),
            last_finalized_block_hash: genesis_block.block_hash.clone(),
            finalized_blocks_set: Default::default(),
            block_metadata_index: std::sync::Arc::new(std::sync::RwLock::new(block_metadata_store)),
            deploy_index: std::sync::Arc::new(std::sync::RwLock::new(deploy_typed_store)),
        };

        // Create a minimal runtime manager stub
        // Since this is for testing and most methods return todo!(), we'll just panic here for now
        // The tests can be adjusted to not require complex RuntimeManager functionality
        panic!("NoOpsCasperEffect::new_for_test() - RuntimeManager stub not implemented yet. Use the regular constructor with proper dependencies.");
    }
}

#[async_trait(?Send)]
impl MultiParentCasper for NoOpsCasperEffect {
    async fn fetch_dependencies(&self) -> Result<(), CasperError> {
        Ok(())
    }

    fn normalized_initial_fault(
        &self,
        _weights: HashMap<Validator, u64>,
    ) -> Result<f32, CasperError> {
        Ok(0.0)
    }

    async fn last_finalized_block(&mut self) -> Result<BlockMessage, CasperError> {
        Ok(get_random_block_default())
    }

    async fn block_dag(&self) -> Result<KeyValueDagRepresentation, CasperError> {
        Ok(self.block_dag_storage.clone())
    }

    fn block_store(&self) -> &KeyValueBlockStore {
        &self.block_store
    }

    fn rspace_state_manager(&self) -> &RSpaceStateManager {
        todo!()
    }

    fn get_validator(&self) -> Option<ValidatorIdentity> {
        None
    }

    fn get_history_exporter(
        &self,
    ) -> std::sync::Arc<
        std::sync::Mutex<Box<dyn rspace_plus_plus::rspace::state::rspace_exporter::RSpaceExporter>>,
    > {
        todo!()
    }
}

#[async_trait(?Send)]
impl Casper for NoOpsCasperEffect {
    async fn get_snapshot(&mut self) -> Result<CasperSnapshot, CasperError> {
        todo!()
    }

    fn contains(&self, hash: &BlockHash) -> bool {
        self.store.contains_key(hash)
    }

    fn dag_contains(&self, _hash: &BlockHash) -> bool {
        false
    }

    fn buffer_contains(&self, _hash: &BlockHash) -> bool {
        false
    }

    fn deploy(
        &mut self,
        _deploy: Signed<DeployData>,
    ) -> Result<Either<DeployError, DeployId>, CasperError> {
        Ok(Either::Right(DeployId::default()))
    }

    async fn estimator(
        &self,
        _dag: &mut KeyValueDagRepresentation,
    ) -> Result<Vec<BlockHash>, CasperError> {
        Ok(self.estimator_func.clone())
    }

    fn get_version(&self) -> i64 {
        1
    }

    async fn validate(
        &mut self,
        _block: &BlockMessage,
        _snapshot: &mut CasperSnapshot,
    ) -> Result<Either<BlockError, ValidBlock>, CasperError> {
        todo!()
    }

    async fn handle_valid_block(
        &mut self,
        _block: &BlockMessage,
    ) -> Result<KeyValueDagRepresentation, CasperError> {
        todo!()
    }

    fn handle_invalid_block(
        &mut self,
        _block: &BlockMessage,
        _status: &InvalidBlock,
        _dag: &KeyValueDagRepresentation,
    ) -> Result<KeyValueDagRepresentation, CasperError> {
        todo!()
    }

    fn get_dependency_free_from_buffer(&self) -> Result<Vec<BlockMessage>, CasperError> {
        todo!()
    }

    fn get_approved_block(&self) -> Result<&BlockMessage, CasperError> {
        // For test purposes, return a reference to any block from our store
        // or create a default block if store is empty
        self.store.values().next().ok_or_else(|| {
            CasperError::RuntimeError("No approved block available in test".to_string())
        })
    }
}

// Additional test-friendly methods for compatibility with the old MockCasper API
impl NoOpsCasperEffect {
    /// Add a block to the internal store for testing
    pub fn add_block_to_store(&mut self, block: BlockMessage) {
        // Add to internal store
        self.store.insert(block.block_hash.clone(), block.clone());

        // Also add to the KeyValueBlockStore so engine can find it via block_store().get()
        match self.block_store.put_block_message(&block) {
            Ok(_) => log::debug!(
                "Successfully stored block {} in KeyValueBlockStore",
                hex::encode(&block.block_hash)
            ),
            Err(e) => log::error!("Failed to store block in KeyValueBlockStore: {:?}", e),
        }
    }

    /// Add block hash to DAG (no-op for testing)
    pub fn add_to_dag(&self, _block_hash: BlockHash) {
        // NoOp for testing - just acknowledge the call
    }

    /// Set latest messages (updates the shared DAG storage for testing)
    pub fn set_latest_messages(&self, tips: HashMap<Validator, BlockHash>) {
        // Clear existing latest messages and insert new ones
        self.block_dag_storage.latest_messages_map.clear();
        for (validator, block_hash) in tips {
            self.block_dag_storage
                .latest_messages_map
                .insert(validator, block_hash);
        }
    }

    /// Get approved block reference directly (for test compatibility)
    pub fn get_approved_block_direct(&self) -> &BlockMessage {
        self.store
            .values()
            .next()
            .expect("No approved block available in test")
    }
}
