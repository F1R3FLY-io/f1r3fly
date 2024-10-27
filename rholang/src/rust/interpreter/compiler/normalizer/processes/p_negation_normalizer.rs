use std::error::Error;
use models::rhoapi::{Par, Connective, connective};
use tree_sitter::Node;
use crate::rust::interpreter::compiler::normalize::{normalize_match, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;
use super::exports::*;

pub fn normalize_p_negation(
  node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing negation node of kind: {}", node.kind());

  let proc_node = node.child_by_field_name("proc").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected proc in negation expression".to_string())
  })?;

  let body_result = normalize_match(
    proc_node,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: FreeMap::default(),
    },
    source_code,
  )?;

  // Create Connective with ConnNotBody
  let connective = Connective {
    connective_instance: Some(connective::ConnectiveInstance::ConnNotBody(body_result.par.clone())),
  };

  let updated_par = prepend_connective(input.par, connective.clone(), input.bound_map_chain.clone().depth() as i32);

  Ok(ProcVisitOutputs {
    par: updated_par,
    free_map: input.free_map.add_connective(connective.connective_instance.unwrap(), SourcePosition {
      row: node.start_position().row as usize,
      column: node.start_position().column as usize,
    }),
  })
}

#[test]
fn test_normalize_p_negation() {
  let rholang_code = r#"
        ~x
    "#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let negation_node = root_node.named_child(0).expect("Expected a negation node");
  println!("Found negation node: {}", negation_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(negation_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");
      println!("Resulting Par: {:?}", result.par);
      assert_eq!(result.par.connectives.len(), 1, "Expected one connective in the resulting Par");
      if let Some(connective::ConnectiveInstance::ConnNotBody(body)) = &result.par.connectives[0].connective_instance {
        assert!(body.exprs.len() > 0, "Expected body of negation to contain an expression");
      } else {
        panic!("Expected connective to be ConnNotBody");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}
