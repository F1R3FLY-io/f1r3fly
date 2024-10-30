use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::errors::InterpreterError;

pub fn normalize_p_par(
    left: &Proc,
    right: &Proc,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let result = normalize_match_proc(&left, input.clone())?;
    let chained_input = ProcVisitInputs {
        par: result.par.clone(),
        free_map: result.free_map.clone(),
        ..input.clone()
    };

    let chained_res = normalize_match_proc(&right, chained_input)?;
    Ok(chained_res)
}

// #[test]
// fn test_normalize_p_par() {
//   let rholang_code = r#"
//         x | y
//     "#;

//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", root_node.to_sexp());
//   println!("Root node kind: {}", root_node.kind());

//   let par_node = root_node.named_child(0).expect("Expected a par node");
//   println!("Found par node: {}", par_node.to_sexp());

//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: Default::default(),
//   };

//   match normalize_match(par_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       println!("Normalization successful!");
//       assert_eq!(result.par.exprs.len(), 2, "Expected two expressions in the resulting Par");

//       let mut free_vars = vec![];
//       for expr in &result.par.exprs {
//         if let Some(expr::ExprInstance::EVarBody(evar)) = &expr.expr_instance {
//           if let Some(var::VarInstance::FreeVar(idx)) = evar.v.as_ref().unwrap().var_instance {
//             free_vars.push(idx);
//           }
//         }
//       }
//       assert!(free_vars.contains(&0), "Expected first variable to be a free variable");
//       assert!(free_vars.contains(&1), "Expected second variable to be a free variable");
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
