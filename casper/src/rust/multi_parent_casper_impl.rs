// See casper/src/main/scala/coop/rchain/casper/MultiParentCasperImpl.scala

use std::collections::HashMap;

use block_storage::rust::dag::block_dag_key_value_storage::KeyValueDagRepresentation;
use crypto::rust::signatures::signed::Signed;
use models::{
    rhoapi::DeployId,
    rust::{
        block_hash::BlockHash,
        casper::protocol::casper_message::{BlockMessage, DeployData},
        validator::Validator,
    },
};
use rspace_plus_plus::rspace::history::Either;
use shared::rust::shared::f1r3fly_event::F1r3flyEvent;

use crate::rust::{
    block_status::{BlockError, ValidBlock},
    casper::{Casper, CasperShardConf, CasperSnapshot, DeployError, MultiParentCasper},
    errors::CasperError,
    validator_identity::ValidatorIdentity,
};

pub struct MultiParentCasperImpl {
    pub validator_id: Option<ValidatorIdentity>,
    // TODO: this should be read from chain, for now read from startup options - OLD
    pub casper_shard_conf: CasperShardConf,
    pub approved_block: BlockMessage,
}

impl Casper for MultiParentCasperImpl {
    fn get_snapshot(&self) -> Result<CasperSnapshot, CasperError> {
        todo!()
    }

    fn contains(&self, hash: &BlockHash) -> Result<bool, CasperError> {
        todo!()
    }

    fn dag_contains(&self, hash: &BlockHash) -> Result<bool, CasperError> {
        todo!()
    }

    fn buffer_contains(&self, hash: &BlockHash) -> Result<bool, CasperError> {
        todo!()
    }

    fn deploy(
        &self,
        deploy: Signed<DeployData>,
    ) -> Result<Either<DeployError, DeployId>, CasperError> {
        todo!()
    }

    fn estimator(&self, dag: &KeyValueDagRepresentation) -> Result<Vec<BlockHash>, CasperError> {
        todo!()
    }

    fn get_approved_block(&self) -> Result<BlockMessage, CasperError> {
        todo!()
    }

    fn get_validator(&self) -> Result<Option<ValidatorIdentity>, CasperError> {
        todo!()
    }

    fn get_version(&self) -> Result<i64, CasperError> {
        todo!()
    }

    fn validate(
        &self,
        block: &BlockMessage,
        snapshot: &CasperSnapshot,
    ) -> Result<Either<BlockError, ValidBlock>, CasperError> {
        todo!()
    }

    fn handle_valid_block(
        &self,
        block: &BlockMessage,
    ) -> Result<KeyValueDagRepresentation, CasperError> {
        todo!()
    }

    fn handle_invalid_block(
        &self,
        block: &BlockMessage,
        status: &super::block_status::InvalidBlock,
        dag: &KeyValueDagRepresentation,
    ) -> Result<KeyValueDagRepresentation, CasperError> {
        todo!()
    }

    fn get_dependency_free_from_buffer(&self) -> Result<Vec<BlockMessage>, CasperError> {
        todo!()
    }
}

impl MultiParentCasper for MultiParentCasperImpl {
    fn fetch_dependencies(&self) -> Result<(), CasperError> {
        todo!()
    }

    fn normalized_initial_fault(
        &self,
        weights: HashMap<Validator, u64>,
    ) -> Result<f32, CasperError> {
        todo!()
    }

    fn last_finalized_block(&self) -> Result<BlockMessage, CasperError> {
        todo!()
    }
}

pub fn created_event(block: &BlockMessage) -> F1r3flyEvent {
    let block_hash = hex::encode(block.block_hash.clone());
    let parents_hashes = block
        .header
        .parents_hash_list
        .iter()
        .map(|h| hex::encode(h))
        .collect::<Vec<_>>();

    let justifications = block
        .justifications
        .iter()
        .map(|j| {
            (
                hex::encode(j.validator.clone()),
                hex::encode(j.latest_block_hash.clone()),
            )
        })
        .collect::<Vec<_>>();

    let deploy_ids = block
        .body
        .deploys
        .iter()
        .map(|d| hex::encode(d.deploy.sig.clone()))
        .collect::<Vec<_>>();

    let creator = hex::encode(block.sender.clone());
    let seq_num = block.seq_num;

    F1r3flyEvent::block_created(
        block_hash,
        parents_hashes,
        justifications,
        deploy_ids,
        creator,
        seq_num,
    )
}
