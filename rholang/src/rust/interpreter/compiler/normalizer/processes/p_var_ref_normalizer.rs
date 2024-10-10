use crate::rust::interpreter::compiler::exports::BoundContext;
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::compiler::rholang_ast::{PVarRef, VarRefKind};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;

use super::exports::*;
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::{Connective, VarRef};
use std::result::Result;

pub fn normalize_p_var_ref(
    p: PVarRef,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    match input.bound_map_chain.find(&p.var) {
        Some((
            BoundContext {
                index,
                typ,
                source_position,
            },
            depth,
        )) => match typ {
            VarSort::ProcSort => match p.var_ref_kind {
                VarRefKind::Proc => Ok(ProcVisitOutputs {
                    par: prepend_connective(
                        input.par,
                        Connective {
                            connective_instance: Some(ConnectiveInstance::VarRefBody(VarRef {
                                index: index as i32,
                                depth: depth as i32,
                            })),
                        },
                        input.bound_map_chain.depth() as i32,
                    ),
                    free_map: input.free_map,
                }),

                _ => Err(InterpreterError::UnexpectedProcContext {
                    var_name: p.var,
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: p.line_num,
                        column: p.col_num,
                    },
                }),
            },
            VarSort::NameSort => match p.var_ref_kind {
                VarRefKind::Name => Ok(ProcVisitOutputs {
                    par: prepend_connective(
                        input.par,
                        Connective {
                            connective_instance: Some(ConnectiveInstance::VarRefBody(VarRef {
                                index: index as i32,
                                depth: depth as i32,
                            })),
                        },
                        input.bound_map_chain.depth() as i32,
                    ),
                    free_map: input.free_map,
                }),

                _ => Err(InterpreterError::UnexpectedProcContext {
                    var_name: p.var,
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: p.line_num,
                        column: p.col_num,
                    },
                }),
            },
        },

        None => Err(InterpreterError::UnboundVariableRef {
            var_name: p.var,
            line: p.line_num,
            col: p.col_num,
        }),
    }
}
