use crate::compiler::exports::FreeMap;
use crate::errors::InterpreterError;
use crate::normal_forms::Var as NormalizedVar;

use crate::compiler::rholang_ast::{NameRemainder, ProcRemainder, Var};

pub(crate) fn handle_proc_remainder(
    r: ProcRemainder,
    known_free: &mut FreeMap,
) -> Result<NormalizedVar, InterpreterError> {
    match r.var {
        Var::Wildcard => {
            known_free.add_wildcard(r.pos);
            Ok(NormalizedVar::Wildcard)
        }
        Var::Id(id) => known_free
            .put_id_in_proc_context(&id)
            .map(NormalizedVar::FreeVar)
            .map_err(
                |old_context| InterpreterError::UnexpectedReuseOfProcContextFree {
                    var_name: id.name.to_string(),
                    first_use: old_context.source_position,
                    second_use: id.pos,
                },
            ),
    }
}

#[inline(always)]
pub(crate) fn handle_name_remainder(
    r: NameRemainder,
    known_free: &mut FreeMap,
) -> Result<NormalizedVar, InterpreterError> {
    handle_proc_remainder(r, known_free)
}
