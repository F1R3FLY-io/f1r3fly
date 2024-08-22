// use lazy_static::lazy_static;
use models::rhoapi::expr::ExprInstance::EMapBody;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::Bundle;
use models::rhoapi::Var;
use models::rhoapi::{BindPattern, Expr, ListParWithRandom, Par, TaggedContinuation};
use models::rhoapi::{EMap, KeyValuePair};
use models::rust::block_hash::BlockHash;
use models::rust::utils::new_freevar_par;
use models::rust::validator::Validator;
use rspace_plus_plus::rspace::checkpoint::{Checkpoint, SoftCheckpoint};
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::history::history_repository_impl::HistoryRepositoryImpl;
use rspace_plus_plus::rspace::internal::{Datum, Row, WaitingContinuation};
use rspace_plus_plus::rspace::rspace::RSpace;
use rspace_plus_plus::rspace::trace::Log;
use rspace_plus_plus::rspace::util::unpack_option;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex, RwLock};

use crate::rust::interpreter::system_processes::{BodyRefs, FixedChannels};

use super::accounting::costs::Cost;
use super::accounting::has_cost::{CostState, HasCost};
use super::dispatch::RholangAndRustDispatcher;
use super::env::Env;
use super::interpreter::{EvaluateResult, Interpreter, InterpreterImpl};
use super::reduce::{DebruijnInterpreter, Reduce};
use super::system_processes::{
    Arity, BlockData, BodyRef, Definition, InvalidBlocks, Name, ProcessContext, Remainder,
    RhoDispatchMap, SystemProcesses,
};
use models::rhoapi::expr::ExprInstance::GByteArray;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/RhoRuntime.scala
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
    fn inj(&self, par: Par, env: Env<Par>, rand: Blake2b256Hash) -> ();

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

pub struct RhoRuntimeImpl {
    reducer: DebruijnInterpreter,
    space: RhoISpace,
    cost: CostState,
    block_data_ref: Arc<RwLock<BlockData>>,
    invalid_blocks_param: InvalidBlocks,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
}

impl RhoRuntimeImpl {
    fn new(
        reducer: DebruijnInterpreter,
        space: RhoISpace,
        cost: CostState,
        block_data_ref: Arc<RwLock<BlockData>>,
        invalid_blocks_param: InvalidBlocks,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
    ) -> RhoRuntimeImpl {
        RhoRuntimeImpl {
            reducer,
            space,
            cost,
            block_data_ref,
            invalid_blocks_param,
            merge_chs,
        }
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

    fn inj(&self, par: Par, _env: Env<Par>, rand: Blake2b256Hash) -> () {
        self.reducer.inj(par, rand)
    }

    fn create_soft_checkpoint(
        &mut self,
    ) -> SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation> {
        self.space.create_soft_checkpoint()
    }

    fn revert_to_soft_checkpoint(
        &mut self,
        soft_checkpoint: SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    ) -> () {
        self.space
            .revert_to_soft_checkpoint(soft_checkpoint)
            .unwrap()
    }

    fn create_checkpoint(&mut self) -> Checkpoint {
        self.space.create_checkpoint().unwrap()
    }

    fn reset(&mut self, root: Blake2b256Hash) -> () {
        self.space.reset(root).unwrap()
    }

    fn consume_result(
        &mut self,
        channel: Vec<Par>,
        pattern: Vec<BindPattern>,
    ) -> Option<(TaggedContinuation, Vec<ListParWithRandom>)> {
        let v = self.space.consume(
            channel,
            pattern,
            TaggedContinuation::default(),
            false,
            BTreeSet::new(),
        );

        unpack_option(&v)
    }

    fn get_data(&self, channel: Par) -> Vec<Datum<ListParWithRandom>> {
        self.space.get_data(channel)
    }

    fn get_joins(&self, channel: Par) -> Vec<Vec<Par>> {
        self.space.get_joins(channel)
    }

    fn get_continuation(
        &self,
        channels: Vec<Par>,
    ) -> Vec<WaitingContinuation<BindPattern, TaggedContinuation>> {
        self.space.get_waiting_continuations(channels)
    }

    fn set_block_data(&self, block_data: BlockData) -> () {
        let mut lock = self.block_data_ref.write().unwrap();
        *lock = block_data;
    }

    fn set_invalid_blocks(&self, invalid_blocks: HashMap<BlockHash, Validator>) -> () {
        let invalid_blocks: Par = Par::default().with_exprs(vec![Expr {
            expr_instance: Some(EMapBody(EMap {
                kvs: {
                    invalid_blocks
                        .into_iter()
                        .map(|(validator, block_hash)| KeyValuePair {
                            key: Some(Par::default().with_exprs(vec![Expr {
                                expr_instance: Some(GByteArray(validator)),
                            }])),
                            value: Some(Par::default().with_exprs(vec![Expr {
                                expr_instance: Some(GByteArray(block_hash)),
                            }])),
                        })
                        .collect()
                },
                locally_free: Vec::new(),
                connective_used: false,
                remainder: None,
            })),
        }]);

        self.invalid_blocks_param.set_params(invalid_blocks)
    }

    fn get_hot_changes(
        &self,
    ) -> HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>> {
        self.space.to_map()
    }
}

// TODO: Where is this implementation for the scala code?
impl HasCost for RhoRuntimeImpl {
    fn cost(&self) -> &CostState {
        todo!()
    }
}

pub struct ReplayRhoRuntimeImpl {
    runtime: RhoRuntimeImpl,
}

impl ReplayRhoRuntimeImpl {
    pub fn new(
        reducer: DebruijnInterpreter,
        space: RhoISpace,
        cost: CostState,
        block_data_ref: Arc<RwLock<BlockData>>,
        invalid_blocks_param: InvalidBlocks,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
    ) -> Self {
        ReplayRhoRuntimeImpl {
            runtime: RhoRuntimeImpl {
                reducer,
                space,
                cost,
                block_data_ref,
                invalid_blocks_param,
                merge_chs,
            },
        }
    }
}

impl ReplayRhoRuntime for ReplayRhoRuntimeImpl {
    fn rig(&self, log: Log) -> () {
        self.runtime.space.rig(log)
    }

