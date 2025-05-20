// See casper/src/main/scala/coop/rchain/casper/Casper.scala

use block_storage::rust::dag::block_dag_key_value_storage::KeyValueDagRepresentation;
use crypto::rust::signatures::signed::Signed;
use dashmap::{DashMap, DashSet};
use models::rust::{
    block_hash::BlockHash,
    casper::protocol::casper_message::{BlockMessage, DeployData, Justification},
};

use super::genesis::contracts::validator::Validator;

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
    pub max_block_num: u64,
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
    bonds_map: DashMap<Validator, u64>,
    active_validators: Vec<Validator>,
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
    fault_tolerance_threshold: f32,
    shard_name: String,
    parent_shard_id: String,
    finalization_rate: i32,
    max_number_of_parents: i32,
    max_parent_depth: i32,
    pub synchrony_constraint_threshold: f32,
    height_constraint_threshold: i64,
    // Validators will try to put deploy in a block only for next `deployLifespan` blocks.
    // Required to enable protection from re-submitting duplicate deploys
    deploy_lifespan: i32,
    casper_version: i64,
    config_version: i64,
    bond_minimum: i64,
    bond_maximum: i64,
    epoch_length: i32,
    quarantine_length: i32,
    min_phlo_price: i64,
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
