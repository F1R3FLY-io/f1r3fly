use models::rhoapi::{Bundle, Par};
use super::exports::*;
use std::result::Result;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc};
use crate::rust::interpreter::compiler::rholang_ast::{Block, BundleType};

pub fn normalize_p_bundle(
  bundle_type: &BundleType,
  block: &Box<Block>,
  input: ProcVisitInputs,
  line_num: usize,
  column_num: usize,
) -> Result<ProcVisitOutputs, InterpreterError> {
  let target_result = normalize_match_proc(&block.proc, ProcVisitInputs {
    par: Par::default(),
    ..input.clone()
  })?;
  if !target_result.par.connectives.is_empty() {
    return Err(InterpreterError::NormalizerError(format!(
      "Illegal top-level connective in bundle at line {}, column {}.",
      line_num, column_num
    )));
  }

  if !target_result.free_map.wildcards.is_empty() || !target_result.free_map.level_bindings.is_empty() {
    return raise_error(target_result);
  }

  let outermost_bundle = match bundle_type {
    BundleType::BundleReadWrite { .. } => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: true,
      read_flag: true,
    },
    BundleType::BundleRead { .. } => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: false,
      read_flag: true,
    },
    BundleType::BundleWrite { .. } => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: true,
      read_flag: false,
    },
    BundleType::BundleEquiv { .. } => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: false,
      read_flag: false,
    },
  };

  let new_bundle = match target_result.par.bundles.get(0) {
    Some(single) => merge_bundles(&outermost_bundle, single),
    None => outermost_bundle,
  };

  let mut par = input.par;
  par.bundles.push(new_bundle);

  println!("Final par with new bundle: {:?}", par);
  Ok(ProcVisitOutputs {
    par,
    free_map: input.free_map.clone(),
  })
}

fn merge_bundles(outer: &Bundle, inner: &Bundle) -> Bundle {
  Bundle {
    body: outer.body.clone(),
    write_flag: outer.write_flag || inner.write_flag,
    read_flag: outer.read_flag || inner.read_flag,
  }
}

fn raise_error(target_result: ProcVisitOutputs) -> Result<ProcVisitOutputs, InterpreterError> {
  let err_msg = {
    let wildcards_positions: Vec<String> = target_result
      .free_map
      .wildcards
      .iter()
      .map(|pos| format!(" at line {}, column {}", pos.row, pos.column))
      .collect();

    let free_vars_positions: Vec<String> = target_result
      .free_map
      .level_bindings
      .iter()
      .map(|(name, context)| format!("`{}` at line {}, column {}", name, context.source_position.row, context.source_position.column))
      .collect();

    let err_msg_wildcards = if !wildcards_positions.is_empty() {
      format!(" Wildcards positions: {}", wildcards_positions.join(", "))
    } else {
      String::new()
    };

    let err_msg_free_vars = if !free_vars_positions.is_empty() {
      format!(" Free variables positions: {}", free_vars_positions.join(", "))
    } else {
      String::new()
    };

    format!(
      "Bundle's content must not have free variables or wildcards.{}{}",
      err_msg_wildcards, err_msg_free_vars
    )
  };

  Err(InterpreterError::NormalizerError(err_msg))
}

#[test]
fn test_normalize_p_bundle() {
  let bundle_type = BundleType::BundleReadWrite { line_num: 1, col_num: 2 };
  let block = Block {
    proc: Proc::Nil { line_num: 1, col_num: 2 },
    line_num: 1,
    col_num: 2,
  };
  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };
  let result = normalize_p_bundle(&bundle_type, &Box::new(block), input, 1, 2);
  match result {
    Ok(output) => println!("Normalization successful: {:?}", output),
    Err(e) => println!("Normalization failed: {:?}", e),
  }
}

//it should work when I proc_var case will be described inside normalize_match
// #[test]
// fn test_normalize_bundle_with_evar() {
//   let rholang_code = r#" { bundle+ { x } }"#;
//
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let block_node = root_node.child(0).expect("Expected a block node");
//   println!("Found block node: {}", block_node.to_sexp());
//   let bundle_node = block_node.child_by_field_name("body").expect("Expected a bundle node");
//   println!("Found bundle node: {}", bundle_node.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: BoundMapChain::default(),
//     free_map: FreeMap::default(),
//   };
//
//   // Expected result with EVar(BoundVar(0))
//   let expected_par = Par {
//     bundles: vec![Bundle {
//       body: Some(Par {
//         exprs: vec![Expr {
//           expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
//             v: Some(Var {
//               var_instance: Some(var::VarInstance::BoundVar(0)),
//             }),
//           })),
//         }],
//         locally_free: vec![0], // Locally free set for variable
//         ..Par::default()
//       }),
//       write_flag: true,
//       read_flag: true, // Corresponds to bundle+
//     }],
//     locally_free: vec![0], // Locally free set for the top-level Par
//     ..Par::default()
//   };
//
//   // Call normalize_match, which should internally handle bundle normalization
//   match normalize_p_bundle(bundle_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       println!("Normalization succeeded. Resulting Par: {:?}", result.par);
//       assert_eq!(result.par, expected_par, "Expected Par structure does not match");
//     }
//     Err(e) => {
//       println!("Normalization failed with error: {}", e);
//       assert!(false, "Expected normalization to succeed but it failed.");
//     }
//   }
// }

//it should work when I send case will be described inside normalize_match
// #[test]
// fn test_bundle_with_top_level_connective() {
//   let rholang_code = r#" { bundle { @Nil!(Uri) } }"#;
//
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let block_node = root_node.child(0).expect("Expected a block node");
//   println!("Found block node: {}", block_node.to_sexp());
//   let bundle_node = block_node.child_by_field_name("body").expect("Expected a bundle node");
//   println!("Found bundle node: {}", bundle_node.to_sexp());
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: BoundMapChain::default(),
//     free_map: FreeMap::default(),
//   };
//
//   match normalize_p_bundle(bundle_node, input, rholang_code.as_bytes()) {
//     Ok(_) => assert!(false, "Expected an error for top-level connective, but got Ok"),
//     Err(e) => {
//       println!("Normalization failed with error: {}", e);
//       assert!(e.to_string().contains("Illegal top-level connective"), "Expected a connective error");
//     }
//   }
// }

//it should work when I proc_var case will be described inside normalize_match
// #[test]
// fn test_bundle_with_wildcard_and_free_var() {
//   let rholang_code = r#" { bundle+ { _ | x } }"#;
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let block_node = root_node.child(0).expect("Expected a block node");
//   println!("Found block node: {}", block_node.to_sexp());
//   let bundle_node = block_node.child_by_field_name("body").expect("Expected a bundle node");
//   println!("Found bundle node: {}", bundle_node.to_sexp());
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: BoundMapChain::default(),
//     free_map: FreeMap::default(),
//   };
//   match normalize_p_bundle(bundle_node, input, rholang_code.as_bytes()) {
//     Ok(_) => assert!(false, "Expected an error due to wildcard or free variable, but got Ok"),
//     Err(e) => {
//       println!("Normalization failed with error: {}", e);
//       assert!(
//         e.to_string().contains("Bundle's content must not have free variables or wildcards"),
//         "Expected a wildcard or free variable error"
//       );
//     }
//   }
// }











