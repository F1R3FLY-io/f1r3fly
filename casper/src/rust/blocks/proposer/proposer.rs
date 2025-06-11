// See casper/src/main/scala/coop/rchain/casper/blocks/proposer/Proposer.scala

use std::sync::Arc;

use block_storage::rust::{
    deploy::key_value_deploy_storage::KeyValueDeployStorage,
    key_value_block_store::KeyValueBlockStore,
};
use comm::rust::{
    rp::{connect::ConnectionsCell, rp_conf::RPConf},
    transport::transport_layer::TransportLayer,
};
use crypto::rust::private_key::PrivateKey;
use log;
use models::rust::casper::protocol::casper_message::BlockMessage;
use shared::rust::shared::f1r3fly_events::F1r3flyEvents;
use tokio::sync::oneshot;

use crate::rust::{
    blocks::proposer::{
        block_creator,
        propose_result::{
            BlockCreatorResult, CheckProposeConstraintsFailure, CheckProposeConstraintsResult,
            ProposeFailure, ProposeResult,
        },
    },
    casper::{Casper, CasperSnapshot},
    engine::block_retriever::BlockRetriever,
    errors::CasperError,
    last_finalized_height_constraint_checker,
    multi_parent_casper_impl::{self},
    synchrony_constraint_checker,
    util::rholang::runtime_manager::RuntimeManager,
    validator_identity::ValidatorIdentity,
    ValidBlockProcessing,
};

use super::propose_result::ProposeStatus;

pub enum ProposerResult {
    Empty,
    Success(ProposeStatus, BlockMessage),
    Failure(ProposeStatus, i32),
    Started(i32),
}

impl ProposerResult {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn success(status: ProposeStatus, block: BlockMessage) -> Self {
        Self::Success(status, block)
    }

    pub fn failure(status: ProposeStatus, seq_number: i32) -> Self {
        Self::Failure(status, seq_number)
    }

    pub fn started(seq_number: i32) -> Self {
        Self::Started(seq_number)
    }
}

pub struct Proposer<T: TransportLayer + Send + Sync> {
    pub validator: ValidatorIdentity,
    pub runtime_manager: RuntimeManager,
    pub block_store: KeyValueBlockStore,
    pub deploy_storage: KeyValueDeployStorage,
    pub transport: Arc<T>,
    pub block_retriever: BlockRetriever<T>,
    pub connections_cell: Arc<ConnectionsCell>,
    pub conf: Arc<RPConf>,
    pub event_publisher: F1r3flyEvents,
    pub dummy_deploy_opt: Option<(PrivateKey, String)>,
}

impl<T: TransportLayer + Send + Sync> Proposer<T> {
    pub fn new(
        validator: ValidatorIdentity,
        dummy_deploy_opt: Option<(PrivateKey, String)>,
        runtime_manager: RuntimeManager,
        block_store: KeyValueBlockStore,
        deploy_storage: KeyValueDeployStorage,
        transport: Arc<T>,
        block_retriever: BlockRetriever<T>,
        connections_cell: Arc<ConnectionsCell>,
        conf: Arc<RPConf>,
        event_publisher: F1r3flyEvents,
    ) -> Self {
        Self {
            validator,
            runtime_manager,
            block_store,
            deploy_storage,
            transport,
            event_publisher,
            dummy_deploy_opt,
            block_retriever,
            connections_cell,
            conf,
        }
    }

    // base state on top of which block will be created
    pub fn get_casper_snapshot(&self, casper: &dyn Casper) -> Result<CasperSnapshot, CasperError> {
        casper.get_snapshot()
    }

    pub async fn create_block(
        &self,
        casper_snapshot: &CasperSnapshot,
        validator_identity: &ValidatorIdentity,
        block_store: &mut KeyValueBlockStore,
    ) -> Result<BlockCreatorResult, CasperError> {
        block_creator::create(
            casper_snapshot,
            validator_identity,
            self.dummy_deploy_opt.clone(),
            &self.deploy_storage,
            &self.runtime_manager,
            block_store,
        )
        .await
    }

    pub fn validate_block(
        &self,
        casper: &dyn Casper,
        casper_snapshot: &CasperSnapshot,
        block: &BlockMessage,
    ) -> Result<ValidBlockProcessing, CasperError> {
        casper.validate(block, casper_snapshot)
    }

    // propose constraint checkers
    pub fn check_active_validator(
        &self,
        casper_snapshot: &CasperSnapshot,
        validator_identity: &ValidatorIdentity,
    ) -> CheckProposeConstraintsResult {
        if casper_snapshot
            .on_chain_state
            .active_validators
            .contains(&validator_identity.public_key.bytes)
        {
            CheckProposeConstraintsResult::success()
        } else {
            CheckProposeConstraintsResult::not_bonded()
        }
    }

    pub async fn check_enough_base_stake(
        &self,
        casper_snapshot: &CasperSnapshot,
    ) -> Result<CheckProposeConstraintsResult, CasperError> {
        synchrony_constraint_checker::check(
            casper_snapshot,
            &self.runtime_manager,
            &self.block_store,
            &self.validator,
        )
        .await
    }

    pub async fn check_finalized_height(
        &self,
        casper_snapshot: &CasperSnapshot,
    ) -> Result<CheckProposeConstraintsResult, CasperError> {
        last_finalized_height_constraint_checker::check(casper_snapshot, &self.validator)
    }

