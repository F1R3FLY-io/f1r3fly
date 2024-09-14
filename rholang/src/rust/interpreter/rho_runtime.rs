// See rholang/src/main/scala/coop/rchain/rholang/interpreter/RhoRuntime.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::expr::ExprInstance::EMapBody;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::Bundle;
use models::rhoapi::Var;
use models::rhoapi::{BindPattern, Expr, ListParWithRandom, Par, TaggedContinuation};
use models::rust::block_hash::BlockHash;
use models::rust::par_map::ParMap;
use models::rust::par_map_type_mapper::ParMapTypeMapper;
use models::rust::sorted_par_map::SortedParMap;
use models::rust::utils::new_freevar_par;
use models::rust::validator::Validator;
use rspace_plus_plus::rspace::checkpoint::{Checkpoint, SoftCheckpoint};
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::history::history_repository_impl::HistoryRepositoryImpl;
use rspace_plus_plus::rspace::internal::{Datum, Row, WaitingContinuation};
use rspace_plus_plus::rspace::r#match::Match;
use rspace_plus_plus::rspace::rspace::RSpace;
use rspace_plus_plus::rspace::rspace::RSpaceInstances;
use rspace_plus_plus::rspace::rspace::RSpaceStore;
use rspace_plus_plus::rspace::trace::Log;
use rspace_plus_plus::rspace::util::unpack_option;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use crate::rust::interpreter::system_processes::{BodyRefs, FixedChannels};

use super::accounting::_cost;
use super::accounting::cost_accounting::CostAccounting;
use super::accounting::costs::Cost;
use super::accounting::has_cost::HasCost;
use super::dispatch::RhoDispatch;
use super::dispatch::RholangAndScalaDispatcher;
use super::env::Env;
use super::errors::InterpreterError;
use super::interpreter::{EvaluateResult, Interpreter, InterpreterImpl};
use super::reduce::DebruijnInterpreter;
use super::registry::registry_bootstrap::ast;
use super::storage::charging_rspace::ChargingRSpace;
use super::system_processes::{
    Arity, BlockData, BodyRef, Definition, InvalidBlocks, Name, ProcessContext, Remainder,
    RhoDispatchMap,
};
use models::rhoapi::expr::ExprInstance::GByteArray;

pub trait RhoRuntime: HasCost {
    /**
     * Parse the rholang term into [[coop.rchain.models.Par]] and execute it with provided initial phlo.
     *
     * This function would change the state in the runtime.
     * @param term The rholang contract which would run on the runtime
     * @param initialPhlo initial cost for the this evaluation. If the phlo is not enough,
     *                    [[coop.rchain.rholang.interpreter.errors.OutOfPhlogistonsError]] would return.
     * @param normalizerEnv additional env for Par when parsing term into Par
     * @param rand random seed for rholang execution
     * @return
     */
    fn evaluate(
        &self,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b256Hash,
    ) -> EvaluateResult;

