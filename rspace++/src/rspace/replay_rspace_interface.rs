// See rspace/src/main/scala/coop/rchain/rspace/IReplaySpace.scala

use std::any::Any;

use super::{
    errors::RSpaceError, hashing::blake2b256_hash::Blake2b256Hash, rspace_interface::ISpace,
    trace::Log,
};

pub trait IReplayRSpace<C, P, A, K>: ISpace<C, P, A, K> + Any
where
    C: Eq + std::hash::Hash,
    P: Clone,
    A: Clone,
    K: Clone,
{
    fn rig_and_reset(&mut self, start_root: Blake2b256Hash, log: Log) -> Result<(), RSpaceError>;

    fn rig(&self, log: Log) -> Result<(), RSpaceError>;

    fn check_replay_data(&self) -> Result<(), RSpaceError>;
}