    // This is the whole logic of propose
    async fn do_propose(
        &mut self,
        casper_snapshot: &CasperSnapshot,
        casper: &dyn Casper,
    ) -> Result<(ProposeResult, Option<BlockMessage>), CasperError> {
        // check if node is allowed to propose a block
        let constraint_check = self.check_propose_constraints(casper_snapshot).await?;

        match constraint_check {
            CheckProposeConstraintsResult::Failure(failure) => Ok((
                ProposeResult::failure(ProposeFailure::CheckConstraintsFailure(failure)),
                None,
            )),
            CheckProposeConstraintsResult::Success => {
                let block_result = block_creator::create(
                    casper_snapshot,
                    &self.validator,
                    self.dummy_deploy_opt.clone(),
                    &self.deploy_storage,
                    &self.runtime_manager,
                    &mut self.block_store,
                )
                .await?;

                match block_result {
                    BlockCreatorResult::NoNewDeploys => {
                        Ok((ProposeResult::failure(ProposeFailure::NoNewDeploys), None))
                    }
                    BlockCreatorResult::Created(block) => {
                        let validation_result =
                            self.validate_block(casper, casper_snapshot, &block)?;

                        match validation_result {
                            ValidBlockProcessing::Right(valid_status) => {
                                self.propose_effect(casper, &block).await?;
                                Ok((ProposeResult::success(valid_status), Some(block)))
                            }
                            ValidBlockProcessing::Left(invalid_reason) => {
                                return Err(CasperError::RuntimeError(format!(
                                    "Validation of self created block failed with reason: {:?}, cancelling propose.",
                                    invalid_reason
                                )));
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if proposer can issue a block
    pub async fn check_propose_constraints(
        &self,
        casper_snapshot: &CasperSnapshot,
    ) -> Result<CheckProposeConstraintsResult, CasperError> {
        match self.check_active_validator(casper_snapshot, &self.validator) {
            CheckProposeConstraintsResult::Failure(CheckProposeConstraintsFailure::NotBonded) => {
                Ok(CheckProposeConstraintsResult::not_bonded())
            }
            _ => {
                // Run both async checks in parallel
                let (stake_result, height_result) = tokio::join!(
                    self.check_enough_base_stake(casper_snapshot),
                    self.check_finalized_height(casper_snapshot)
                );

                // Handle any errors from the async calls
                let stake_check = stake_result?;
                let height_check = height_result?;

                // Pick some result that is not Success, or return Success
                let results = vec![stake_check, height_check];
                for result in results {
                    match result {
                        CheckProposeConstraintsResult::Success => continue,
                        failure => return Ok(failure),
                    }
                }

                Ok(CheckProposeConstraintsResult::success())
            }
        }
    }

    pub async fn propose(
        &mut self,
        casper: &dyn Casper,
        is_async: bool,
        propose_id_sender: oneshot::Sender<ProposerResult>,
    ) -> Result<(ProposeResult, Option<BlockMessage>), CasperError> {
        fn get_validator_next_seq_number(
            casper_snapshot: &CasperSnapshot,
            validator_public_key: &[u8],
        ) -> i32 {
            casper_snapshot
                .max_seq_nums
                .get(validator_public_key)
                .map(|seq| *seq + 1)
                .unwrap_or(1) as i32
        }

        let start_time = std::time::Instant::now();

        // get snapshot to serve as a base for propose
        let casper_snapshot = self.get_casper_snapshot(casper)?;

        let elapsed = start_time.elapsed();
        log::info!("getCasperSnapshot [{}ms]", elapsed.as_millis());

        let result = if is_async {
            let next_seq =
                get_validator_next_seq_number(&casper_snapshot, &self.validator.public_key.bytes);
            let _ = propose_id_sender.send(ProposerResult::started(next_seq));

            // propose
            self.do_propose(&casper_snapshot, casper).await?
        } else {
            // propose
            let result = self.do_propose(&casper_snapshot, casper).await?;

            let (propose_result, block_opt) = &result;
            let proposer_result = match block_opt {
                None => {
                    let seq_number = get_validator_next_seq_number(
                        &casper_snapshot,
                        &self.validator.public_key.bytes,
                    );
                    ProposerResult::failure(propose_result.propose_status.clone(), seq_number)
                }
                Some(block) => {
                    ProposerResult::success(propose_result.propose_status.clone(), block.clone())
                }
            };
            let _ = propose_id_sender.send(proposer_result);

            result
        };

        Ok(result)
    }

    pub async fn propose_effect(
        &mut self,
        casper: &dyn Casper,
        block: &BlockMessage,
    ) -> Result<(), CasperError> {
        // store block
        self.block_store.put_block_message(block)?;

        // save changes to Casper
        casper.handle_valid_block(block)?;

        // inform block retriever about block
        self.block_retriever
            .ack_in_casper(block.block_hash.clone())
            .await?;

        // broadcast hash to peers
        self.transport
            .send_block_hash(
                &self.connections_cell,
                &self.conf,
                &block.block_hash,
                &block.sender,
            )
            .await?;

        // Publish event
        self.event_publisher
            .publish(multi_parent_casper_impl::created_event(block))
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        Ok(())
    }
}