    /**
     * The function would execute the par regardless setting cost which would possibly cause
     * [[coop.rchain.rholang.interpreter.errors.OutOfPhlogistonsError]]. Because of that, use this
     * function in some situation which is not cost sensitive.
     *
     * This function would change the state in the runtime.
     *
     * Ideally, this function should be removed or hack the runtime without cost accounting in the future .
     * @param par [[coop.rchain.models.Par]] for the execution
     * @param env additional env for execution
     * @param rand random seed for rholang execution
     * @return
     */
    async fn inj(
        &self,
        par: Par,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError>;

    /**
     * After some executions([[evaluate]]) on the runtime, you can create a soft checkpoint which is the changes
     * for the current state of the runtime. You can revert the changes by [[revertToSoftCheckpoint]]
     * @return
     */
    fn create_soft_checkpoint(
        &mut self,
    ) -> SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>;

    fn revert_to_soft_checkpoint(
        &mut self,
        soft_checkpoint: SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    ) -> ();

    /**
     * Create a checkpoint for the runtime. All the changes which happened in the runtime would persistent in the disk
     * and result in a new stateHash for the new state.
     * @return
     */
    fn create_checkpoint(&mut self) -> Checkpoint;

    /**
     * Reset the runtime to the specific state. Then you can operate some execution on the state.
     * @param root the target state hash to reset
     * @return
     */
    fn reset(&mut self, root: Blake2b256Hash) -> ();

    /**
     * Consume the result in the rspace.
     *
     * This function would change the state in the runtime.
     * @param channel target channel for the consume
     * @param pattern pattern for the consume
     * @return
     */
    fn consume_result(
        &mut self,
        channel: Vec<Par>,
        pattern: Vec<BindPattern>,
    ) -> Option<(TaggedContinuation, Vec<ListParWithRandom>)>;

    /**
     * get data directly from history repository
     *
     * This function would not change the state in the runtime
     */
    fn get_data(&self, channel: Par) -> Vec<Datum<ListParWithRandom>>;

    fn get_joins(&self, channel: Par) -> Vec<Vec<Par>>;

    /**
     * get continuation directly from history repository
     *
     * This function would not change the state in the runtime
     */
    fn get_continuation(
        &self,
        channels: Vec<Par>,
    ) -> Vec<WaitingContinuation<BindPattern, TaggedContinuation>>;

    /**
     * Set the runtime block data environment.
     */
    fn set_block_data(&self, block_data: BlockData) -> ();

    /**
     * Set the runtime invalid blocks environment.
     */
    fn set_invalid_blocks(&self, invalid_blocks: HashMap<BlockHash, Validator>) -> ();

    /**
     * Get the hot changes after some executions for the runtime.
     * Currently this is only for debug info mostly.
     */
    fn get_hot_changes(
        &self,
    ) -> HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>>;
}

pub trait ReplayRhoRuntime: RhoRuntime {
    fn rig(&self, log: Log) -> ();

    fn check_replay_data(&self) -> ();
}

#[derive(Clone)]
pub struct RhoRuntimeImpl {
    reducer: DebruijnInterpreter,
    space: RhoISpace,
    cost: _cost,
    block_data_ref: Arc<RwLock<BlockData>>,
    invalid_blocks_param: InvalidBlocks,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
}

impl RhoRuntimeImpl {
    fn new(
        reducer: DebruijnInterpreter,
        space: RhoISpace,
        cost: _cost,
        block_data_ref: Arc<RwLock<BlockData>>,
        invalid_blocks_param: InvalidBlocks,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
    ) -> Arc<Mutex<RhoRuntimeImpl>> {
        Arc::new(Mutex::new(RhoRuntimeImpl {
            reducer,
            space,
            cost,
            block_data_ref,
            invalid_blocks_param,
            merge_chs,
        }))
    }
}

impl RhoRuntime for RhoRuntimeImpl {
    fn evaluate(
        &self,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b256Hash,
    ) -> EvaluateResult {
        let i = InterpreterImpl::new(self.cost.clone(), self.merge_chs.clone());
        let reducer = &self.reducer;
        i.inj_attempt(reducer, term, initial_phlo, normalizer_env, rand)
    }

    async fn inj(
        &self,
        par: Par,
        _env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        self.reducer.inj(par, rand).await
    }

    fn create_soft_checkpoint(
        &mut self,
    ) -> SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation> {
        self.space.lock().unwrap().create_soft_checkpoint()
    }

    fn revert_to_soft_checkpoint(
        &mut self,
        soft_checkpoint: SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    ) -> () {
        self.space
            .lock()
            .unwrap()
            .revert_to_soft_checkpoint(soft_checkpoint)
            .unwrap()
    }

    fn create_checkpoint(&mut self) -> Checkpoint {
        self.space.lock().unwrap().create_checkpoint().unwrap()
    }

