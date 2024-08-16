use models::rhoapi::Par;
use std::collections::HashSet;

use super::accounting::costs::Cost;
use super::errors::InterpreterError;

//See rholang/src/main/scala/coop/rchain/rholang/interpreter/Interpreter.scala
pub struct EvaluateResult {
    cost: Cost,
    errors: Vec<InterpreterError>,
    mergeable: HashSet<Par>,
}
