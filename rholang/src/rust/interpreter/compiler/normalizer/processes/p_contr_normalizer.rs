use std::collections::HashMap;
use models::rhoapi::{Par, Receive, ReceiveBind};
use models::rust::utils::union;
use crate::rust::interpreter::compiler::normalize::{NameVisitInputs, normalize_match_proc, ProcVisitInputs, ProcVisitOutputs, VarSort};
use crate::rust::interpreter::compiler::normalizer::processes::utils::fail_on_invalid_connective;
use crate::rust::interpreter::compiler::normalizer::remainder_normalizer_matcher::normalize_match_name;
use crate::rust::interpreter::compiler::rholang_ast::{Block, Name, Names};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use crate::rust::interpreter::util::filter_and_adjust_bitset;
use super::exports::*;

pub fn normalize_p_contr(
  name: &Name,
  formals: &Names,
  proc: &Box<Block>,
  input: ProcVisitInputs,
  env: &HashMap<String, Par>
) -> Result<ProcVisitOutputs, InterpreterError> {

  let name_match_result = normalize_name(
    name,
    NameVisitInputs {
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    env
  )?;

  let mut init_acc = (vec![], FreeMap::<VarSort>::default(), Vec::new());

  for name in formals.names.clone() {
    let res = normalize_name(
      &name,
      NameVisitInputs {
        bound_map_chain: input.clone().bound_map_chain.push(),
        free_map: init_acc.1.clone(),
      },
      env)?;

    let result = fail_on_invalid_connective(&input, &res)?;

    // Accumulate the result
    init_acc.0.insert(0, result.par.clone());
    init_acc.1 = result.free_map.clone();
    init_acc.2 = union(init_acc.clone().2, result.par.locally_free(result.par.clone(), (input.bound_map_chain.depth() + 1) as i32));
  }

  let remainder_result = normalize_match_name(&formals.cont, init_acc.1.clone())?;

  let new_enw = input.bound_map_chain.absorb_free(remainder_result.1.clone());
  let bound_count = remainder_result.1.count_no_wildcards();

  let body_result = normalize_match_proc(
    &proc.proc,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: new_enw,
      free_map: name_match_result.free_map.clone(),
    },
    env
  )?;

  let receive = Receive {
    binds: vec![ReceiveBind {
      patterns: init_acc.0.clone().into_iter().rev().collect(),
      source: Some(name_match_result.par.clone()),
      remainder: remainder_result.0.clone(),
      free_count: bound_count as i32,
    }],
    body: Some(body_result.par.clone()),
    persistent: true,
    peek: false,
    bind_count: bound_count as i32,
    // locally_free: union(
    //   union(
    //     init_acc.2,
    //     name_match_result.par.locally_free(name_match_result.par.clone(), (input.bound_map_chain.depth() + 1) as i32),
    //   ),
    //   //In Scala, .from(boundCount) returns a new collection starting at element boundCount, that is, it contains all elements greater than or equal to boundCount.
    //   //Next, .map(x => x - boundCount) decrements each value in this collection by boundCount.
    //   body_result.par.locally_free(body_result.par.clone(), (bound_count as i32))
    //     .iter()
    //     .filter(|&&x| x >= bound_count as u8)
    //     .map(|&x| x - bound_count as u8)
    //     .collect::<Vec<u8>>(),
    // ),
    locally_free: union(
      name_match_result.par.locally_free(
          name_match_result.par.clone(),
          input.bound_map_chain.depth() as i32,
      ),
      union(
          init_acc.2,
          filter_and_adjust_bitset(body_result.par.clone().locally_free, bound_count),
      ),
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

// #[test]
// fn test_normalize_p_contr() {
//   let rholang_code = r#"
//     contract sum(var) = {
//         "10"
//     }
//     "#;
//
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", root_node.to_sexp());
//   println!("Root node kind: {}", root_node.kind());
//
//   let contract_node = root_node.child(0).expect("Expected a contract node");
//   println!("Found contract node: {}", contract_node.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: Default::default(),
//   };
//
//   match normalize_match(contract_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       println!("Normalization successful!");
//
//       assert_eq!(result.par.receives.len(), 1);
//       if let Some(receive) = result.par.receives.get(0) {
//         if let Some(body_par) = &receive.body {
//           if let Some(ExprInstance::GString(value)) = body_par.exprs.get(0).and_then(|e| e.expr_instance.clone()) {
//             assert_eq!(value, "10", "Expected value '10' in contract body");
//           } else {
//             panic!("Contract body is not a GString with value '10'");
//           }
//         } else {
//           panic!("Receive body is missing");
//         }
//       } else {
//         panic!("No Receive found in result.par");
//       }
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
