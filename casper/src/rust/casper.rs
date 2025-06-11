// See casper/src/main/scala/coop/rchain/casper/Casper.scala

use block_storage::rust::dag::block_dag_key_value_storage::KeyValueDagRepresentation;
use crypto::rust::signatures::signed::Signed;
use dashmap::{DashMap, DashSet};
use models::{
    rhoapi::DeployId,
    rust::{
        block_hash::BlockHash,
        casper::protocol::casper_message::{BlockMessage, DeployData, Justification},
        validator::Validator,
    },
};
use rspace_plus_plus::rspace::history::Either;

use crate::rust::{
    block_status::{BlockError, InvalidBlock, ValidBlock},
    errors::CasperError,
    multi_parent_casper_impl::MultiParentCasperImpl,
    util::rholang::runtime_manager::RuntimeManager,
    validator_identity::ValidatorIdentity,
};

use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeployError {
    ParsingError(String),
    MissingUser,
    UnknownSignatureAlgorithm(String),
    SignatureVerificationFailed,
}

impl DeployError {
    pub fn parsing_error(details: String) -> Self {
        DeployError::ParsingError(details)
    }

    pub fn missing_user() -> Self {
        DeployError::MissingUser
    }

    pub fn unknown_signature_algorithm(alg: String) -> Self {
        DeployError::UnknownSignatureAlgorithm(alg)
    }

    pub fn signature_verification_failed() -> Self {
        DeployError::SignatureVerificationFailed
    }
}

impl Display for DeployError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeployError::ParsingError(details) => write!(f, "Parsing error: {}", details),
            DeployError::MissingUser => write!(f, "Missing user"),
            DeployError::UnknownSignatureAlgorithm(alg) => {
                write!(f, "Unknown signature algorithm '{}'", alg)
            }
            DeployError::SignatureVerificationFailed => write!(f, "Signature verification failed"),
        }
    }
}

pub trait Casper {
    fn get_snapshot(&self) -> Result<CasperSnapshot, CasperError>;

    fn contains(&self, hash: &BlockHash) -> Result<bool, CasperError>;

    fn dag_contains(&self, hash: &BlockHash) -> Result<bool, CasperError>;

    fn buffer_contains(&self, hash: &BlockHash) -> Result<bool, CasperError>;

    fn deploy(
        &self,
        deploy: Signed<DeployData>,
    ) -> Result<Either<DeployError, DeployId>, CasperError>;

    fn estimator(&self, dag: &KeyValueDagRepresentation) -> Result<Vec<BlockHash>, CasperError>;

    fn get_approved_block(&self) -> Result<BlockMessage, CasperError>;

    fn get_validator(&self) -> Result<Option<ValidatorIdentity>, CasperError>;

    fn get_version(&self) -> Result<i64, CasperError>;

    fn validate(
        &self,
        block: &BlockMessage,
        snapshot: &CasperSnapshot,
    ) -> Result<Either<BlockError, ValidBlock>, CasperError>;

    fn handle_valid_block(
        &self,
        block: &BlockMessage,
    ) -> Result<KeyValueDagRepresentation, CasperError>;

    fn handle_invalid_block(
        &self,
        block: &BlockMessage,
        status: &InvalidBlock,
        dag: &KeyValueDagRepresentation,
    ) -> Result<KeyValueDagRepresentation, CasperError>;

    fn get_dependency_free_from_buffer(&self) -> Result<Vec<BlockMessage>, CasperError>;
}

pub trait MultiParentCasper: Casper {
    fn block_dag(&self) -> Result<KeyValueDagRepresentation, CasperError>;

    fn fetch_dependencies(&self) -> Result<(), CasperError>;

    // This is the weight of faults that have been accumulated so far.
    // We want the clique oracle to give us a fault tolerance that is greater than
    // this initial fault weight combined with our fault tolerance threshold t.
    fn normalized_initial_fault(
        &self,
        weights: HashMap<Validator, u64>,
    ) -> Result<f32, CasperError>;

    fn last_finalized_block(&self) -> Result<BlockMessage, CasperError>;

    fn get_runtime_manager(&self) -> Result<RuntimeManager, CasperError>;
}

pub fn hash_set_casper(
    validator_id: Option<ValidatorIdentity>,
    casper_shard_conf: CasperShardConf,
    approved_block: BlockMessage,
) -> Result<impl MultiParentCasper, CasperError> {
    Ok(MultiParentCasperImpl {
        validator_id,
        casper_shard_conf,
        approved_block,
    })
}

/**
 * Casper snapshot is a state that is changing in discrete manner with each new block added.
 * This class represents full information about the state. It is required for creating new blocks
 * as well as for validating blocks.
 */
pub struct CasperSnapshot {
    pub dag: KeyValueDagRepresentation,
    pub last_finalized_block: BlockHash,
    pub lca: BlockHash,
    pub tips: Vec<BlockHash>,
    pub parents: Vec<BlockMessage>,
    pub justifications: DashSet<Justification>,
    pub invalid_blocks: DashMap<Validator, BlockHash>,
    pub deploys_in_scope: DashSet<Signed<DeployData>>,
    pub max_block_num: i64,
    pub max_seq_nums: DashMap<Validator, u64>,
    pub on_chain_state: OnChainCasperState,
}

impl CasperSnapshot {
    pub fn new(dag: KeyValueDagRepresentation) -> Self {
        Self {
            dag,
            last_finalized_block: BlockHash::default(),
            lca: BlockHash::default(),
            tips: vec![],
            parents: vec![],
            justifications: DashSet::new(),
            invalid_blocks: DashMap::new(),
            deploys_in_scope: DashSet::new(),
            max_block_num: 0,
            max_seq_nums: DashMap::new(),
            on_chain_state: OnChainCasperState::new(CasperShardConf::new()),
        }
    }
}

pub struct OnChainCasperState {
    pub shard_conf: CasperShardConf,
    pub bonds_map: DashMap<Validator, u64>,
    pub active_validators: Vec<Validator>,
}

impl OnChainCasperState {
    pub fn new(shard_conf: CasperShardConf) -> Self {
        Self {
            shard_conf,
            bonds_map: DashMap::new(),
            active_validators: vec![],
        }
    }
}

pub struct CasperShardConf {
    pub fault_tolerance_threshold: f32,
    pub shard_name: String,
    pub parent_shard_id: String,
    pub finalization_rate: i32,
    pub max_number_of_parents: i32,
    pub max_parent_depth: i32,
    pub synchrony_constraint_threshold: f32,
    pub height_constraint_threshold: i64,
    // Validators will try to put deploy in a block only for next `deployLifespan` blocks.
    // Required to enable protection from re-submitting duplicate deploys
    pub deploy_lifespan: i64,
    pub casper_version: i64,
    pub config_version: i64,
    pub bond_minimum: i64,
    pub bond_maximum: i64,
    pub epoch_length: i32,
    pub quarantine_length: i32,
    pub min_phlo_price: i64,
}

impl CasperShardConf {
    fn new() -> Self {
        Self {
            fault_tolerance_threshold: 0.0,
            shard_name: "".to_string(),
            parent_shard_id: "".to_string(),
            finalization_rate: 0,
            max_number_of_parents: 0,
            max_parent_depth: 0,
            synchrony_constraint_threshold: 0.0,
            height_constraint_threshold: 0,
            deploy_lifespan: 0,
            casper_version: 0,
            config_version: 0,
            bond_minimum: 0,
            bond_maximum: 0,
            epoch_length: 0,
            quarantine_length: 0,
            min_phlo_price: 0,
        }
    }
}
