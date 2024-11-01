use std::collections::HashMap;
use models::rhoapi::{Connective, Par};
use models::rhoapi::connective::ConnectiveInstance;
use crate::rust::interpreter::compiler::normalize::{ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::rholang_ast::SimpleType;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;

pub fn normalize_simple_type(
  simple_type: &SimpleType,
  input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
  let connective_instance = match simple_type {
    SimpleType::Bool { .. } => ConnectiveInstance::ConnBool(true),
    SimpleType::Int { .. } => ConnectiveInstance::ConnInt(true),
    SimpleType::String { .. } => ConnectiveInstance::ConnString(true),
    SimpleType::Uri { .. } => ConnectiveInstance::ConnUri(true),
    SimpleType::ByteArray { .. } => ConnectiveInstance::ConnByteArray(true),
  };

  let connective = Connective {
    connective_instance: Some(connective_instance),
  };

  Ok(ProcVisitOutputs {
    par: {
      let mut updated_par = prepend_connective(
        input.par.clone(),
        connective,
        input.bound_map_chain.depth() as i32, );
      updated_par.connective_used = true;
      updated_par
    },
    free_map: input.free_map,
  })
}

mod tests {
  use super::*;
  use crate::rust::interpreter::compiler::rholang_ast::{Proc, SimpleType};
  use models::rhoapi::Par;
  use crate::rust::interpreter::compiler::exports::BoundMapChain;
  use crate::rust::interpreter::compiler::normalize::normalize_match_proc;

  #[test]
  fn test_normalize_match_proc_with_simple_type_bool() {
    let proc = Proc::SimpleType(SimpleType::Bool { line_num: 1, col_num: 1 });
    let input = ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: BoundMapChain::default(),
      free_map: Default::default(),
    };

    let result = normalize_match_proc(&proc, input, &HashMap::new());
    assert!(result.is_ok());
    let output = result.unwrap();
    println!("normalized output: {:?}", output);
    assert!(output.par.connectives.len() == 1);
    if let Some(ConnectiveInstance::ConnBool(value)) = &output.par.connectives[0].connective_instance {
      assert!(*value);
    } else {
      panic!("Expected ConnBool connective");
    }
  }
}