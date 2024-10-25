use std::error::Error;
use models::rhoapi::{Par, Receive, ReceiveBind};
use models::rhoapi::expr::ExprInstance;
use models::rust::utils::union;
use tree_sitter::Node;
use crate::rust::interpreter::compiler::normalize::{NameVisitInputs, normalize_match, ProcVisitInputs, ProcVisitOutputs, VarSort};
use crate::rust::interpreter::compiler::normalizer::processes::Utils::fail_on_invalid_connective;
use crate::rust::interpreter::compiler::normalizer::remainder_normalizer_matcher::normalize_match_name;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use super::exports::*;

pub fn normalize_p_contr(
  node: Node,
  mut input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  println!("Normalizing contract node of kind: {}", node.kind());

  let mut name_node = node.child_by_field_name("name").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected contract name".to_string())
  })?;

  //name normalizer can't handle proc_var, so I should take child from proc_var and provide it to normalize_name
  //If the name is of type "proc_var", extract the actual variable or wildcard
  if name_node.kind() == "proc_var" {
    if let Some(child) = name_node.named_child(0) {
      name_node = child;
    } else {
      return Err(InterpreterError::SyntaxError("Expected a child of proc_var".to_string()).into());
    }
  }
  println!("name node before first call: {}", name_node.kind());
  let name_match_result = normalize_name(
    name_node,
    NameVisitInputs {
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    source_code,
  )?;

  let mut init_acc = (vec![], FreeMap::<VarSort>::default(), Vec::new());

  let names_node = node.child_by_field_name("formals").ok_or_else(|| {
    InterpreterError::SyntaxError("Expected contract formals".to_string())
  })?;

  for mut name in names_node.named_children(&mut node.walk()) { //fold is better?
    // If the name is of type "proc_var", extract the actual variable or wildcard
    if name.kind() == "proc_var" {
      if let Some(child) = name.named_child(0) {
        name = child;
      } else {
        return Err(InterpreterError::SyntaxError("Expected a child of proc_var".to_string()).into());
      }
    }
    println!("name node in cycle: {}", name.kind());
    let res = normalize_name(
      name,
      NameVisitInputs {
        bound_map_chain: input.clone().bound_map_chain.push(),
        free_map: init_acc.1.clone(),
      },
      source_code,
    )?;

    let result = fail_on_invalid_connective(&input, &res)?;

    // Accumulate the result
    init_acc.0.push(result.par.clone());
    init_acc.1 = result.free_map.clone();
    init_acc.2 = union(init_acc.clone().2, result.par.locally_free(result.par.clone(), (input.bound_map_chain.depth() + 1) as i32));
  }

  let remainder_result = normalize_match_name(names_node, init_acc.1.clone(), source_code)?;

  let new_enw = input.bound_map_chain.absorb_free(remainder_result.1.clone());
  let bound_count = remainder_result.1.count_no_wildcards();

  let body_result = normalize_match(
    node.child_by_field_name("proc").ok_or_else(|| {
      InterpreterError::SyntaxError("Expected contract body".to_string())
    })?,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: new_enw,
      free_map: name_match_result.free_map.clone(),
    },
    source_code,
  )?;

  let receive = Receive {
    binds: vec![ReceiveBind {
      patterns: init_acc.0.clone().into_iter().rev().collect(),
      source: Option::from(name_match_result.par.clone()),
      remainder: remainder_result.0.clone(),
      free_count: bound_count as i32,
    }],
    body: Option::from(body_result.par.clone()),
    persistent: true,
    peek: false,
    bind_count: bound_count as i32,
    locally_free: union(
      union(
        init_acc.2,
        name_match_result.par.locally_free(name_match_result.par.clone(), (input.bound_map_chain.depth() + 1) as i32),
      ),
      //In Scala, .from(boundCount) returns a new collection starting at element boundCount, that is, it contains all elements greater than or equal to boundCount.
      //Next, .map(x => x - boundCount) decrements each value in this collection by boundCount.
      body_result.par.locally_free(body_result.par.clone(), (bound_count as i32))
        .iter()
        .filter(|&&x| x >= bound_count as u8)
        .map(|&x| x - bound_count as u8)
        .collect::<Vec<u8>>(),
    ),
    connective_used: name_match_result.par.connective_used(name_match_result.par.clone())
      || body_result.par.connective_used(body_result.par.clone()),
  };
  //I should create new Expr for prepend_expr and provide it instead of receive.clone().into
  let updated_par = input.clone().par.prepend_receive(receive);
  Ok(ProcVisitOutputs {
    par: updated_par,
    free_map: body_result.free_map,
  })
}

#[test]
fn test_normalize_p_contr() {
  let rholang_code = r#"
    contract sum(var) = {
        "10"
    }
    "#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let contract_node = root_node.child(0).expect("Expected a contract node");
  println!("Found contract node: {}", contract_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  match normalize_match(contract_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization successful!");

      assert_eq!(result.par.receives.len(), 1);
      if let Some(receive) = result.par.receives.get(0) {
        if let Some(body_par) = &receive.body {
          if let Some(ExprInstance::GString(value)) = body_par.exprs.get(0).and_then(|e| e.expr_instance.clone()) {
            assert_eq!(value, "10", "Expected value '10' in contract body");
          } else {
            panic!("Contract body is not a GString with value '10'");
          }
        } else {
          panic!("Receive body is missing");
        }
      } else {
        panic!("No Receive found in result.par");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}
