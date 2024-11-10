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
use std::collections::HashMap;

pub fn normalize_p_method(
    receiver: &Proc,
    name_var: &Var,
    args: &ProcList,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let target_result = normalize_match_proc(
        receiver,
        ProcVisitInputs {
            par: Par::default(),
            ..input.clone()
        },
        env,
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

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use models::{
        create_bit_vector,
        rhoapi::{expr::ExprInstance, EMethod, Expr, Par},
        rust::utils::{new_boundvar_par, new_gint_par},
    };

    use crate::rust::interpreter::{
        compiler::{
            exports::SourcePosition,
            normalize::{normalize_match_proc, VarSort},
            rholang_ast::{Proc, ProcList, Var},
        },
        test_utils::utils::proc_visit_inputs_and_env,
        util::prepend_expr,
    };

    #[test]
    fn p_method_should_produce_proper_method_call() {
        let methods = vec![String::from("nth"), String::from("toByteArray")];

        fn test(method_name: String) {
            let p_method = Proc::Method {
                receiver: Box::new(Proc::new_proc_var("x")),
                name: Var::new(method_name.clone()),
                args: ProcList::new(vec![Proc::new_proc_int(0)]),
                line_num: 0,
                col_num: 0,
            };

            let (mut inputs, env) = proc_visit_inputs_and_env();
            inputs.bound_map_chain = inputs.bound_map_chain.put((
                "x".to_string(),
                VarSort::ProcSort,
                SourcePosition::new(0, 0),
            ));

            let result = normalize_match_proc(&p_method, inputs.clone(), &env);
            assert!(result.is_ok());

            let expected_result = prepend_expr(
                Par::default(),
                Expr {
                    expr_instance: Some(ExprInstance::EMethodBody(EMethod {
                        method_name,
                        target: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                        arguments: vec![new_gint_par(0, Vec::new(), false)],
                        locally_free: create_bit_vector(&vec![0]),
                        connective_used: false,
                    })),
                },
                0,
            );

            assert_eq!(result.clone().unwrap().par, expected_result);
            assert_eq!(result.unwrap().free_map, inputs.free_map);
        }

        test(methods[0].clone());
        test(methods[1].clone());
    }
}
