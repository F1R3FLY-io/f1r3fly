use crate::rust::interpreter::compiler::normalize::{ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::rholang_ast::{Case, Proc};
use crate::rust::interpreter::errors::InterpreterError;

pub fn normalize_p_match(
  expressions: &Box<Proc>,
  cases: &Vec<Case>,
  input: ProcVisitInputs,
  line_num: usize,
  column_num: usize,
) -> Result<ProcVisitOutputs, InterpreterError> {
  todo!("normalize_p_match")
}