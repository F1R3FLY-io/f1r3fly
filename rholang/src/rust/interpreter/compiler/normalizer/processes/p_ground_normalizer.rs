use crate::rust::interpreter::util::prepend_expr;

use super::exports::*;

pub fn normalize_p_ground(
    proc: &Proc,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    normalize_ground(proc).map(|expr| {
        let new_par = prepend_expr(
            input.par.clone(),
            expr,
            input.bound_map_chain.depth() as i32,
        );
        ProcVisitOutputs {
            par: new_par,
            free_map: input.free_map.clone(),
        }
    })
}
