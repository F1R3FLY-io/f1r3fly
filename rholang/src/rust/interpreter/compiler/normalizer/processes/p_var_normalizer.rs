use models::rust::utils::{new_boundvar_expr, new_freevar_expr, new_wildcard_expr};

use crate::rust::interpreter::compiler::exports::{BoundContext, FreeContext};
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::compiler::rholang_ast::PVar;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;

use super::exports::*;
use std::result::Result;

pub fn normalize_p_var(
    p: PVar,
    mut input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    match p {
        PVar::ProcVarVar {
            name,
            line_num,
            col_num,
        } => match input.bound_map_chain.get(&name) {
            Some(BoundContext {
                index,
                typ,
                source_position,
            }) => match typ {
                VarSort::ProcSort => Ok(ProcVisitOutputs {
                    par: prepend_expr(
                        input.par,
                        new_boundvar_expr(index as i32),
                        input.bound_map_chain.depth() as i32,
                    ),
                    free_map: input.free_map,
                }),
                VarSort::NameSort => Err(InterpreterError::UnexpectedProcContext {
                    var_name: name,
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: line_num,
                        column: col_num,
                    },
                }),
            },

            None => match input.free_map.get(&name) {
                Some(FreeContext {
                    source_position, ..
                }) => Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                    var_name: name,
                    first_use: source_position,
                    second_use: SourcePosition {
                        row: line_num,
                        column: col_num,
                    },
                }),

                None => {
                    let new_bindings_pair = input.free_map.put((
                        name,
                        VarSort::ProcSort,
                        SourcePosition {
                            row: line_num,
                            column: col_num,
                        },
                    ));

                    Ok(ProcVisitOutputs {
                        par: prepend_expr(
                            input.par,
                            new_freevar_expr(input.free_map.next_level as i32),
                            input.bound_map_chain.depth() as i32,
                        ),
                        free_map: new_bindings_pair,
                    })
                }
            },
        },

        PVar::ProcVarWildcard { line_num, col_num } => Ok(ProcVisitOutputs {
            par: {
                let mut par = prepend_expr(
                    input.par,
                    new_wildcard_expr(),
                    input.bound_map_chain.depth() as i32,
                );
                par.connective_used = true;
                par
            },
            free_map: input.free_map.add_wildcard(SourcePosition {
                row: line_num,
                column: col_num,
            }),
        }),
    }
}
