use std::error::Error;
use models::rhoapi::{Par, Send};
use models::rust::utils::union;
use tree_sitter::Node;
use crate::rust::interpreter::compiler::normalize::{NameVisitInputs, normalize_match, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use super::exports::*;

pub fn normalize_p_send(
  node: Node,
  mut input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing send node of kind: {}", node.kind());

  // Extract the name field from the send node
  let name_node = node.child_by_field_name("name").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected name in send expression".to_string())
  })?;

  let name_match_result = normalize_name(
    name_node,
    NameVisitInputs {
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    source_code,
  )?;

  let mut acc = (
    Vec::new(),
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: name_match_result.free_map.clone(),
    },
    Vec::new(),
    false,
  );

  let inputs_node = node.child_by_field_name("inputs").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected inputs in send expression".to_string())
  })?;

  for input_node in inputs_node.named_children(&mut node.walk()) {
    let proc_match_result = normalize_match(
      input_node,
      acc.1.clone(),
      source_code,
    )?;

    acc.0.insert(0, proc_match_result.par.clone());
    acc.1 = ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: proc_match_result.free_map.clone(),
    };
    acc.2 = union(acc.2.clone(), proc_match_result.par.locally_free.clone());
    acc.3 = acc.3 || proc_match_result.par.connective_used;
  }

  let persistent = match node.child_by_field_name("send_type").map(|n| n.kind()) {
    Some("send_single") => false,
    Some("send_multiple") => true,
    _ => return Err(InterpreterError::SyntaxError("Unknown send type".to_string()).into()),
  };

  let send = Send {
    chan: Some(name_match_result.par.clone()),
    data: acc.0,
    persistent,
    locally_free: union(
      name_match_result.par.clone()
        .locally_free(name_match_result.par.clone(), input.bound_map_chain.depth() as i32),
      acc.2),
    connective_used: name_match_result.par.connective_used(name_match_result.par.clone()) || acc.3,
  };

  let updated_par = input.par.prepend_send(send);

  Ok(ProcVisitOutputs {
    par: updated_par,
    free_map: acc.1.free_map,
  })
}

#[test]
fn test_normalize_p_send() {
  let rholang_code = r#"
        stdout!("hello, world!", *z)
    "#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let send_node = root_node.child(0).expect("Expected a send node");
  println!("Found send node: {}", send_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(send_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");
      println!("Resulting Par: {:?}", result.par);
      assert_eq!(result.par.sends.len(), 1, "Expected one send in the resulting Par");
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}

#[test]
fn test_normalize_p_send_multiple() {
  let rholang_code = r#"
        HelloWorld!!("Hello, world!")
    "#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let send_node = root_node.child(0).expect("Expected a send node");
  println!("Found send node: {}", send_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(send_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");
      println!("Resulting Par: {:?}", result.par);
      assert_eq!(result.par.sends.len(), 1, "Expected one send in the resulting Par");
      assert!(result.par.sends[0].persistent, "Expected the send to be persistent");
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}