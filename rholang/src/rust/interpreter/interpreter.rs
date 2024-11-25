use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::Par;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::accounting::_cost;
use super::accounting::costs::{parsing_cost, Cost};
use super::compiler::compiler::Compiler;
use super::errors::InterpreterError;
use super::reduce::DebruijnInterpreter;

//See rholang/src/main/scala/coop/rchain/rholang/interpreter/Interpreter.scala
#[derive(Clone, Debug)]
pub struct EvaluateResult {
    pub cost: Cost,
    pub errors: Vec<InterpreterError>,
    pub mergeable: HashSet<Par>,
}

pub trait Interpreter {
    async fn inj_attempt(
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
    async fn inj_attempt(
        &self,
        reducer: &DebruijnInterpreter,
        term: String,
        initial_phlo: Cost,
        normalizer_env: HashMap<String, Par>,
        rand: Blake2b512Random,
    ) -> Result<EvaluateResult, InterpreterError> {
        // println!("\nhit inj_attempt");
        let parsing_cost = parsing_cost(&term);

        let evaluation_result: Result<EvaluateResult, InterpreterError> = {
            let _ = self.c.set(initial_phlo.clone());
            // let phlos_left_before = self.c.get();
            let _ = self.c.charge(parsing_cost.clone())?;
            // let phlos_left_after = self.c.get();

            // println!("\nterm: {:#?}", term);
            let parsed = Compiler::source_to_adt_with_normalizer_env(&term, normalizer_env)?;
            // println!("\nparsed: {:?}", parsed);
            // let phlos_left_after_adt = self.c.get();

            // Empty mergeable channels
            let mut merge_chs_lock = self.merge_chs.write().unwrap();
            merge_chs_lock.clear();
            drop(merge_chs_lock);

            reducer.inj(parsed, rand).await?;
            let phlos_left = self.c.get();
            let mergeable_channels = self.merge_chs.read().unwrap().clone();

            Ok(EvaluateResult {
                cost: initial_phlo.clone() - phlos_left,
                errors: Vec::new(),
                mergeable: mergeable_channels,
            })
        };

        match evaluation_result {
            Ok(eval_result) => Ok(eval_result),
            Err(err) => self.handle_error(initial_phlo, parsing_cost, err),
        }
    }
}

impl InterpreterImpl {
    pub fn new(cost: _cost, merge_chs: Arc<RwLock<HashSet<Par>>>) -> InterpreterImpl {
        InterpreterImpl { c: cost, merge_chs }
    }

    // TODO: Implement and handle just 'InterpreterError'
    fn handle_error(
        &self,
        initial_cost: Cost,
        parsing_cost: Cost,
        error: InterpreterError,
    ) -> Result<EvaluateResult, InterpreterError> {
        // println!("\nhit handle_error");
        match error {
            // Parsing error consumes only parsing cost
            InterpreterError::ParserError(_) => Ok(EvaluateResult {
                cost: parsing_cost,
                errors: vec![error],
                mergeable: HashSet::new(),
            }),

            // For Out Of Phlogistons error initial cost is used because evaluated cost can be higher
            // - all phlos are consumed
            InterpreterError::OutOfPhlogistonsError => Ok(EvaluateResult {
                cost: initial_cost,
                errors: vec![error],
                mergeable: HashSet::new(),
            }),

            // InterpreterError(s) - multiple errors are result of parallel execution
            InterpreterError::AggregateError { interpreter_errors } => Ok(EvaluateResult {
                cost: initial_cost,
                errors: interpreter_errors,
                mergeable: HashSet::new(),
            }),

            _ => panic!("Fatal Interpreter error: {:?}", error),
        }
    }
}
