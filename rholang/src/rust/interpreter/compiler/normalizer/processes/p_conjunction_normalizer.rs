use crate::rust::interpreter::compiler::exports::SourcePosition;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::compiler::rholang_ast::Conjunction;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;
use f1r3fly_models::rhoapi::connective::ConnectiveInstance;
use f1r3fly_models::rhoapi::{Connective, ConnectiveBody, Par};
use std::collections::HashMap;

pub fn normalize_p_conjunction(
    proc: &Conjunction,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let left_result = normalize_match_proc(
        &proc.left,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: input.free_map.clone(),
        },
        env,
    )?;

    let right_result = normalize_match_proc(
        &proc.right,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: left_result.free_map.clone(),
        },
        env,
    )?;

    let lp = left_result.par;
    let result_connective = match lp.single_connective() {
        Some(Connective {
            connective_instance: Some(ConnectiveInstance::ConnAndBody(conn_body)),
        }) => Connective {
            connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
                ps: {
                    let mut ps = conn_body.ps.clone();
                    ps.push(right_result.par);
                    ps
                },
            })),
        },
        _ => Connective {
            connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
                ps: vec![lp, right_result.par],
            })),
        },
    };

    let result_par = prepend_connective(
        input.par,
        result_connective.clone(),
        input.bound_map_chain.depth() as i32,
    );

    let updated_free_map = right_result.free_map.add_connective(
        result_connective.connective_instance.unwrap(),
        SourcePosition {
            row: proc.line_num,
            column: proc.col_num,
        },
    );

    Ok(ProcVisitOutputs {
        par: result_par,
        free_map: updated_free_map,
    })
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
    use crate::rust::interpreter::compiler::normalize::VarSort::ProcSort;
    use crate::rust::interpreter::compiler::rholang_ast::Conjunction;
    use crate::rust::interpreter::compiler::source_position::SourcePosition;
    use crate::rust::interpreter::test_utils::utils::proc_visit_inputs_and_env;
    use f1r3fly_models::rhoapi::connective::ConnectiveInstance;
    use f1r3fly_models::rhoapi::{Connective, ConnectiveBody};
    use f1r3fly_models::rust::utils::new_freevar_par;
    use pretty_assertions::assert_eq;

    #[test]
    fn p_conjunction_should_delegate_and_count_any_free_variables_inside() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let proc = Conjunction::new_conjunction_with_par_of_var("x", "y");

        let result = normalize_match_proc(&proc, inputs.clone(), &env);
        let expected_result = inputs
            .par
            .with_connectives(vec![Connective {
                connective_instance: Some(ConnectiveInstance::ConnAndBody(ConnectiveBody {
                    ps: vec![
                        new_freevar_par(0, Vec::new()),
                        new_freevar_par(1, Vec::new()),
                    ],
                })),
            }])
            .with_connective_used(true);
        assert_eq!(result.clone().unwrap().par, expected_result);

        let expected_free = inputs.free_map.put_all(vec![
            ("x".to_string(), ProcSort, SourcePosition::new(0, 0)),
            ("y".to_string(), ProcSort, SourcePosition::new(0, 0)),
        ]);

        assert_eq!(
            result.clone().unwrap().free_map.level_bindings,
            expected_free.level_bindings
        );
        assert_eq!(
            result.unwrap().free_map.next_level,
            expected_free.next_level
        );
    }
}
