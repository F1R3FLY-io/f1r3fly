// See casper/src/test/scala/coop/rchain/casper/helper/NoOpsCasperEffect.scala

use std::collections::HashMap;

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
    runtime_manager: RuntimeManager,
    block_store: KeyValueBlockStore,
    block_dag_storage: KeyValueDagRepresentation,
}

impl NoOpsCasperEffect {
    pub fn new(
        blocks: Option<HashMap<BlockHash, BlockMessage>>,
        estimator_func: Option<Vec<BlockHash>>,
        runtime_manager: RuntimeManager,
        block_store: KeyValueBlockStore,
        block_dag_storage: KeyValueDagRepresentation,
    ) -> Self {
        Self {
            store: blocks.unwrap_or_default(),
            estimator_func: estimator_func.unwrap_or_default(),
            runtime_manager,
            block_store,
            block_dag_storage,
        }
    }
}

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
}

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

    fn validate(
        &self,
        _block: &BlockMessage,
        _snapshot: &CasperSnapshot,
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
}
