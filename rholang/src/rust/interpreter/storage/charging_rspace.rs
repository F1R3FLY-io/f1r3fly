// See rholang/src/main/scala/coop/rchain/rholang/interpreter/storage/ChargingRSpace.scala

use crate::rust::interpreter::{
    errors::InterpreterError, rho_runtime::RhoTuplespace, unwrap_option_safe,
};
use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{tagged_continuation::TaggedCont, TaggedContinuation};

pub struct ChargingRSpace;

pub enum TriggeredBy {
    Consume {
        id: Blake2b512Random,
        persistent: bool,
        channels_count: usize,
    },
    Produce {
        id: Blake2b512Random,
        persistent: bool,
    },
}

fn consume_id(continuation: TaggedContinuation) -> Result<Blake2b512Random, InterpreterError> {
    //TODO: Make ScalaBodyRef-s have their own random state and merge it during its COMMs - OLD
    match unwrap_option_safe(continuation.tagged_cont)? {
        TaggedCont::ParBody(par_with_random) => {
            Ok(Blake2b512Random::new(&par_with_random.random_state))
        }
        TaggedCont::ScalaBodyRef(value) => Ok(Blake2b512Random::new(&value.to_be_bytes())),
    }
}

impl ChargingRSpace {
    pub fn charging_rspace(space: RhoTuplespace) -> RhoTuplespace {
        
    }
}
