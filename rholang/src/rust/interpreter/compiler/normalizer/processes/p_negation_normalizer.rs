use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::compiler::rholang_ast::Negation;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;
use models::rhoapi::{connective, Connective, Par};
use std::collections::HashMap;

pub fn normalize_p_negation(
    negation: &Negation,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let body_result = normalize_match_proc(
        &negation.proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: FreeMap::default(),
        },
        env,
    )?;

    // Create Connective with ConnNotBody
    let connective = Connective {
        connective_instance: Some(connective::ConnectiveInstance::ConnNotBody(
            body_result.par.clone(),
        )),
    };

    let updated_par = prepend_connective(
        input.par,
        connective.clone(),
        input.bound_map_chain.clone().depth() as i32,
    );

    Ok(ProcVisitOutputs {
        par: updated_par,
        free_map: input.free_map.add_connective(
            connective.connective_instance.unwrap(),
            SourcePosition {
                row: negation.line_num,
                column: negation.col_num,
            },
        ),
    })
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
    use crate::rust::interpreter::compiler::rholang_ast::Negation;
    use crate::rust::interpreter::test_utils::utils::proc_visit_inputs_and_env;
    use models::rhoapi::connective::ConnectiveInstance;
    use models::rhoapi::Connective;
    use models::rust::utils::new_freevar_par;
    use pretty_assertions::assert_eq;

    #[test]
    fn p_negation_should_delegate_but_not_count_any_free_variables_inside() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let proc = Negation::new_negation_var("x");

        let result = normalize_match_proc(&proc, inputs.clone(), &env);
        let expected_result = inputs
            .par
            .with_connectives(vec![Connective {
                connective_instance: Some(ConnectiveInstance::ConnNotBody(new_freevar_par(
                    0,
                    Vec::new(),
                ))),
            }])
            .with_connective_used(true);

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(
            result.clone().unwrap().free_map.level_bindings,
            inputs.free_map.level_bindings
        );
        assert_eq!(
            result.unwrap().free_map.next_level,
            inputs.free_map.next_level
        )
    }
}
