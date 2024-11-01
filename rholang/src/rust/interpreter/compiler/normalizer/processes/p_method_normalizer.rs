use std::collections::HashMap;
use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::compiler::rholang_ast::{ProcList, Var};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use crate::rust::interpreter::util::prepend_expr;
use models::rhoapi::{expr, EMethod, Expr, Par};
use models::rust::utils::union;

pub fn normalize_p_method(
    receiver: &Proc,
    name_var: &Var,
    args: &ProcList,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>
) -> Result<ProcVisitOutputs, InterpreterError> {
    let target_result = normalize_match_proc(
        receiver,
        ProcVisitInputs {
            par: Par::default(),
            ..input.clone()
        },
        env
    )?;

    let target = target_result.par;

    let mut acc = (
        Vec::new(),
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: target_result.free_map.clone(),
        },
        Vec::new(),
        false,
    );

    for arg in &args.procs {
        let proc_match_result = normalize_match_proc(&arg, acc.1.clone(), env)?;

        acc.0.insert(0, proc_match_result.par.clone());
        acc.1 = ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: proc_match_result.free_map.clone(),
        };
        acc.2 = union(acc.2.clone(), proc_match_result.par.locally_free.clone());
        acc.3 = acc.3 || proc_match_result.par.connective_used;
    }

    let method = EMethod {
        method_name: name_var.name.clone(),
        target: Some(target.clone()),
        arguments: acc.0,
        locally_free: union(
            target.locally_free(target.clone(), input.bound_map_chain.depth() as i32),
            acc.2,
        ),
        connective_used: target.connective_used(target.clone()) || acc.3,
    };

    let updated_par = prepend_expr(
        input.par,
        Expr {
            expr_instance: Some(expr::ExprInstance::EMethodBody(method)),
        },
        input.bound_map_chain.depth() as i32,
    );

    Ok(ProcVisitOutputs {
        par: updated_par,
        free_map: acc.1.free_map,
    })
}

// #[test]
// fn test_normalize_p_method() {
//     let rholang_code = r#"
//         x.methodName("arg1", *arg2)
//     "#;

//     let tree = parse_rholang_code(rholang_code);
//     let root_node = tree.root_node();
//     println!("Tree S-expression: {}", root_node.to_sexp());
//     println!("Root node kind: {}", root_node.kind());

//     let method_node = root_node.named_child(0).expect("Expected a method node");
//     println!("Found method node: {}", method_node.to_sexp());

//     let input = ProcVisitInputs {
//         par: Par::default(),
//         bound_map_chain: Default::default(),
//         free_map: Default::default(),
//     };

//     match normalize_match(method_node, input, rholang_code.as_bytes()) {
//         Ok(result) => {
//             println!("Normalization successful!");
//             println!("Resulting Par: {:?}", result.par);
//             assert_eq!(
//                 result.par.exprs.len(),
//                 1,
//                 "Expected one expression in the resulting Par"
//             );
//             if let Some(expr::ExprInstance::EMethodBody(emethod)) =
//                 &result.par.exprs[0].expr_instance
//             {
//                 assert_eq!(
//                     emethod.method_name, "methodName",
//                     "Expected method name to be 'methodName'"
//                 );
//                 assert_eq!(
//                     emethod.arguments.len(),
//                     2,
//                     "Expected two arguments in the method"
//                 );
//                 assert!(
//                     matches!(emethod.arguments[1].exprs[0].expr_instance, Some(expr::ExprInstance::GString(ref s)) if s == "arg1"),
//                     "Expected first argument to be 'arg1'"
//                 );
//                 assert!(
//                     matches!(
//                         emethod.arguments[0].exprs[0].expr_instance,
//                         Some(expr::ExprInstance::EVarBody(_))
//                     ),
//                     "Expected second argument to be a variable"
//                 );
//             } else {
//                 panic!("Expected an EMethodBody expression");
//             }
//         }
//         Err(e) => {
//             println!("Normalization failed: {}", e);
//             panic!("Test failed due to normalization error");
//         }
//     }
// }
