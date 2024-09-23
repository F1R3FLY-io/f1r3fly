use crate::rust::interpreter::compiler::normalize::{normalize_match, prepend_expr};
use super::exports::{normalize_ground, ProcVisitOutputs, ProcVisitInputs, FreeMap};
use super::exports::parse_rholang_code;
use models::rhoapi::{EMatches, Expr, expr, Par, Var, var, EVar};
use std::error::Error;
use tree_sitter::Node;

pub fn normalize_p_matches(
  p_node: Node,
  mut input: ProcVisitInputs, // Зробимо input мутабельним
  source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
  // Normalization of 'matches' expression, discarding free variables from the pattern.
  let target_node = p_node.child_by_field_name("target").unwrap();
  let pattern_node = p_node.child_by_field_name("pattern").unwrap();

  let left_result = normalize_match(target_node, input.clone(), source_code)?;

  // Використовуємо мутабельну копію input для додавання нового рівня
  input.bound_map_chain.push(); // Додаємо новий рівень

  let right_result = normalize_match(
    pattern_node,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: FreeMap::new(), // Discard free variables from pattern
    },
    source_code,
  )?;

  let new_expr = Expr {
    expr_instance: Some(expr::ExprInstance::EMatchesBody(EMatches {
      target: Some(left_result.par.clone()),
      pattern: Some(right_result.par.clone()),
    })),
  };

  let new_par = prepend_expr(
    input.par.clone(),
    new_expr,
    input.bound_map_chain.depth() as i32,
  );

  Ok(ProcVisitOutputs {
    par: new_par,
    free_map: left_result.free_map,
  })
}

// #[test]
// fn test_normalize_pmatches_wildcard() {
//   let rholang_code = "1 matches _";
//   let tree = parse_rholang_code(rholang_code);
//   let root = tree.root_node();
//
//   println!("Parsed tree: {:?}", root.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: FreeMap::new(),
//   };
//
//   let output = normalize_p_matches(root, input.clone(), rholang_code.as_bytes()).unwrap();
//
//   let expected_par = Par {
//     exprs: vec![Expr {
//       expr_instance: Some(expr::ExprInstance::EMatchesBody(EMatches {
//         target: Some(Par {
//           exprs: vec![Expr {
//             expr_instance: Some(expr::ExprInstance::GInt(1)),
//           }],
//           ..Default::default()
//         }),
//         pattern: Some(Par {
//           exprs: vec![Expr {
//             expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
//               v: Some(Var {
//                 var_instance: Some(var::VarInstance::Wildcard(var::WildcardMsg {})),
//               })
//             })),
//           }],
//           ..Default::default() // Інші поля залишаємо за замовчуванням
//         }),
//       })),
//     }],
//     ..Default::default() // Інші поля `Par` можна залишити за замовчуванням
//   };
//
//   assert_eq!(output.par, expected_par);
//   assert_eq!(output.par.connective_used, false);
// }