    fn reset(&mut self, root: Blake2b256Hash) -> () {
        self.space.lock().unwrap().reset(root).unwrap()
    }

    fn consume_result(
        &mut self,
        channel: Vec<Par>,
        pattern: Vec<BindPattern>,
    ) -> Option<(TaggedContinuation, Vec<ListParWithRandom>)> {
        let v = self.space.lock().unwrap().consume(
            channel,
            pattern,
            TaggedContinuation::default(),
            false,
            BTreeSet::new(),
        );

        unpack_option(&v)
    }

    fn get_data(&self, channel: Par) -> Vec<Datum<ListParWithRandom>> {
        self.space.lock().unwrap().get_data(channel)
    }

    fn get_joins(&self, channel: Par) -> Vec<Vec<Par>> {
        self.space.lock().unwrap().get_joins(channel)
    }

    fn get_continuation(
        &self,
        channels: Vec<Par>,
    ) -> Vec<WaitingContinuation<BindPattern, TaggedContinuation>> {
        self.space
            .lock()
            .unwrap()
            .get_waiting_continuations(channels)
    }

    fn set_block_data(&self, block_data: BlockData) -> () {
        let mut lock = self.block_data_ref.write().unwrap();
        *lock = block_data;
    }

    fn set_invalid_blocks(&self, invalid_blocks: HashMap<BlockHash, Validator>) -> () {
        let invalid_blocks: Par = Par::default().with_exprs(vec![Expr {
            expr_instance: Some(EMapBody(ParMapTypeMapper::par_map_to_emap(
                ParMap::create_from_sorted_par_map(SortedParMap::create_from_map(
                    invalid_blocks
                        .into_iter()
                        .map(|(validator, block_hash)| {
                            (
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(GByteArray(validator)),
                                }]),
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(GByteArray(block_hash)),
                                }]),
                            )
                        })
                        .collect(),
                )),
            ))),
        }]);

        self.invalid_blocks_param.set_params(invalid_blocks)
    }

    fn get_hot_changes(
        &self,
    ) -> HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>> {
        self.space.lock().unwrap().to_map()
    }
}

// TODO: Where is this implementation for the scala code?
impl HasCost for RhoRuntimeImpl {
    fn cost(&self) -> &_cost {
        todo!()
    }
}

pub struct ReplayRhoRuntimeImpl {
    reducer: DebruijnInterpreter,
    space: RhoISpace,
    cost: _cost,
    block_data_ref: Arc<RwLock<BlockData>>,
    invalid_blocks_param: InvalidBlocks,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
}

impl ReplayRhoRuntimeImpl {
    pub fn new(
        reducer: DebruijnInterpreter,
        space: RhoReplayISpace,
        cost: _cost,
        block_data_ref: Arc<RwLock<BlockData>>,
        invalid_blocks_param: InvalidBlocks,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
    ) -> Arc<Mutex<ReplayRhoRuntimeImpl>> {
        Arc::new(Mutex::new(ReplayRhoRuntimeImpl {
            reducer,
            space,
            cost,
            block_data_ref,
            invalid_blocks_param,
            merge_chs,
        }))
    }
}

impl ReplayRhoRuntime for ReplayRhoRuntimeImpl {
    fn rig(&self, log: Log) -> () {
        self.space.lock().unwrap().rig(log)
    }

    fn check_replay_data(&self) -> () {
        self.space.lock().unwrap().check_replay_data()
    }
}

impl RhoRuntime for ReplayRhoRuntimeImpl {
    fn evaluate(
        &self,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b256Hash,
    ) -> EvaluateResult {
        let i = InterpreterImpl::new(self.cost.clone(), self.merge_chs.clone());
        let reducer = &self.reducer;
        i.inj_attempt(reducer, term, initial_phlo, normalizer_env, rand)
    }

    async fn inj(
        &self,
        par: Par,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        self.reducer.inj(par, rand).await
    }

