// See rspace/src/main/scala/coop/rchain/rspace/ISpace.scala

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::rspace::checkpoint::SoftCheckpoint;

use super::{
    checkpoint::Checkpoint,
    errors::RSpaceError,
    hashing::blake2b256_hash::Blake2b256Hash,
    internal::{Datum, ProduceCandidate, Row, WaitingContinuation},
    tuplespace_interface::Tuplespace,
};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct RSpaceResult<C, A> {
    pub channel: C,
    pub matched_datum: A,
    pub removed_datum: A,
    pub persistent: bool,
}

// NOTE: On Scala side, they are defaulting "peek" to false
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct ContResult<C, P, K> {
    pub continuation: K,
    pub persistent: bool,
    pub channels: Vec<C>,
    pub patterns: Vec<P>,
    pub peek: bool,
}

pub type MaybeProduceCandidate<C, P, A, K> = Option<ProduceCandidate<C, P, A, K>>;
pub type MaybeActionResult<C, P, A, K> = Option<(ContResult<C, P, K>, Vec<RSpaceResult<C, A>>)>;

pub const CONSUME_COMM_LABEL: &str = "comm.consume";
pub const PRODUCE_COMM_LABEL: &str = "comm.produce";

/** The interface for RSpace
 *
 * @tparam C a type representing a channel
 * @tparam P a type representing a pattern
 * @tparam A a type representing an arbitrary piece of data and match result
 * @tparam K a type representing a continuation
 */
pub trait ISpace<C: Eq + std::hash::Hash, P: Clone, A: Clone, K: Clone>:
    Tuplespace<C, P, A, K>
{
    /** Creates a checkpoint.
     *
     * @return A [[Checkpoint]]
     */
    fn create_checkpoint(&mut self) -> Result<Checkpoint, RSpaceError>;

    fn get_data(&self, channel: C) -> Vec<Datum<A>>;

    fn get_waiting_continuations(&self, channels: Vec<C>) -> Vec<WaitingContinuation<P, K>>;

    fn get_joins(&self, channel: C) -> Vec<Vec<C>>;

    /** Clears the store.  Does not affect the history trie.
     */
    fn clear(&mut self) -> Result<(), RSpaceError>;

    /** Resets the store to the given root.
     *
     * @param root A BLAKE2b256 Hash representing the checkpoint
     */
    fn reset(&mut self, root: Blake2b256Hash) -> Result<(), RSpaceError>;

    // TODO: this should not be exposed - OLD
    fn to_map(&self) -> HashMap<Vec<C>, Row<P, A, K>>;

    /**
    Allows to create a "soft" checkpoint which doesn't persist the checkpointed data into history.
    This operation is significantly faster than {@link #createCheckpoint()} because the computationally
    expensive operation of creating the history trie is avoided.
    */
    fn create_soft_checkpoint(&mut self) -> SoftCheckpoint<C, P, A, K>;

    /**
    Reverts the ISpace to the state checkpointed using {@link #createSoftCheckpoint()}
    */
    fn revert_to_soft_checkpoint(
        &mut self,
        checkpoint: SoftCheckpoint<C, P, A, K>,
    ) -> Result<(), RSpaceError>;
}
