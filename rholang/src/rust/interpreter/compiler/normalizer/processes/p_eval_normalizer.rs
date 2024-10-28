use std::error::Error;
use models::rhoapi::{Par};
use tree_sitter::Node;
use crate::rust::interpreter::compiler::normalize::{NameVisitInputs, normalize_match, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::errors::InterpreterError;
use super::exports::*;

pub fn normalize_p_eval(
  node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing eval node of kind: {}", node.kind());

  let name_node = node.named_child(0).ok_or_else(|| {
    InterpreterError::SyntaxError("Expected name in eval expression".to_string())
  })?;

  let name_match_result = normalize_name(
    name_node,
    NameVisitInputs {
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    source_code,
  )?;

  let updated_par = input.par.append(name_match_result.par.clone());

  Ok(ProcVisitOutputs {
    par: updated_par,
    free_map: name_match_result.free_map,
  })
}


#[test]
fn test_normalize_p_eval() {
  let rholang_code = r#"
        *x
    "#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let eval_node = root_node.child(0).expect("Expected an eval node");
  println!("Found eval node: {}", eval_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(eval_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");
      println!("Resulting Par: {:?}", result.par);
      assert_eq!(result.par.exprs.len(), 1, "Expected one expression in the resulting Par");
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}


