// See casper/src/main/scala/coop/rchain/casper/rholang/RuntimeSyntax.scala

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crypto::rust::signatures::signed::Signed;
use models::{
    casper::system_deploy_data_proto::SystemDeploy,
    rhoapi::{BindPattern, Par},
    rust::{
        block::state_hash::StateHash,
        block_hash::BlockHash,
        casper::protocol::casper_message::{
            Bond, DeployData, ProcessedDeploy, ProcessedSystemDeploy,
        },
        validator::Validator,
    },
};
use rholang::rust::interpreter::{
    rho_runtime::{bootstrap_registry, RhoRuntime, RhoRuntimeImpl},
    system_processes::BlockData,
};
use rspace_plus_plus::rspace::{
    history::instances::radix_history::RadixHistory, merger::merging_logic::NumberChannelsEndVal,
};

use crate::rust::errors::CasperError;

pub struct RuntimeOps;

impl RuntimeOps {
    /**
     * Because of the history legacy, the emptyStateHash does not really represent an empty trie.
     * The `emptyStateHash` is used as genesis block pre state which the state only contains registry
     * fixed channels in the state.
     */
    pub async fn empty_state_hash(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
    ) -> Result<StateHash, CasperError> {
        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock.reset(RadixHistory::empty_root_node_hash());
        drop(runtime_lock);

        bootstrap_registry(runtime.clone()).await;
        let mut runtime_lock = runtime.lock().unwrap();
        let checkpoint = runtime_lock.create_checkpoint();
        Ok(checkpoint.root.bytes().into())
    }

    pub fn compute_state(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: StateHash,
        terms: Vec<Signed<DeployData>>,
        system_deploys: Vec<SystemDeploy>,
        block_data: BlockData,
        invalid_blocks: HashMap<BlockHash, Validator>,
    ) -> (
        StateHash,
        Vec<(ProcessedDeploy, NumberChannelsEndVal)>,
        Vec<(ProcessedSystemDeploy, NumberChannelsEndVal)>,
    ) {
        todo!()
    }

    pub fn compute_genesis(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        terms: Vec<Signed<DeployData>>,
        block_time: i64,
        block_number: i64,
    ) -> (
        StateHash,
        StateHash,
        Vec<(ProcessedDeploy, NumberChannelsEndVal)>,
    ) {
        todo!()
    }

    /**
     * Evaluates exploratory (read-only) deploy
     */
    pub fn play_exploratory_deploy(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        term: String,
        hash: StateHash,
    ) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    // Return channel on which result is captured is the first name
    // in the deploy term `new return in { return!(42) }`
    pub fn capture_results(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start: StateHash,
        deploy: Signed<DeployData>,
    ) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    pub fn get_data_par(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        channel: Par,
    ) -> Result<Vec<Par>, CasperError> {
        todo!()
    }

    pub fn get_continuation_par(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        channels: Vec<Par>,
    ) -> Result<Vec<(Vec<BindPattern>, Par)>, CasperError> {
        todo!()
    }

    pub fn get_active_validators(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        start_hash: StateHash,
    ) -> Result<Vec<Validator>, CasperError> {
        todo!()
    }

    pub fn compute_bonds(
        runtime: Arc<Mutex<RhoRuntimeImpl>>,
        hash: StateHash,
    ) -> Result<Vec<Bond>, CasperError> {
        // let bonds_query_source = r#"
        //     new return, rl(`rho:registry:lookup`), poSCh in {
        //       rl!(`rho:rchain:pos`, *poSCh) |
        //       for(@(_, PoS) <- poSCh) {
        //         @PoS!("getBonds", *return)
        //       }
        //     }
        // "#;

        // let mut runtime_lock = runtime.lock().unwrap();
        // let bonds_par = runtime_lock
        //     .play_exploratory_deploy(bonds_query_source, hash)
        //     .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        // if bonds_par.len() != 1 {
        //     return Err(CasperError::RuntimeError(format!(
        //         "Incorrect number of results from query of current bonds in state {}: {}",
        //         hash.to_string(),
        //         bonds_par.len()
        //     )));
        // }

        // let bonds_map = bonds_par[0].exprs[0].get_emap_body().ps;
        // let mut bonds = Vec::new();

        // for (validator, bond) in bonds_map {
        //     if validator.exprs.len() != 1 {
        //         return Err(CasperError::RuntimeError(
        //             "Validator in bonds map wasn't a single string.".to_string(),
        //         ));
        //     }
        //     if bond.exprs.len() != 1 {
        //         return Err(CasperError::RuntimeError(
        //             "Stake in bonds map wasn't a single integer.".to_string(),
        //         ));
        //     }

        //     let validator_name = validator.exprs[0].get_gbyte_array();
        //     let stake_amount = bond.exprs[0].get_gint();

        //     bonds.push(Bond {
        //         validator: validator_name.into(),
        //         stake: stake_amount,
        //     });
        // }

        // Ok(bonds)
        todo!()
    }
}
