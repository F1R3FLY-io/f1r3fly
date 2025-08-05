use async_trait::async_trait;
use casper::rust::{
    casper::{Casper, MultiParentCasper, DeployError, CasperSnapshot},
    errors::CasperError,
    validator_identity::ValidatorIdentity,
    block_status::{BlockError, ValidBlock, InvalidBlock},
};
use models::rust::{
    block_hash::{BlockHash, BlockHashSerde},
    block_metadata::BlockMetadata,
    casper::protocol::casper_message::{BlockMessage, ApprovedBlock, DeployData},
};
use crypto::rust::signatures::signed::Signed;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet, BTreeMap};
use block_storage::rust::dag::block_dag_key_value_storage::{KeyValueDagRepresentation as RealKeyValueDagRepresentation, DeployId};
use block_storage::rust::key_value_block_store::KeyValueBlockStore;
use rspace_plus_plus::rspace::{history::Either, state::rspace_state_manager::RSpaceStateManager};
use models::rust::validator::Validator;
use dashmap::DashMap;
use shared::rust::store::{key_value_store::{KeyValueStore, KvStoreError}, key_value_typed_store_impl::KeyValueTypedStoreImpl};
use block_storage::rust::dag::block_metadata_store::BlockMetadataStore;

// Mock KeyValueStore for testing
#[derive(Clone, Default)]
struct MockKeyValueStore {
    data: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MockKeyValueStore {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl KeyValueStore for MockKeyValueStore {
    fn get(&self, keys: &Vec<Vec<u8>>) -> Result<Vec<Option<Vec<u8>>>, KvStoreError> {
        let data = self.data.lock().unwrap();
        let results = keys.iter().map(|key| data.get(key).cloned()).collect();
        Ok(results)
    }
    fn put(&mut self, kv_pairs: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), KvStoreError> {
        let mut data = self.data.lock().unwrap();
        for (key, value) in kv_pairs {
            data.insert(key, value);
        }
        Ok(())
    }
    fn delete(&mut self, keys: Vec<Vec<u8>>) -> Result<usize, KvStoreError> {
        let mut data = self.data.lock().unwrap();
        let mut count = 0;
        for key in keys {
            if data.remove(&key).is_some() {
                count += 1;
            }
        }
        Ok(count)
    }
    fn contains(&self, keys: &Vec<Vec<u8>>) -> Result<Vec<bool>, KvStoreError> {
        let data = self.data.lock().unwrap();
        let results = keys.iter().map(|key| data.contains_key(key)).collect();
        Ok(results)
    }
    fn to_map(&self) -> Result<BTreeMap<Vec<u8>, Vec<u8>>, KvStoreError> {
        let data = self.data.lock().unwrap();
        Ok(data.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }
    fn iterate(&self, f: fn(Vec<u8>, Vec<u8>)) -> Result<(), KvStoreError> {
        let data = self.data.lock().unwrap();
        for (k, v) in data.iter() {
            f(k.clone(), v.clone());
        }
        Ok(())
    }
    fn clone_box(&self) -> Box<dyn KeyValueStore> {
        Box::new(self.clone())
    }
    fn print_store(&self) {}
    fn size_bytes(&self) -> usize {
        let data = self.data.lock().unwrap();
        data.len() * 100 // rough estimate
    }
}

// Newtype wrapper to implement Default locally
pub struct DagRepresentation(pub RealKeyValueDagRepresentation);

impl Default for DagRepresentation {
    fn default() -> Self {
        let store = Box::new(MockKeyValueStore::default());
        let typed_store = KeyValueTypedStoreImpl::<BlockHashSerde, BlockMetadata>::new(store);
        let block_metadata_store = BlockMetadataStore::new(typed_store);

        let deploy_store = Box::new(MockKeyValueStore::default());
        let deploy_typed_store = KeyValueTypedStoreImpl::<DeployId, BlockHashSerde>::new(deploy_store);

        Self(RealKeyValueDagRepresentation {
            dag_set: Default::default(),
            latest_messages_map: Default::default(),
            child_map: Default::default(),
            height_map: Default::default(),
            invalid_blocks_set: Default::default(),
            last_finalized_block_hash: Default::default(),
            finalized_blocks_set: Default::default(),
            block_metadata_index: Arc::new(std::sync::RwLock::new(block_metadata_store)),
            deploy_index: Arc::new(std::sync::RwLock::new(deploy_typed_store)),
        })
    }
}

// A mock implementation of MultiParentCasper for testing purposes.
pub struct MockCasper {
    block_store_map: Arc<Mutex<HashMap<BlockHash, BlockMessage>>>,
    dag: Arc<Mutex<HashSet<BlockHash>>>,
    buffer: Arc<Mutex<HashSet<BlockHash>>>,
    validator: Option<ValidatorIdentity>,
    latest_messages: Arc<DashMap<Validator, BlockHash>>,
    approved_block: ApprovedBlock,
    key_value_block_store: KeyValueBlockStore,
}

impl MockCasper {
    pub fn new(approved_block: ApprovedBlock) -> Self {
        let store = Box::new(MockKeyValueStore::new());
        let store_approved_block = Box::new(MockKeyValueStore::new());
        let mut key_value_block_store = KeyValueBlockStore::new(store, store_approved_block);
        
        // Store the genesis/approved block so it can be found by the handlers
        let genesis_block = approved_block.candidate.block.clone();
        key_value_block_store.put_block_message(&genesis_block).unwrap_or_else(|e| {
            log::warn!("Failed to store genesis block in mock: {:?}", e);
        });
        
        Self {
            block_store_map: Arc::new(Mutex::new(HashMap::new())),
            dag: Arc::new(Mutex::new(HashSet::new())),
            buffer: Arc::new(Mutex::new(HashSet::new())),
            validator: None,
            latest_messages: Arc::new(DashMap::new()),
            approved_block,
            key_value_block_store,
        }
    }

    pub fn add_block_to_store(&self, block: BlockMessage) {
        self.block_store_map.lock().unwrap().insert(block.block_hash.clone(), block.clone());
        // Also store in the actual KeyValueBlockStore, but since we can't mutate it directly,
        // we'll override the get method to check our HashMap first
    }

    pub fn add_to_dag(&self, block_hash: BlockHash) {
        self.dag.lock().unwrap().insert(block_hash);
    }

    pub fn set_latest_messages(&self, tips: HashMap<Validator, BlockHash>) {
        for (validator, hash) in tips {
            self.latest_messages.insert(validator, hash);
        }
    }

    pub fn get_latest_messages(&self) -> HashMap<Validator, BlockHash> {
        self.latest_messages.iter().map(|entry| (entry.key().clone(), entry.value().clone())).collect()
    }

    pub fn get_approved_block(&self) -> &ApprovedBlock {
        &self.approved_block
    }
}

impl Clone for MockCasper {
    fn clone(&self) -> Self {
        let store = Box::new(MockKeyValueStore::new());
        let store_approved_block = Box::new(MockKeyValueStore::new());
        let key_value_block_store = KeyValueBlockStore::new(store, store_approved_block);
        
        Self {
            block_store_map: self.block_store_map.clone(),
            dag: self.dag.clone(),
            buffer: self.buffer.clone(),
            validator: self.validator.clone(),
            latest_messages: self.latest_messages.clone(),
            approved_block: self.get_approved_block().clone(),
            key_value_block_store,
        }
    }
}

#[async_trait(?Send)]
impl Casper for MockCasper {
    async fn get_snapshot(&mut self) -> Result<CasperSnapshot, CasperError> {
        unimplemented!();
    }

    fn contains(&self, hash: &BlockHash) -> bool {
        self.dag_contains(hash) || self.buffer_contains(hash)
    }

    fn dag_contains(&self, hash: &BlockHash) -> bool {
        self.dag.lock().unwrap().contains(hash)
    }

    fn buffer_contains(&self, hash: &BlockHash) -> bool {
        self.buffer.lock().unwrap().contains(hash)
    }

    fn get_approved_block(&self) -> Result<&BlockMessage, CasperError> {
        Ok(&self.approved_block.candidate.block)
    }

    fn deploy(
        &mut self,
        _deploy: Signed<DeployData>,
    ) -> Result<Either<DeployError, DeployId>, CasperError> {
        unimplemented!();
    }

    async fn estimator(
        &self,
        _dag: &mut RealKeyValueDagRepresentation,
    ) -> Result<Vec<BlockHash>, CasperError> {
        unimplemented!();
    }

    fn get_version(&self) -> i64 {
        1
    }

    async fn validate(
        &mut self,
        _block: &BlockMessage,
        _snapshot: &mut CasperSnapshot,
    ) -> Result<Either<BlockError, ValidBlock>, CasperError> {
        unimplemented!();
    }

    async fn handle_valid_block(
        &mut self,
        _block: &BlockMessage,
    ) -> Result<RealKeyValueDagRepresentation, CasperError> {
        unimplemented!();
    }

    fn handle_invalid_block(
        &mut self,
        _block: &BlockMessage,
        _status: &InvalidBlock,
        _dag: &RealKeyValueDagRepresentation,
    ) -> Result<RealKeyValueDagRepresentation, CasperError> {
        unimplemented!();
    }

    fn get_dependency_free_from_buffer(&self) -> Result<Vec<BlockMessage>, CasperError> {
        unimplemented!();
    }
}

#[async_trait(?Send)]
impl MultiParentCasper for MockCasper {
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
        // For the tests, we can return the genesis block as the last finalized block.
        Ok(self.get_approved_block().candidate.block.clone())
    }

    async fn block_dag(&self) -> Result<RealKeyValueDagRepresentation, CasperError> {
        let mut dag = DagRepresentation::default().0;
        // Add latest messages from our test data
        for entry in self.latest_messages.iter() {
            dag.latest_messages_map.insert(entry.key().clone(), entry.value().clone());
        }
        // Set the last finalized block to the genesis block so it can be found
        dag.last_finalized_block_hash = self.get_approved_block().candidate.block.block_hash.clone();
        Ok(dag)
    }

    fn block_store(&self) -> &KeyValueBlockStore {
        &self.key_value_block_store
    }

    fn rspace_state_manager(&self) -> &RSpaceStateManager {
        unimplemented!();
    }

    fn get_validator(&self) -> Option<ValidatorIdentity> {
        self.validator.clone()
    }

    fn get_history_exporter(
        &self,
    ) -> std::sync::Arc<
        std::sync::Mutex<Box<dyn rspace_plus_plus::rspace::state::rspace_exporter::RSpaceExporter>>,
    > {
        unimplemented!();
    }
}
