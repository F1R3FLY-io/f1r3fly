use std::error::Error;
use models::rhoapi::{Connective, ConnectiveBody, Par};
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::expr::ExprInstance;
use tree_sitter::Node;
use crate::rust::interpreter::compiler::exports::SourcePosition;
use crate::rust::interpreter::compiler::normalize::{normalize_match, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::normalizer::exports::parse_rholang_code;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;

pub fn normalize_p_сonjunction(
  node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing сonjunction node of kind: {}", node.kind());

  let left_node = node.child_by_field_name("left").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected left operand in conjunction".to_string())
  })?;

  let right_node = node.child_by_field_name("right").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected right operand in conjunction".to_string())
  })?;

  let left_result = normalize_match(
    left_node,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    source_code,
  )?;

  let mut right_result = normalize_match(
    right_node,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: left_result.free_map.clone(),
    },
    source_code,
  )?;

  let lp = left_result.par;
  let result_connective = match lp.single_connective() {
    Some(Connective { connective_instance: Some(ConnectiveInstance::ConnAndBody(conn_body)), }) =>
      Connective {
        connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
          ps: {
            let mut ps = conn_body.ps.clone();
            ps.push(right_result.par);
            ps
          },
        })),
      },
    _ => Connective {
      connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
        ps: vec![lp, right_result.par],
      })),
    },
  };

  let result_par = prepend_connective(input.par, result_connective.clone(), input.bound_map_chain.depth() as i32);

  let updated_free_map = right_result
    .free_map
    .add_connective(result_connective.connective_instance.unwrap(), SourcePosition {
      row: node.start_position().row as usize,
      column: node.start_position().column as usize,
    });

  Ok(ProcVisitOutputs {
    par: result_par,
    free_map: updated_free_map,
  })
}

#[test]
fn test_normalize_conjunction() {
  let rholang_code = r#"{ 1 /\ (1,"2")}"#;
  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let block_node = root_node.child(0).expect("Expected a block node");
  println!("Found block node: {}", block_node.to_sexp());

  let conjunction_node = block_node.child_by_field_name("body").expect("Expected a conjunction node");
  println!("Found conjunction node: {}", conjunction_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(conjunction_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");

      assert_eq!(result.par.connectives.len(), 1);
      if let Some(ConnectiveInstance::ConnAndBody(conn_body)) = &result.par.connectives[0].connective_instance {
        assert_eq!(conn_body.ps.len(), 2);

        // Check left operand
        if let Some(ExprInstance::GInt(value)) = conn_body.ps[0].exprs.get(0).and_then(|e| e.expr_instance.clone()) {
          assert_eq!(value, 1);
        } else {
          panic!("Left operand is not GInt(1)");
        }

        // Check right operand is a tuple with expected values
        if let Some(ExprInstance::ETupleBody(tuple_body)) = conn_body.ps[1].exprs.get(0).and_then(|e| e.expr_instance.clone()) {
          assert_eq!(tuple_body.ps.len(), 2);
          if let Some(ExprInstance::GInt(inner_value)) = tuple_body.ps[0].exprs.get(0).and_then(|e| e.expr_instance.clone()) {
            assert_eq!(inner_value, 1);
          } else {
            panic!("First element of tuple is not GInt(1)");
          }
          if let Some(ExprInstance::GString(inner_value)) = tuple_body.ps[1].exprs.get(0).and_then(|e| e.expr_instance.clone()) {
            assert_eq!(inner_value, "2");
          } else {
            panic!("Second element of tuple is not GString(\"2\")");
          }
        } else {
          panic!("Right operand is not a tuple");
        }
      } else {
        panic!("Expected ConnAndBody in connectives");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}