use std::collections::HashMap;
use super::exports::{FreeMap, InterpreterError, Proc, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::{compiler::normalize::normalize_match_proc, util::prepend_expr};
use models::rhoapi::{expr, EMatches, Expr, Par};

pub fn normalize_p_matches(
    left_proc: &Proc,
    right_proc: &Proc,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>
) -> Result<ProcVisitOutputs, InterpreterError> {
    let left_result = normalize_match_proc(
        left_proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: input.free_map.clone(),
        },
        env
    )?;

    let right_result = normalize_match_proc(
        right_proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone().push(),
            free_map: FreeMap::default(),
        },
        env
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
// #[test]
// fn test_normalize_nil_matches_nil() {
//     let rholang_code = "Nil matches Nil";
//     let tree = parse_rholang_code(rholang_code);
//     let root_node = tree.root_node();
//     println!("Tree S-expression: {}", tree.root_node().to_sexp());
//     let matches_node = root_node.child(0).unwrap();
//     println!("Found matches node: {}", matches_node.to_sexp());
//     let input = ProcVisitInputs {
//         par: Par::default(),
//         bound_map_chain: Default::default(),
//         free_map: FreeMap::new(),
//     };

//     let output = normalize_p_matches(matches_node, input.clone(), rholang_code.as_bytes()).unwrap();

//     let expected_par = Par {
//         exprs: vec![Expr {
//             expr_instance: Some(expr::ExprInstance::EMatchesBody(EMatches {
//                 target: Some(Par::default()),
//                 pattern: Some(Par::default()),
//             })),
//         }],
//         ..Default::default()
//     };

//     assert_eq!(output.par, expected_par);
// }
