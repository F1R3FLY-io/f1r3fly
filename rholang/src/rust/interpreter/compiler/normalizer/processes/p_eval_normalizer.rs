use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    NameVisitInputs, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::compiler::rholang_ast::Eval;
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::Par;
use std::collections::HashMap;

pub fn normalize_p_eval(
    proc: &Eval,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let name_match_result = normalize_name(
        &proc.name,
        NameVisitInputs {
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: input.free_map.clone(),
        },
        env,
    )?;

    let updated_par = input.par.append(name_match_result.par.clone());

    Ok(ProcVisitOutputs {
        par: updated_par,
        free_map: name_match_result.free_map,
    })
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use models::rust::utils::new_boundvar_expr;

    use crate::rust::interpreter::{
        compiler::{
            normalize::{normalize_match_proc, VarSort},
            rholang_ast::{Eval, Name, Quote},
        },
        test_utils::utils::proc_visit_inputs_and_env,
        util::prepend_expr,
    };

    use super::{Proc, SourcePosition};

    #[test]
    fn p_eval_should_handle_a_bound_name_variable() {
        let p_eval = Proc::Eval(Eval {
            name: Name::new_name_var("x", 0, 0),
            line_num: 0,
            col_num: 0,
        });

        let (mut inputs, env) = proc_visit_inputs_and_env();
        inputs.bound_map_chain = inputs.bound_map_chain.put((
            "x".to_string(),
            VarSort::NameSort,
            SourcePosition::new(0, 0),
        ));

        let result = normalize_match_proc(&p_eval, inputs.clone(), &env);
        assert!(result.is_ok());
        assert_eq!(
            result.clone().unwrap().par,
            prepend_expr(inputs.par, new_boundvar_expr(0), 0)
        );
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_eval_should_collapse_a_quote() {
        let p_eval = Proc::Eval(Eval {
            name: Name::Quote(Box::new(Quote {
                quotable: Box::new(Proc::Par {
                    left: Box::new(Proc::new_proc_var("x", 0, 0)),
                    right: Box::new(Proc::new_proc_var("x", 0, 0)),
                    line_num: 0,
                    col_num: 0,
                }),
                line_num: 0,
                col_num: 0,
            })),
            line_num: 0,
            col_num: 0,
        });

        let (mut inputs, env) = proc_visit_inputs_and_env();
        inputs.bound_map_chain = inputs.bound_map_chain.put((
            "x".to_string(),
            VarSort::ProcSort,
            SourcePosition::new(0, 0),
        ));

        let result = normalize_match_proc(&p_eval, inputs.clone(), &env);
        assert!(result.is_ok());
        assert_eq!(
            result.clone().unwrap().par,
            prepend_expr(
                prepend_expr(inputs.par, new_boundvar_expr(0), 0),
                new_boundvar_expr(0),
                0
            )
        );
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }
}
