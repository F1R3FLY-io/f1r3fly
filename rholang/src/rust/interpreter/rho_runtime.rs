use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use models::rust::block_hash::BlockHash;
use models::rust::validator::Validator;
use rspace_plus_plus::rspace::checkpoint::{Checkpoint, SoftCheckpoint};
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::internal::{Datum, Row, WaitingContinuation};
use rspace_plus_plus::rspace::trace::Log;
use std::collections::HashMap;

use super::accounting::costs::Cost;
use super::accounting::has_cost::HasCost;
use super::env::Env;
use super::interpreter::EvaluateResult;
use super::system_processes::BlockData;

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
    fn inj(&self, par: Par, env: Env<Par>) -> ();

    /**
     * After some executions([[evaluate]]) on the runtime, you can create a soft checkpoint which is the changes
     * for the current state of the runtime. You can revert the changes by [[revertToSoftCheckpoint]]
     * @return
     */
    fn create_soft_checkpoint(
        &self,
    ) -> SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>;

    fn revert_to_soft_checkpoint(
        &self,
        soft_checkpoint: SoftCheckpoint<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    ) -> ();

    /**
     * Create a checkpoint for the runtime. All the changes which happened in the runtime would persistent in the disk
     * and result in a new stateHash for the new state.
     * @return
     */
    fn create_checkpoint(&self) -> Checkpoint;

    /**
     * Reset the runtime to the specific state. Then you can operate some execution on the state.
     * @param root the target state hash to reset
     * @return
     */
    fn reset(&self, root: Blake2b256Hash) -> ();

    /**
     * Consume the result in the rspace.
     *
     * This function would change the state in the runtime.
     * @param channel target channel for the consume
     * @param pattern pattern for the consume
     * @return
     */
    fn consume_result(
        &self,
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
