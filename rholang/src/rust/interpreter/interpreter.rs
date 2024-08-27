use models::rhoapi::Par;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::accounting::_cost;
use super::accounting::costs::Cost;
use super::errors::InterpreterError;
use super::reduce::DebruijnInterpreter;

//See rholang/src/main/scala/coop/rchain/rholang/interpreter/Interpreter.scala
pub struct EvaluateResult {
    cost: Cost,
    errors: Vec<InterpreterError>,
    mergeable: HashSet<Par>,
}

pub trait Interpreter {
    fn inj_attempt(
        &self,
        reducer: &DebruijnInterpreter,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b256Hash,
    ) -> EvaluateResult;
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
        rand: Blake2b256Hash,
    ) -> EvaluateResult {
        todo!()
    }
}

impl InterpreterImpl {
    pub fn new(cost: _cost, merge_chs: Arc<RwLock<HashSet<Par>>>) -> InterpreterImpl {
        InterpreterImpl { c: cost, merge_chs }
    }
}
