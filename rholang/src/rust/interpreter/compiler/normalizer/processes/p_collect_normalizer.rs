use crate::rust::interpreter::compiler::normalize::{CollectVisitInputs, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::normalizer::collection_normalize_matcher::normalize_collection;
use crate::rust::interpreter::compiler::rholang_ast::{Collection};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;

pub fn normalize_p_collect(
  proc: &Collection,
  input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
  let collection_result = normalize_collection(
    proc,
    CollectVisitInputs {
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    })?;

  let updated_par = prepend_expr(input.par, collection_result.expr, input.bound_map_chain.depth() as i32);

  Ok(ProcVisitOutputs {
    par: updated_par,
    free_map: collection_result.free_map,
  })
}