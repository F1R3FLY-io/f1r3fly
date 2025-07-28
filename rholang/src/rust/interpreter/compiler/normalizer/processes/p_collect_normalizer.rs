use crate::rust::interpreter::compiler::normalize::{
    CollectVisitInputs, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::compiler::normalizer::collection_normalize_matcher::normalize_collection;
use crate::rust::interpreter::compiler::rholang_ast::Collection;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;
use f1r3fly_models::rhoapi::Par;
use std::collections::HashMap;

pub fn normalize_p_collect(
    proc: &Collection,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let collection_result = normalize_collection(
        proc,
        CollectVisitInputs {
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: input.free_map.clone(),
        },
        env,
    )?;

    let updated_par = prepend_expr(
        input.par,
        collection_result.expr,
        input.bound_map_chain.depth() as i32,
    );

    Ok(ProcVisitOutputs {
        par: updated_par,
        free_map: collection_result.free_map,
    })
}
