use models::rhoapi::{Bundle, EVar, Par};
use super::exports::*;
use tree_sitter::Node;
use std::error::Error;
use std::result::Result;
use models::BitSet;
use crate::rust::interpreter::compiler::exports::{BoundMapChain, FreeMap};
use crate::rust::interpreter::compiler::normalize::{normalize_match, VarSort};


/*
The PBundle normalizer is responsible for normalizing bundle expressions in Rholang.
In Rholang, a bundle defines access rights to a process or part of a program.
This may include write or read permissions.
The bundle expression itself defines an object that restricts the ability to modify or read the content.
 */
pub fn normalize_p_bundle(
  b_node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {

  for i in 0..b_node.child_count() {
    if let Some(child) = b_node.child(i) {
      println!("Child {}: {} - {}", i, child.kind(), child.utf8_text(source_code).unwrap_or(""));
    }
  }

  let proc_node = b_node.child_by_field_name("proc").ok_or("Missing proc in bundle")?;
  let target_result = normalize_match(proc_node, input.clone(), source_code)?;

  let outermost_bundle = match b_node.kind() {
    "Bundle+" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: true,
      read_flag: true,
    },
    "Bundle-" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: false,
      read_flag: true,
    },
    "Bundle0" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: true,
      read_flag: false,
    },
    "Bundle" => Bundle {
      body: Some(target_result.par.clone()),
      write_flag: false,
      read_flag: false,
    },
    _ => return Err("Unknown bundle type".into()),
  };

  if !target_result.par.connectives.is_empty() {
    return Err(format!(
      "Illegal top-level connective in bundle at position: line {}, column {}.",
      b_node.start_position().row,
      b_node.start_position().column
    )
      .into());
  }

  if !target_result.free_map.wildcards.is_empty() || !target_result.free_map.level_bindings.is_empty() {
    return raise_error(target_result);
  }

  let new_bundle = match target_result.par.bundles.get(0) {
    Some(single) => merge_bundles(&outermost_bundle, single),
    None => outermost_bundle,
  };

  let mut par = input.par;
  par.bundles.push(new_bundle);
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

fn walk_tree<'a>(node: tree_sitter::Node<'a>, target_kind: &str, source_code: &'a [u8], level: usize) -> Option<tree_sitter::Node<'a>> {
  let indent = "  ".repeat(level);
  let node_text = node.utf8_text(source_code).unwrap_or(""); // Отримуємо текст вузла
  println!("{}{}: {}", indent, node.kind(), node_text); // Виводимо відступи, тип вузла і текст

  if node.kind() == target_kind {
    return Some(node);
  }

  for i in 0..node.child_count() {
    if let Some(child) = node.child(i) {
      if let Some(found) = walk_tree(child, target_kind, source_code, level + 1) {
        return Some(found);
      }
    }
  }
  None
}

//This test not working

// #[test]
// fn test_normalize_bundle_with_var() {
//   let rholang_code = r#" { bundle+ {Nil} }"#;
//
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//
//   // Запускаємо рекурсивний обхід дерева для пошуку вузла "bundle_proc" і виведення всіх нод
//   let bundle_node = walk_tree(root_node, "bundle_proc", rholang_code.as_bytes(), 0)
//     .expect("Expected a Bundle node");
//   println!("Found bundle node: {}", bundle_node.to_sexp());
//
//   // Перевіряємо, що вузол є "bundle_proc"
//   assert_eq!(bundle_node.kind(), "bundle_proc");
//
//   // Перевіряємо, що текст вмісту відповідає "bundle+"
//   let bundle_text = &rholang_code[bundle_node.start_byte()..bundle_node.start_byte() + "bundle+".len()];
//   assert_eq!(bundle_text, "bundle+");
//
//
//   let mut bound_map_chain = BoundMapChain::default();
//   bound_map_chain.put(("x".to_string(), VarSort::ProcSort, SourcePosition { row: 0, column: 0 }));
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain,
//     free_map: FreeMap::default(),
//   };
//   println!("Bundle node S-expression: {}", bundle_node.to_sexp());
//   println!("Input for normalization: {:?}", input);
//   let result = normalize_p_bundle(bundle_node, input, rholang_code.as_bytes());
//   assert!(result.is_ok());
//
//   let output = result.unwrap();
//   assert!(!output.par.bundles.is_empty());
//   assert_eq!(output.par.bundles.len(), 1);
//   assert!(output.par.bundles[0].write_flag);
//   assert!(output.par.bundles[0].read_flag);
//
//   // Перевіряємо очікуваний результат
//   let expected_par = Par::default().with_bundles(vec![Bundle {
//     body: Some(new_boundvar_par(0, vec![], false)),
//     write_flag: true,
//     read_flag: true,
//   }]).with_locally_free(vec![0]); // locally_free - вектор байтів
//
//   assert_eq!(output.par, expected_par);
// }