    fn check_replay_data(&self) -> () {
        self.runtime.space.check_replay_data()
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
        self.runtime
            .evaluate(term, initial_phlo, normalizer_env, rand)
    }

    fn inj(&self, par: Par, env: Env<Par>, rand: Blake2b256Hash) -> () {
        self.runtime.inj(par, env, rand)
    }

    fn create_soft_checkpoint(
        &mut self,
    ) -> SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation> {
        self.runtime.create_soft_checkpoint()
    }

    fn revert_to_soft_checkpoint(
        &mut self,
        soft_checkpoint: SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    ) -> () {
        self.runtime.revert_to_soft_checkpoint(soft_checkpoint)
    }

    fn create_checkpoint(&mut self) -> Checkpoint {
        self.runtime.create_checkpoint()
    }

    fn reset(&mut self, root: Blake2b256Hash) -> () {
        self.runtime.reset(root)
    }

    fn consume_result(
        &mut self,
        channel: Vec<Par>,
        pattern: Vec<BindPattern>,
    ) -> Option<(TaggedContinuation, Vec<ListParWithRandom>)> {
        self.runtime.consume_result(channel, pattern)
    }

    fn get_data(&self, channel: Par) -> Vec<Datum<ListParWithRandom>> {
        self.runtime.get_data(channel)
    }

    fn get_joins(&self, channel: Par) -> Vec<Vec<Par>> {
        self.runtime.get_joins(channel)
    }

    fn get_continuation(
        &self,
        channels: Vec<Par>,
    ) -> Vec<WaitingContinuation<BindPattern, TaggedContinuation>> {
        self.runtime.get_continuation(channels)
    }

    fn set_block_data(&self, block_data: BlockData) -> () {
        self.runtime.set_block_data(block_data)
    }

    fn set_invalid_blocks(&self, invalid_blocks: HashMap<BlockHash, Validator>) -> () {
        self.runtime.set_invalid_blocks(invalid_blocks)
    }

    fn get_hot_changes(
        &self,
    ) -> HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>> {
        self.runtime.get_hot_changes()
    }
}

// TODO: Where is this implementation for the scala code?
impl HasCost for ReplayRhoRuntimeImpl {
    fn cost(&self) -> &CostState {
        todo!()
    }
}

pub type RhoTuplespace =
    Arc<Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>>;
pub type RhoISpace = RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>;
pub type RhoReplayISpace = RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>;
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
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.std_out(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stdoutAck".to_string(),
            fixed_channel: FixedChannels::stdout_ack(),
            arity: 2,
            body_ref: BodyRefs::STDOUT_ACK,
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.std_out_ack(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stderr".to_string(),
            fixed_channel: FixedChannels::stderr(),
            arity: 1,
            body_ref: BodyRefs::STDERR,
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.std_err(args))),
            remainder: None,
        },
        Definition {
            urn: "rho:io:stderrAck".to_string(),
            fixed_channel: FixedChannels::stderr_ack(),
            arity: 2,
            body_ref: BodyRefs::STDERR_ACK,
            handler: Box::new(|ctx| Box::new(move |args| ctx.system_processes.std_err_ack(args))),
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
    dispatcher: Arc<Mutex<RholangAndRustDispatcher>>,
    block_data: Arc<RwLock<BlockData>>,
    invalid_blocks: InvalidBlocks,
    extra_system_processes: Vec<Definition>,
) -> RhoDispatchMap {
    let mut dispatch_table = HashMap::new();

    for def in std_system_processes().iter().chain(
        std_rho_crypto_processes()
            .iter()
            .chain(extra_system_processes.iter()),
    ) {
        let tuple = def.to_dispatch_table(ProcessContext {
            space: space.clone(),
            dispatcher: dispatcher.clone(),
            block_data: block_data.clone(),
            invalid_blocks: invalid_blocks.clone(),
            system_processes: SystemProcesses {
                dispatcher: dispatcher.clone(),
                space: space.clone(),
            },
        });

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

// TODO: Review this code and make sure it follows the same logic as Scala function.
//       "lazy val" and circular dependency
fn setup_reducer(
    charging_rspace: RhoTuplespace,
    block_data_ref: Arc<RwLock<BlockData>>,
    invalid_blocks: InvalidBlocks,
    extra_system_processes: Vec<Definition>,
    urn_map: HashMap<String, Par>,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
    mergeable_tag_name: Par,
) -> DebruijnInterpreter {
    let (replay_dispatcher, replay_reducer) = RholangAndRustDispatcher::create(
        charging_rspace.clone(),
        HashMap::new(), // this should be replay_dispatch_table
        urn_map,
        merge_chs,
        mergeable_tag_name,
    );

    let replay_dispatch_table = dispatch_table_creator(
        charging_rspace,
        Arc::new(Mutex::new(replay_dispatcher)),
        block_data_ref,
        invalid_blocks,
        extra_system_processes,
    );

    replay_reducer
}

fn setup_maps_and_refs(
    extra_system_processes: Vec<Definition>,
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
