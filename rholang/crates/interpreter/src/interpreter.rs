use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::Par;
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use crate::accounting::CostManager;
use crate::aliases::EnvHashMap;
use crate::env::Env;
use crate::normal_forms;

use super::accounting::costs::{Cost, parsing_cost};
use super::compiler::compiler::Compiler;
use super::errors::InterpreterError;
use super::reduce::DebruijnInterpreter;

//See rholang/src/main/scala/coop/rchain/rholang/interpreter/Interpreter.scala
#[derive(Clone, Debug)]
pub struct EvaluateResult {
    pub cost: Cost,
    pub errors: Vec<InterpreterError>,
    pub mergeable: HashSet<normal_forms::Par>,
}

pub struct Interpreter {
    cost_manager: CostManager,
    merge_chs: Arc<RwLock<HashSet<normal_forms::Par>>>,
}

impl Interpreter {
    pub async fn inject_attempt(
        &self,
        reducer: &DebruijnInterpreter,
        term: String,
        initial_cost: Cost,
        normalizer_env: EnvHashMap,
        rand: Blake2b512Random,
    ) -> Result<EvaluateResult, InterpreterError> {
        let cost = parsing_cost(&term);

        self.cost_manager.set(initial_cost.clone());
        self.cost_manager.charge(cost.clone())?;

        // Empty mergeable channels
        {
            let mut merge_chs_lock = self.merge_chs.write().expect("Can't lock to write");
            merge_chs_lock.clear();
        }

        let parsed = Compiler::new(&term).compile_to_adt()?;
        reducer.eval(parsed, &Env::new(), rand).await?;

        let phlos_left = self.cost_manager.get();
        let mergeable = *self
            .merge_chs
            .read()
            .expect("Can't read from merge channels");

        Ok(EvaluateResult {
            cost: initial_cost - phlos_left,
            errors: Vec::new(),
            mergeable,
        })
    }

    pub fn new(
        cost: CostManager,
        merge_chs: Arc<RwLock<HashSet<normal_forms::Par>>>,
    ) -> Interpreter {
        Interpreter {
            cost_manager: cost,
            merge_chs,
        }
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
            InterpreterError::ParserError(_, _, _) => Ok(EvaluateResult {
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
