use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::Par;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::accounting::_cost;
use super::accounting::costs::Cost;
use super::errors::InterpreterError;
use super::reduce::DebruijnInterpreter;

//See rholang/src/main/scala/coop/rchain/rholang/interpreter/Interpreter.scala
#[derive(Clone)]
pub struct EvaluateResult {
    pub cost: Cost,
    pub errors: Vec<InterpreterError>,
    pub mergeable: HashSet<Par>,
}

pub trait Interpreter {
    fn inj_attempt(
        &self,
        reducer: &DebruijnInterpreter,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b512Random,
    ) -> Result<EvaluateResult, InterpreterError>;
}

pub struct InterpreterImpl {
    c: _cost,
    merge_chs: Arc<RwLock<HashSet<Par>>>,
}

impl Interpreter for InterpreterImpl {
    fn inj_attempt(
        &self,
        reducer: &DebruijnInterpreter,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b512Random,
    ) -> Result<EvaluateResult, InterpreterError> {
        todo!()
    }
}

impl InterpreterImpl {
    pub fn new(cost: _cost, merge_chs: Arc<RwLock<HashSet<Par>>>) -> InterpreterImpl {
        InterpreterImpl { c: cost, merge_chs }
    }
}
