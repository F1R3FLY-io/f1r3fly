use std::error::Error;
use models::rhoapi::{Expr, expr, Par};
use tree_sitter::Node;
use super::exports::parse_rholang_code;
use super::exports::{normalize_ground, ProcVisitOutputs, ProcVisitInputs};
use crate::rust::interpreter::compiler::normalize::prepend_expr;
use crate::rust::interpreter::compiler::normalize::ground_to_expr;

pub fn normalize_p_ground(
  p_node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  if let Some(ground) = normalize_ground(p_node, source_code) {
    let expr = ground_to_expr(ground);
    let new_par = prepend_expr(input.par.clone(), expr, input.bound_map_chain.depth() as i32);
    Ok(ProcVisitOutputs {
      par: new_par,
      free_map: input.free_map.clone(),
    })
  } else {
    Err("Failed to normalize ground value".into())
  }
}



#[test]
fn test_normalize_pground_int() {
  let rholang_code = "42";
  let tree = parse_rholang_code(rholang_code);
  let root = tree.root_node();

  println!("Parsed tree: {:?}", root.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  let literal_node = root
    .child(0)
    .expect("Expected a long_literal node");

  let output = normalize_p_ground(literal_node, input.clone(), rholang_code.as_bytes());

  assert!(output.is_ok(), "Expected Ok(ProcVisitOutputs) but got Err");

  let output = output.unwrap();

  if let Some(expr::ExprInstance::GInt(value)) = output.par.exprs.get(0).and_then(|e| e.expr_instance.as_ref()) {
    assert_eq!(*value, 42);
  } else {
    panic!("Expected GInt(42) but got something else");
  }
}


#[test]
fn test_normalize_pground_string() {
  let rholang_code = "\"Hello, Rholang!\"";
  let tree = parse_rholang_code(rholang_code);
  let root = tree.root_node();

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  let literal_node = root
    .child(0)
    .expect("Expected a string_literal node");

  let output = normalize_p_ground(literal_node, input.clone(), rholang_code.as_bytes());

  assert!(output.is_ok(), "Expected Ok(ProcVisitOutputs) but got Err");

  let output = output.unwrap();

  if let Some(expr::ExprInstance::GString(value)) = output.par.exprs.get(0).and_then(|e| e.expr_instance.as_ref()) {
    assert_eq!(value, "Hello, Rholang!");
  } else {
    panic!("Expected GString(\"Hello, Rholang!\") but got something else");
  }
}


#[test]
fn test_normalize_pground_bool() {
  let rholang_code = "true";
  let tree = parse_rholang_code(rholang_code);
  let root = tree.root_node();

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  let literal_node = root
    .child(0)
    .expect("Expected a bool_literal node");

  let output = normalize_p_ground(literal_node, input.clone(), rholang_code.as_bytes());

  assert!(output.is_ok(), "Expected Ok(ProcVisitOutputs) but got Err");

  let output = output.unwrap();

  if let Some(expr::ExprInstance::GBool(value)) = output.par.exprs.get(0).and_then(|e| e.expr_instance.as_ref()) {
    assert_eq!(*value, true);
  } else {
    panic!("Expected GBool(true) but got something else");
  }
}

#[test]
fn test_normalize_pground_uri() {
  let rholang_code = "`http://example.com`";
  let tree = parse_rholang_code(rholang_code);
  let root = tree.root_node();

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  let literal_node = root
    .child(0)
    .expect("Expected a uri_literal node");

  let output = normalize_p_ground(literal_node, input.clone(), rholang_code.as_bytes());

  assert!(output.is_ok(), "Expected Ok(ProcVisitOutputs) but got Err");

  let output = output.unwrap();

  if let Some(expr::ExprInstance::GUri(value)) = output.par.exprs.get(0).and_then(|e| e.expr_instance.as_ref()) {
    assert_eq!(value, "http://example.com");
  } else {
    panic!("Expected GUri(\"http://example.com\") but got something else");
  }
}




