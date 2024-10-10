use std::error::Error;
use tree_sitter::Node;
use crate::rust::interpreter::compiler::normalize::{CollectVisitInputs, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::normalizer::collection_normalize_matcher::normalize_collection;
use crate::rust::interpreter::util::prepend_expr;

pub fn normalize_p_collect(
  node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  let collection_result = normalize_collection(
    node,
    CollectVisitInputs {
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    source_code,
  )?;

  let updated_par = prepend_expr(input.par, collection_result.expr, input.bound_map_chain.depth() as i32);

  Ok(ProcVisitOutputs {
    par: updated_par,
    free_map: collection_result.free_map,
  })
}