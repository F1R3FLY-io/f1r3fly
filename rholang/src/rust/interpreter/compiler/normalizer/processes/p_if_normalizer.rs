use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::{Match, MatchCase, Par};
use models::rust::utils::{new_gbool_par, union};

pub fn normalize_p_if(
    value_proc: &Proc,
    true_body_proc: &Proc,
    false_body_proc: &Proc,
    mut input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let target_result = normalize_match_proc(&value_proc, ProcVisitInputs { ..input.clone() })?;

    let true_case_body = normalize_match_proc(
        &true_body_proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: target_result.free_map.clone(),
        },
    )?;

    let false_case_body = normalize_match_proc(
        &false_body_proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: true_case_body.free_map.clone(),
        },
    )?;

    // Construct the desugared if as a Match
    let desugared_if = Match {
        target: Some(target_result.par.clone()),
        cases: vec![
            MatchCase {
                pattern: Some(new_gbool_par(true, vec![], false)),
                source: Some(true_case_body.par.clone()),
                free_count: 0,
            },
            MatchCase {
                pattern: Some(new_gbool_par(false, vec![], false)),
                source: Some(false_case_body.par.clone()),
                free_count: 0,
            },
        ],
        locally_free: union(
            union(
                target_result.par.locally_free.clone(),
                true_case_body.par.locally_free.clone(),
            ),
            false_case_body.par.locally_free.clone(),
        ),
        connective_used: target_result.par.connective_used
            || true_case_body.par.connective_used
            || false_case_body.par.connective_used,
    };

    // Update the input par by prepending the desugared if statement
    let updated_par = input.par.prepend_match(desugared_if);

    Ok(ProcVisitOutputs {
        par: updated_par,
        free_map: false_case_body.free_map,
    })
}

// #[test]
// fn test_normalize_p_if() {
//   let rholang_code = r#"
//         if (x) { y } else { z }
//     "#;
//
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", root_node.to_sexp());
//   println!("Root node kind: {}", root_node.kind());
//
//   let if_node = root_node.named_child(0).expect("Expected an if node");
//   println!("Found if node: {}", if_node.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: Default::default(),
//   };
//
//   match normalize_match(if_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       println!("Normalization successful!");
//       println!("Resulting Par: {:?}", result.par);
//       assert_eq!(result.par.matches.len(), 1, "Expected one match in the resulting Par");
//       let match_case = &result.par.matches[0];
//       assert_eq!(match_case.cases.len(), 2, "Expected two cases in the match");
//       assert!(matches!(match_case.cases[0].pattern.as_ref().unwrap().exprs[0].expr_instance, Some(expr::ExprInstance::GBool(true))), "Expected first case pattern to be true");
//       assert!(matches!(match_case.cases[1].pattern.as_ref().unwrap().exprs[0].expr_instance, Some(expr::ExprInstance::GBool(false))), "Expected second case pattern to be false");
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
//
// #[test]
// fn test_normalize_p_if_without_else() {
//   let rholang_code = r#"
//         if (x) { y }
//     "#;
//
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", root_node.to_sexp());
//   println!("Root node kind: {}", root_node.kind());
//
//   let if_node = root_node.named_child(0).expect("Expected an if node");
//   println!("Found if node: {}", if_node.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: Default::default(),
//   };
//
//   match normalize_match(if_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       println!("Normalization successful!");
//       println!("Resulting Par: {:?}", result.par);
//       assert_eq!(result.par.matches.len(), 1, "Expected one match in the resulting Par");
//       let match_case = &result.par.matches[0];
//       assert_eq!(match_case.cases.len(), 2, "Expected two cases in the match");
//       assert!(matches!(match_case.cases[0].pattern.as_ref().unwrap().exprs[0].expr_instance, Some(expr::ExprInstance::GBool(true))), "Expected first case pattern to be true");
//       assert!(matches!(match_case.cases[1].pattern.as_ref().unwrap().exprs[0].expr_instance, Some(expr::ExprInstance::GBool(false))), "Expected second case pattern to be false");
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
