use crate::rust::interpreter::compiler::exports::BoundContext;
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::prepend_connective;
use crate::rust::interpreter::unwrap_option_safe;

use super::exports::*;
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::{Connective, VarRef};
use std::result::Result;
use tree_sitter::Node;

/*
  Nazar:
    - Is VarRef in grammar.js?
    - How do I translate Scala types to Rust tree_sitter string types?
    - What is p.line_num and p.col_num in Rust?
    - Start using InterpreterError elsewhere?
    - What is source code?
*/
pub fn normalize_p_var_ref(
    node: Node,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    println!("Normalizing p_var_ref node of kind: {}", node.kind());

    let p_var = unwrap_option_safe(node.child_by_field_name("var"))?.kind();
    let p_var_ref_kind = unwrap_option_safe(node.child_by_field_name("var_ref_kind"))?.kind();
    let p_line_num: usize = node.start_position().row;
    let p_col_num: usize = node.start_position().column;

    match input.bound_map_chain.find(p_var) {
        Some((
            BoundContext {
                index,
                typ,
                source_position,
            },
            depth,
        )) => match typ {
            VarSort::ProcSort => match p_var_ref_kind {
                "var_ref_kind_proc" => Ok(ProcVisitOutputs {
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
                    var_name: p_var.to_string(),
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: p_line_num,
                        column: p_col_num,
                    },
                }),
            },
            VarSort::NameSort => match p_var_ref_kind {
                "var_ref_kind_name" => Ok(ProcVisitOutputs {
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
                    var_name: p_var.to_string(),
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: p_line_num,
                        column: p_col_num,
                    },
                }),
            },
        },

        None => Err(InterpreterError::UnboundVariableRef {
            var_name: p_var.to_string(),
            line: 0, // TODO: Figure out eq for tree_sitter
            col: 0,  // TODO: Figure out eq for tree_sitter
        }),
    }
}
