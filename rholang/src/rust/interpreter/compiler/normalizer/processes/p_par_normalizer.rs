use std::error::Error;
use models::rhoapi::{expr, Par, var};
use tree_sitter::Node;
use crate::rust::interpreter::compiler::normalize::{normalize_match, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::errors::InterpreterError;
use super::exports::*;

pub fn normalize_p_par(
  node: Node,
  mut input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing par node of kind: {}", node.kind());

  let left_node = node.child_by_field_name("left").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected left operand in par".to_string())
  })?;

  let result = normalize_match(
    left_node,
    input.clone(),
    source_code,
  )?;

  let chained_input = ProcVisitInputs {
    par: result.par.clone(),
    free_map: result.free_map.clone(),
    ..input.clone()
  };

  let right_node = node.child_by_field_name("right").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected right operand in par".to_string())
  })?;

  let chained_res = normalize_match(
    right_node,
    chained_input,
    source_code,
  )?;

  Ok(chained_res)
}

#[test]
fn test_normalize_p_par() {
  let rholang_code = r#"
        x | y
    "#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let par_node = root_node.named_child(0).expect("Expected a par node");
  println!("Found par node: {}", par_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(par_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");
      assert_eq!(result.par.exprs.len(), 2, "Expected two expressions in the resulting Par");

      let mut free_vars = vec![];
      for expr in &result.par.exprs {
        if let Some(expr::ExprInstance::EVarBody(evar)) = &expr.expr_instance {
          if let Some(var::VarInstance::FreeVar(idx)) = evar.v.as_ref().unwrap().var_instance {
            free_vars.push(idx);
          }
        }
      }
      assert!(free_vars.contains(&0), "Expected first variable to be a free variable");
      assert!(free_vars.contains(&1), "Expected second variable to be a free variable");
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}