    fn create_soft_checkpoint(
        &mut self,
    ) -> SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation> {
        self.space.lock().unwrap().create_soft_checkpoint()
    }

    fn revert_to_soft_checkpoint(
        &mut self,
        soft_checkpoint: SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    ) -> () {
        self.space
            .lock()
            .unwrap()
            .revert_to_soft_checkpoint(soft_checkpoint)
            .unwrap()
    }

    fn create_checkpoint(&mut self) -> Checkpoint {
        self.space
            .lock()
            .unwrap()
            .replay_create_checkpoint()
            .unwrap()
    }

    fn reset(&mut self, root: Blake2b256Hash) -> () {
        self.space.lock().unwrap().reset(root).unwrap()
    }

    fn consume_result(
        &mut self,
        channel: Vec<Par>,
        pattern: Vec<BindPattern>,
    ) -> Option<(TaggedContinuation, Vec<ListParWithRandom>)> {
        let v = self.space.lock().unwrap().replay_consume(
            channel,
            pattern,
            TaggedContinuation::default(),
            false,
            BTreeSet::new(),
        );

        unpack_option(&v)
    }

    fn get_data(&self, channel: Par) -> Vec<Datum<ListParWithRandom>> {
        self.space.lock().unwrap().get_data(channel)
    }

    fn get_joins(&self, channel: Par) -> Vec<Vec<Par>> {
        self.space.lock().unwrap().get_joins(channel)
    }

    fn get_continuation(
        &self,
        channels: Vec<Par>,
    ) -> Vec<WaitingContinuation<BindPattern, TaggedContinuation>> {
        self.space
            .lock()
            .unwrap()
            .get_waiting_continuations(channels)
    }

    fn set_block_data(&self, block_data: BlockData) -> () {
        let mut lock = self.block_data_ref.write().unwrap();
        *lock = block_data;
    }

    fn set_invalid_blocks(&self, invalid_blocks: HashMap<BlockHash, Validator>) -> () {
        let invalid_blocks: Par = Par::default().with_exprs(vec![Expr {
            expr_instance: Some(EMapBody(ParMapTypeMapper::par_map_to_emap(
                ParMap::create_from_sorted_par_map(SortedParMap::create_from_map(
                    invalid_blocks
                        .into_iter()
                        .map(|(validator, block_hash)| {
                            (
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(GByteArray(validator)),
                                }]),
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(GByteArray(block_hash)),
                                }]),
                            )
                        })
                        .collect(),
                )),
            ))),
        }]);

        self.invalid_blocks_param.set_params(invalid_blocks)
    }

    fn get_hot_changes(
        &self,
    ) -> HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>> {
        self.space.lock().unwrap().to_map()
    }
}

// TODO: Where is this implementation for the scala code?
impl HasCost for ReplayRhoRuntimeImpl {
    fn cost(&self) -> &_cost {
        todo!()
    }
}

pub type RhoTuplespace =
    Arc<Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>>;
pub type RhoISpace = Arc<Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>>;
pub type RhoReplayISpace =
    Arc<Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>>;
pub type ISpaceAndReplay = (RhoISpace, RhoReplayISpace);
pub type RhoHistoryRepository =
    HistoryRepositoryImpl<Par, BindPattern, ListParWithRandom, TaggedContinuation>;

fn introduce_system_process(
    mut spaces: Vec<RhoTuplespace>,
    processes: Vec<(Name, Arity, Remainder, BodyRef)>,
) -> Vec<Option<(TaggedContinuation, Vec<ListParWithRandom>)>> {
    let mut results = Vec::new();

    for (name, arity, remainder, body_ref) in processes {
        let channels = vec![name];
        let patterns = vec![BindPattern {
            patterns: (0..arity).map(|i| new_freevar_par(i, Vec::new())).collect(),
            remainder,
            free_count: arity,
        }];

        let continuation = TaggedContinuation {
            tagged_cont: Some(TaggedCont::ScalaBodyRef(body_ref)),
        };

        for space in &mut spaces {
            let result = space.lock().unwrap().install(
                channels.clone(),
                patterns.clone(),
                continuation.clone(),
            );
            results.push(result);
        }
    }

    results
}

