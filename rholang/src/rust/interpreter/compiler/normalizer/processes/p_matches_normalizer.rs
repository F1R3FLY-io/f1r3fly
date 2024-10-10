use crate::rust::interpreter::{compiler::{normalize::normalize_match, normalizer::parser::parse_rholang_code}, util::prepend_expr};
use super::exports::{ProcVisitOutputs, ProcVisitInputs, FreeMap};
use models::rhoapi::{EMatches, Expr, expr, Par};
use std::error::Error;
use tree_sitter::Node;

pub fn normalize_p_matches(
  p_node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing p_matches node of kind: {}", p_node.kind());

  let left_node = p_node.child_by_field_name("left").ok_or("Expected a left node but found None")?;

  let right_node = p_node.child_by_field_name("right").ok_or("Expected a right node but found None")?;

  let left_result = normalize_match(
    left_node,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    source_code,
  )?;

  let right_result  = normalize_match(
    right_node,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone().push(),
      free_map: FreeMap::default()
    },
    source_code,
  )?;

  let new_expr = Expr {
    expr_instance: Some(expr::ExprInstance::EMatchesBody(EMatches {
      target: Some(left_result.par.clone()),
      pattern: Some(right_result.par.clone()),
    })),
  };

  let prepend_par = prepend_expr(input.par, new_expr, input.bound_map_chain.depth() as i32);

  Ok(ProcVisitOutputs {
    par: prepend_par,
    free_map: left_result.free_map,
  })
}

//The test with Nil is described because the normalization logic for Nil is already ready in normalize_match engine.
#[test]
fn test_normalize_nil_matches_nil() {
  let rholang_code = "Nil matches Nil";
  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let matches_node = root_node.child(0).unwrap();
  println!("Found matches node: {}", matches_node.to_sexp());
  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: FreeMap::new(),
  };

  let output = normalize_p_matches(matches_node, input.clone(), rholang_code.as_bytes()).unwrap();

  let expected_par = Par {
    exprs: vec![Expr {
      expr_instance: Some(expr::ExprInstance::EMatchesBody(EMatches {
        target: Some(Par::default()),
        pattern: Some(Par::default()),
      })),
    }],
    ..Default::default()
  };

  assert_eq!(output.par, expected_par);
}
