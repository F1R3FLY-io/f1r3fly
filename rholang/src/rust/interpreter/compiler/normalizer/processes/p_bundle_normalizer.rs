use models::rhoapi::{Bundle, EVar, Expr, expr, Par, Var, var};
use super::exports::*;
use tree_sitter::Node;
use std::error::Error;
use std::result::Result;
use models::BitSet;
use models::rhoapi::var::VarInstance::BoundVar;
use crate::rust::interpreter::compiler::exports::{BoundMapChain, FreeMap};
use crate::rust::interpreter::compiler::normalize::{normalize_match, VarSort};


/*
The PBundle normalizer is responsible for normalizing bundle expressions in Rholang.
In Rholang, a bundle defines access rights to a process or part of a program.
This may include write or read permissions.
The bundle expression itself defines an object that restricts the ability to modify or read the content.
 */
pub fn normalize_p_bundle(
  node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {

  println!("Normalizing bundle node of kind: {}", node.kind());
  let target_result = normalize_match(node.child_by_field_name("proc").unwrap(), input.clone(), source_code).unwrap();

  if !target_result.par.connectives.is_empty() {
    return Err(format!(
      "Illegal top-level connective in bundle at position: line {}, column {}.",
      node.start_position().row,
      node.start_position().column
    )
      .into());
  }
  println!("Target result after normalizing proc: {:?}", target_result);
  // bundle -> bundle_type
  let outermost_bundle = match node.child_by_field_name("bundle_type").unwrap().kind() {
    "bundle_read_write" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: true,
      read_flag: true,
    },
    "bundle_read" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: false,
      read_flag: true,
    },
    "bundle_write" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: true,
      read_flag: false,
    },
    "bundle_equiv" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: false,
      read_flag: false,
    },
    _ => return Err("Unknown bundle type".into()),
  };

  if !target_result.free_map.wildcards.is_empty() || !target_result.free_map.level_bindings.is_empty() {
    return raise_error(target_result);
  }

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

fn raise_error(target_result: ProcVisitOutputs) -> Result<ProcVisitOutputs, Box<dyn Error>> {
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
      format!("Wildcards positions: {}", wildcards_positions.join(", "))
    } else {
      String::new()
    };

    let err_msg_free_vars = if !free_vars_positions.is_empty() {
      format!("Free variables positions: {}", free_vars_positions.join(", "))
    } else {
      String::new()
    };

    format!(
      "Bundle's content must not have free variables or wildcards. {} {}",
      err_msg_wildcards, err_msg_free_vars
    )
  };

  Err(err_msg.into())
}

#[test]
fn test_normalize_bundle_plus() {
  let rholang_code = r#" { bundle+ {Nil} }"#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let block_node = root_node.child(0).expect("Expected a block node");
  println!("Found block node: {}", block_node.to_sexp());
  let bundle_node = block_node.child_by_field_name("body").expect("Expected a bundle node");
  println!("Found bundle node: {}", bundle_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: BoundMapChain::default(),
    free_map: FreeMap::default(),
  };

  let expected_par = Par {
    bundles: vec![Bundle {
      body: Some(Par::default()),
      write_flag: true,
      read_flag: false,
    }],
    ..Par::default()
  };

  //BTW, I can call normalize_match engine with block_node instead of normalize_p_bundle and bundle_node
  match normalize_p_bundle(bundle_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      println!("Normalization succeeded. Resulting Par: {:?}", result.par);
      assert_eq!(result.par, expected_par, "Expected Par structure does not match");
    }
    Err(e) => {
      println!("Normalization failed with error: {}", e);
      assert!(false, "Expected normalization to succeed but it failed.");
    }
  }
}

//it should work when I have a proc_var case inside normalize_match
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

#[test]
fn test_bundle_with_top_level_connective() {
  let rholang_code = r#" { bundle+ { Nil and Nil } }"#;

  let tree = parse_rholang_code(rholang_code);
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let block_node = root_node.child(0).expect("Expected a block node");
  println!("Found block node: {}", block_node.to_sexp());
  let bundle_node = block_node.child_by_field_name("body").expect("Expected a bundle node");
  println!("Found bundle node: {}", bundle_node.to_sexp());
  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: BoundMapChain::default(),
    free_map: FreeMap::default(),
  };

  // Очікуємо помилку при нормалізації з'єднання на верхньому рівні тіла bundle
  match normalize_p_bundle(bundle_node, input, rholang_code.as_bytes()) {
    Ok(_) => assert!(false, "Expected an error for top-level connective, but got Ok"),
    Err(e) => {
      println!("Normalization failed with error: {}", e);
      assert!(e.to_string().contains("Illegal top-level connective"), "Expected a connective error");
    }
  }
}