fn std_system_processes() -> Vec<Definition> {
    vec![
        Definition {
            urn: "rho:io:stdout".to_string(),
            fixed_channel: FixedChannels::stdout(),
            arity: 1,
            body_ref: BodyRefs::STDOUT,
            handler: Box::new(|mut ctx| Box::new(move |args| ctx.system_processes.std_out(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stdoutAck".to_string(),
            fixed_channel: FixedChannels::stdout_ack(),
            arity: 2,
            body_ref: BodyRefs::STDOUT_ACK,
            handler: Box::new(|mut ctx| {
                Box::new(move |args| ctx.system_processes.std_out_ack(args))
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stderr".to_string(),
            fixed_channel: FixedChannels::stderr(),
            arity: 1,
            body_ref: BodyRefs::STDERR,
            handler: Box::new(|mut ctx| Box::new(move |args| ctx.system_processes.std_err(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stderrAck".to_string(),
            fixed_channel: FixedChannels::stderr_ack(),
            arity: 2,
            body_ref: BodyRefs::STDERR_ACK,
            handler: Box::new(|mut ctx| {
                Box::new(move |args| ctx.system_processes.std_err_ack(args))
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:block:data".to_string(),
            fixed_channel: FixedChannels::get_block_data(),
            arity: 1,
            body_ref: BodyRefs::GET_BLOCK_DATA,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    ctx.system_processes
                        .get_block_data(args, ctx.block_data.clone())
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:casper:invalidBlocks".to_string(),
            fixed_channel: FixedChannels::get_invalid_blocks(),
            arity: 1,
            body_ref: BodyRefs::GET_INVALID_BLOCKS,
            handler: Box::new(|ctx| {
                Box::new(move |args| {
                    ctx.system_processes
                        .invalid_blocks(args, &ctx.invalid_blocks)
                })
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:rev:address".to_string(),
            fixed_channel: FixedChannels::rev_address(),
            arity: 3,
            body_ref: BodyRefs::REV_ADDRESS,
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.rev_address(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:rchain:deployerId:ops".to_string(),
            fixed_channel: FixedChannels::deployer_id_ops(),
            arity: 3,
            body_ref: BodyRefs::DEPLOYER_ID_OPS,
            handler: Box::new(|ctx| {
                Box::new(move |args| ctx.system_processes.deployer_id_ops(args))
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:registry:ops".to_string(),
            fixed_channel: FixedChannels::reg_ops(),
            arity: 3,
            body_ref: BodyRefs::REG_OPS,
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.registry_ops(args))),
            remainder: None,
        },
        Definition {
            urn: "sys:authToken:ops".to_string(),
            fixed_channel: FixedChannels::sys_authtoken_ops(),
            arity: 3,
            body_ref: BodyRefs::SYS_AUTHTOKEN_OPS,
            handler: Box::new(|ctx| {
                Box::new(move |args| ctx.system_processes.sys_auth_token_ops(args))
            }),
            remainder: None,
        },
    ]
}

fn std_rho_crypto_processes() -> Vec<Definition> {
    vec![
        Definition {
            urn: "rho:crypto:secp256k1Verify".to_string(),
            fixed_channel: FixedChannels::secp256k1_verify(),
            arity: 4,
            body_ref: BodyRefs::SECP256K1_VERIFY,
            handler: Box::new(|ctx| {
                Box::new(move |args| ctx.system_processes.secp256k1_verify(args))
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:crypto:blake2b256Hash".to_string(),
            fixed_channel: FixedChannels::blake2b256_hash(),
            arity: 2,
            body_ref: BodyRefs::BLAKE2B256_HASH,
            handler: Box::new(|ctx| {
                Box::new(move |args| ctx.system_processes.blake2b256_hash(args))
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:crypto:keccak256Hash".to_string(),
            fixed_channel: FixedChannels::keccak256_hash(),
            arity: 2,
            body_ref: BodyRefs::KECCAK256_HASH,
            handler: Box::new(|ctx| {
                Box::new(move |args| ctx.system_processes.keccak256_hash(args))
            }),
            remainder: None,
        },
        Definition {
            urn: "rho:crypto:sha256Hash".to_string(),
            fixed_channel: FixedChannels::sha256_hash(),
            arity: 2,
            body_ref: BodyRefs::SHA256_HASH,
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.sha256_hash(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:crypto:ed25519Verify".to_string(),
            fixed_channel: FixedChannels::ed25519_verify(),
            arity: 4,
            body_ref: BodyRefs::ED25519_VERIFY,
            handler: Box::new(|ctx| {
                Box::new(move |args| ctx.system_processes.ed25519_verify(args))
            }),
            remainder: None,
        },
    ]
}

fn dispatch_table_creator(
    space: RhoTuplespace,
    dispatcher: RhoDispatch,
    block_data: Arc<RwLock<BlockData>>,
    invalid_blocks: InvalidBlocks,
    extra_system_processes: &mut Vec<Definition>,
) -> RhoDispatchMap {
    let mut dispatch_table = HashMap::new();

    for def in std_system_processes().iter_mut().chain(
        std_rho_crypto_processes()
            .iter_mut()
            .chain(extra_system_processes.iter_mut()),
    ) {
        let tuple = def.to_dispatch_table(ProcessContext::create(
            space.clone(),
            dispatcher.clone(),
            block_data.clone(),
            invalid_blocks.clone(),
        ));

        dispatch_table.insert(tuple.0, tuple.1);
    }

    dispatch_table
}

fn basic_processes() -> HashMap<String, Par> {
    let mut map = HashMap::new();

    map.insert(
        "rho:registry:lookup".to_string(),
        Par::default().with_bundles(vec![Bundle {
            body: Some(FixedChannels::reg_lookup()),
            write_flag: true,
            read_flag: false,
        }]),
    );

    map.insert(
        "rho:registry:insertArbitrary".to_string(),
        Par::default().with_bundles(vec![Bundle {
            body: Some(FixedChannels::reg_insert_random()),
            write_flag: true,
            read_flag: false,
        }]),
    );

    map.insert(
        "rho:registry:insertSigned:secp256k1".to_string(),
        Par::default().with_bundles(vec![Bundle {
            body: Some(FixedChannels::reg_insert_signed()),
            write_flag: true,
            read_flag: false,
        }]),
    );

    map
}

fn setup_reducer(
    charging_rspace: RhoTuplespace,
    block_data_ref: Arc<RwLock<BlockData>>,
    invalid_blocks: InvalidBlocks,
    extra_system_processes: &mut Vec<Definition>,
    urn_map: HashMap<String, Par>,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
    mergeable_tag_name: Par,
    cost: _cost,
) -> DebruijnInterpreter {
    let replay_dispatch_table = dispatch_table_creator(
        charging_rspace.clone(),
        Arc::new(Mutex::new(RholangAndScalaDispatcher {
            _dispatch_table: Arc::new(Mutex::new(HashMap::new())),
            reducer: None,
        })),
        block_data_ref,
        invalid_blocks,
        extra_system_processes,
    );

    let (_, replay_reducer) = RholangAndScalaDispatcher::create(
        charging_rspace.clone(),
        replay_dispatch_table,
        urn_map,
        merge_chs,
        mergeable_tag_name,
        cost,
    );

    replay_reducer
}

fn setup_maps_and_refs(
    extra_system_processes: &Vec<Definition>,
) -> (
    Arc<RwLock<BlockData>>,
    InvalidBlocks,
    HashMap<String, Name>,
    Vec<(Name, Arity, Remainder, BodyRef)>,
) {
    let block_data_ref = Arc::new(RwLock::new(BlockData::empty()));
    let invalid_blocks = InvalidBlocks::new();

    let system_binding = std_system_processes();
    let rho_crypto_binding = std_rho_crypto_processes();
    let combined_processes = system_binding
        .iter()
        .chain(rho_crypto_binding.iter())
        .chain(extra_system_processes.iter())
        .collect::<Vec<&Definition>>();

    let urn_map = combined_processes
        .iter()
        .map(|process| process.to_urn_map())
        .collect::<HashMap<String, Par>>();

    let proc_defs: Vec<(Par, i32, Option<Var>, i64)> = combined_processes
        .iter()
        .map(|process| process.to_proc_defs())
        .collect();

    (block_data_ref, invalid_blocks, urn_map, proc_defs)
}

fn create_rho_env(
    rspace: RhoISpace,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
    mergeable_tag_name: Par,
    extra_system_processes: &mut Vec<Definition>,
    cost: _cost,
) -> (DebruijnInterpreter, Arc<RwLock<BlockData>>, InvalidBlocks) {
    let maps_and_refs = setup_maps_and_refs(&extra_system_processes);
    let (block_data_ref, invalid_blocks, urn_map, proc_defs) = maps_and_refs;
    let reducer = setup_reducer(
        ChargingRSpace::charging_rspace(rspace.clone()),
        block_data_ref.clone(),
        invalid_blocks.clone(),
        extra_system_processes,
        urn_map,
        merge_chs,
        mergeable_tag_name,
        cost,
    );

    let res = introduce_system_process(vec![rspace], proc_defs);
    assert!(res.iter().all(|s| s.is_none()));
    (reducer, block_data_ref, invalid_blocks)
}

// This is from Nassim Taleb's "Skin in the Game"
fn bootstrap_rand() -> Blake2b512Random {
    Blake2b512Random::new("Decentralization is based on the simple notion that it is easier to macrobull***t than microbull***t. \
         Decentralization reduces large structural asymmetries."
         .as_bytes())
}

fn bootstrap_registry(runtime: Arc<Mutex<impl RhoRuntime>>) -> () {
    let rand = bootstrap_rand();
    let runtime_lock = runtime.lock().unwrap();
    let cost = runtime_lock.cost().get();
    let _ = runtime_lock
        .cost()
        .set(Cost::create(i64::MAX, "bootstrap registry".to_string()));
    let _ = runtime_lock.inj(ast(), Env::new(), rand);
    let _ = runtime_lock.cost().set(Cost::create_from_cost(cost));
}

fn create_runtime(
    rspace: RhoISpace,
    extra_system_processes: &mut Vec<Definition>,
    init_registry: bool,
    mergeable_tag_name: Par,
) -> Arc<Mutex<RhoRuntimeImpl>> {
    let cost = CostAccounting::empty_cost();
    let merge_chs = Arc::new(RwLock::new({
        let mut set = HashSet::new();
        set.insert(Par::default());
        set
    }));

    let rho_env = create_rho_env(
        rspace.clone(),
        merge_chs.clone(),
        mergeable_tag_name,
        extra_system_processes,
        cost.clone(),
    );

    let (reducer, block_ref, invalid_blocks) = rho_env;
    let runtime = RhoRuntimeImpl::new(reducer, rspace, cost, block_ref, invalid_blocks, merge_chs);

    if init_registry {
        let _ = bootstrap_registry(runtime.clone());
        let _ = runtime.lock().unwrap().create_checkpoint();
    }

    runtime
}

/**
 *
 * @param rspace the rspace which the runtime would operate on it
 * @param extraSystemProcesses extra system rholang processes exposed to the runtime
 *                             which you can execute function on it
 * @param initRegistry For a newly created rspace, you might need to bootstrap registry
 *                     in the runtime to use rholang registry normally. Actually this initRegistry
 *                     is not the only thing you need for rholang registry, after the bootstrap
 *                     registry, you still need to insert registry contract on the rspace.
 *                     For a exist rspace which bootstrap registry before, you can skip this.
 *                     For some test cases, you don't need the registry then you can skip this
 *                     init process which can be faster.
 * @param costLog currently only the testcases needs a special costLog for test information.
 *                Normally you can just
 *                use [[coop.rchain.rholang.interpreter.accounting.noOpCostLog]]
 * @return
 */
fn create_rho_runtime(
    rspace: RhoISpace,
    mergeable_tag_name: Par,
    init_registry: bool,
    extra_system_processes: &mut Vec<Definition>,
) -> Arc<Mutex<RhoRuntimeImpl>> {
    create_runtime(
        rspace,
        extra_system_processes,
        init_registry,
        mergeable_tag_name,
    )
}

/**
 *
 * @param rspace the replay rspace which the runtime operate on it
 * @param extraSystemProcesses same as [[coop.rchain.rholang.interpreter.RhoRuntime.createRhoRuntime]]
 * @param initRegistry same as [[coop.rchain.rholang.interpreter.RhoRuntime.createRhoRuntime]]
 * @param costLog same as [[coop.rchain.rholang.interpreter.RhoRuntime.createRhoRuntime]]
 * @return
 */
fn create_replay_rho_runtime(
    rspace: RhoReplayISpace,
    mergeable_tag_name: Par,
    init_registry: bool,
    extra_system_processes: &mut Vec<Definition>,
) -> Arc<Mutex<ReplayRhoRuntimeImpl>> {
    let cost = CostAccounting::empty_cost();
    let merge_chs = Arc::new(RwLock::new({
        let mut set = HashSet::new();
        set.insert(Par::default());
        set
    }));

    let rho_env = create_rho_env(
        rspace.clone(),
        merge_chs.clone(),
        mergeable_tag_name,
        extra_system_processes,
        cost.clone(),
    );

    let (reducer, block_ref, invalid_blocks) = rho_env;
    let runtime =
        ReplayRhoRuntimeImpl::new(reducer, rspace, cost, block_ref, invalid_blocks, merge_chs);

    if init_registry {
        let _ = bootstrap_registry(runtime.clone());
        let _ = runtime.lock().unwrap().create_checkpoint();
    }

    runtime
}

fn create_runtimes(
    space: RhoISpace,
    replay_space: RhoReplayISpace,
    init_registry: bool,
    additional_system_processes: &mut Vec<Definition>,
    mergeable_tag_name: Par,
) -> (Arc<Mutex<RhoRuntimeImpl>>, Arc<Mutex<ReplayRhoRuntimeImpl>>) {
    let rho_runtime = create_rho_runtime(
        space.clone(),
        mergeable_tag_name.clone(),
        init_registry,
        additional_system_processes,
    );

    let replay_rho_runtime = create_replay_rho_runtime(
        replay_space,
        mergeable_tag_name,
        init_registry,
        additional_system_processes,
    );

    (rho_runtime, replay_rho_runtime)
}

fn create_runtime_from_kv_store(
    stores: RSpaceStore,
    mergeable_tag_name: Par,
    init_registry: bool,
    additional_system_processes: &mut Vec<Definition>,
    matcher: Arc<Box<dyn Match<BindPattern, ListParWithRandom>>>,
) -> Arc<Mutex<RhoRuntimeImpl>> {
    let space = RSpaceInstances::create::<Par, BindPattern, ListParWithRandom, TaggedContinuation>(
        stores, matcher,
    )
    .unwrap();

    let runtime = create_rho_runtime(
        Arc::new(Mutex::new(space)),
        mergeable_tag_name,
        init_registry,
        additional_system_processes,
    );

    runtime
}
