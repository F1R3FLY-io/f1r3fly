use models::rhoapi::Par;
use std::collections::HashSet;

use super::accounting::costs::Cost;
use super::errors::InterpreterError;

pub struct EvaluateResult {
    cost: Cost,
    errors: Vec<InterpreterError>,
    mergeable: HashSet<Par>,
}
