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

pub fn normalize_p_var(
    node: Node,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    println!("Normalizing p_var_ref node of kind: {}", node.kind());
    let proc_var = unwrap_option_safe(node.child_by_field_name("proc_var"))?.kind();

    match proc_var {
        "proc_var_var" => {
            todo!()
        }

        "proc_var_wildcard" => {
            todo!()
        }

        _ => Err(InterpreterError::BugFoundError(
            "Unmatched proc_var".to_string(),
        )),
    }
}
