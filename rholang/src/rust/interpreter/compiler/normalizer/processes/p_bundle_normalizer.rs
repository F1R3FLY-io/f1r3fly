use super::exports::*;
use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
use crate::rust::interpreter::compiler::rholang_ast::{Block, BundleType};
use crate::rust::interpreter::util::prepend_bundle;
use models::rhoapi::{Bundle, Par};
use models::rust::bundle_ops::BundleOps;
use std::collections::HashMap;
use std::result::Result;

pub fn normalize_p_bundle(
    bundle_type: &BundleType,
    block: &Box<Block>,
    input: ProcVisitInputs,
    line_num: usize,
    column_num: usize,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    fn error(target_result: ProcVisitOutputs) -> Result<ProcVisitOutputs, InterpreterError> {
        let err_msg = {
            let at = |variable: &str, source_position: &SourcePosition| {
                format!(
                    "{} at line {}, column {}",
                    variable, source_position.row, source_position.column
                )
            };

            let wildcards_positions: Vec<String> = target_result
                .free_map
                .wildcards
                .iter()
                .map(|pos| at("", pos))
                .collect();

            let free_vars_positions: Vec<String> = target_result
                .free_map
                .level_bindings
                .iter()
                .map(|(name, context)| at(&format!("`{}`", name), &context.source_position))
                .collect();

            let err_msg_wildcards = if !wildcards_positions.is_empty() {
                format!(" Wildcards positions: {}", wildcards_positions.join(", "))
            } else {
                String::new()
            };

            let err_msg_free_vars = if !free_vars_positions.is_empty() {
                format!(
                    " Free variables positions: {}",
                    free_vars_positions.join(", ")
                )
            } else {
                String::new()
            };

            format!(
                "Bundle's content must not have free variables or wildcards.{}{}",
                err_msg_wildcards, err_msg_free_vars
            )
        };

        Err(InterpreterError::UnexpectedBundleContent(format!(
            "Bundle's content must not have free variables or wildcards. {}",
            err_msg
        )))
    }
    let target_result = normalize_match_proc(
        &block.proc,
        ProcVisitInputs {
            par: Par::default(),
            ..input.clone()
        },
        env,
    )?;

    let outermost_bundle = match bundle_type {
        BundleType::BundleReadWrite { .. } => Bundle {
            body: Some(target_result.par.clone()),
            write_flag: true,
            read_flag: true,
        },
        BundleType::BundleRead { .. } => Bundle {
            body: Some(target_result.par.clone()),
            write_flag: false,
            read_flag: true,
        },
        BundleType::BundleWrite { .. } => Bundle {
            body: Some(target_result.par.clone()),
            write_flag: true,
            read_flag: false,
        },
        BundleType::BundleEquiv { .. } => Bundle {
            body: Some(target_result.par.clone()),
            write_flag: false,
            read_flag: false,
        },
    };

    let res = if !target_result.clone().par.connectives.is_empty() {
        Err(InterpreterError::UnexpectedBundleContent(format!(
            "Illegal top-level connective in bundle at line {}, column {}.",
            line_num, column_num
        )))
    } else if (!target_result.clone().free_map.wildcards.is_empty()
        || !target_result.free_map.level_bindings.is_empty())
    {
        error(target_result)
    } else {
        let new_bundle = match target_result.par.single_bundle() {
            Some(single) => BundleOps::merge(&outermost_bundle, &single),
            None => outermost_bundle,
        };

        Ok(ProcVisitOutputs {
            par: {
                let updated_bundle = prepend_bundle(input.par.clone(), new_bundle);
                updated_bundle
            },
            free_map: input.free_map.clone(),
        })
    };

    res
}

#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, VarSort};
    use crate::rust::interpreter::compiler::rholang_ast::BundleType::BundleReadWrite;
    use crate::rust::interpreter::compiler::rholang_ast::{
        Block, BundleType, Name, Proc, ProcList, SendType, SimpleType,
    };
    use crate::rust::interpreter::errors::InterpreterError;
    use crate::rust::interpreter::test_utils::utils::proc_visit_inputs_and_env;
    use crate::rust::interpreter::test_utils::utils::proc_visit_inputs_with_updated_bound_map_chain;
    use models::create_bit_vector;
    use models::rhoapi::Bundle;
    use models::rust::utils::new_boundvar_par;
    use pretty_assertions::assert_eq;

    #[test]
    fn p_bundle_should_normalize_terms_inside() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let proc = Proc::Bundle {
            bundle_type: BundleType::new_bundle_read_write(),
            proc: Box::new(Block::new(Proc::new_proc_var("x"))),
            line_num: 0,
            col_num: 0,
        };
        let bound_inputs =
            proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", VarSort::ProcSort);

        let result = normalize_match_proc(&proc, inputs.clone(), &env);

        let expected_result = inputs
            .par
            .with_bundles(vec![Bundle {
                body: Some(new_boundvar_par(0, Vec::new(), false)),
                write_flag: true,
                read_flag: true,
            }])
            .with_locally_free(create_bit_vector(&vec![0]));

        //     //TODO fix assertions
        //     assert_eq!(result.clone().unwrap().par, expected_result);
        //     assert_eq!(result.clone().unwrap().free_map, bound_inputs.free_map);
        // }

        /** Example:
         * bundle { _ | x }
         */
        #[test]
        fn pbundle_should_throw_an_error_when_wildcard_or_free_variable_is_found_inside_body_of_bundle(
        ) {
            let (inputs, env) = proc_visit_inputs_and_env();
            let proc = Proc::Bundle {
                bundle_type: BundleType::new_bundle_read_write(),
                proc: Box::new(Block::new(Proc::new_proc_par_with_wildcard_and_var("x"))),
                line_num: 0,
                col_num: 0,
            };
            let result = normalize_match_proc(&proc, inputs.clone(), &env);
            assert!(matches!(
                result,
                Err(InterpreterError::UnexpectedBundleContent { .. })
            ));
        }

        /** Example:
         * bundle { Uri }
         */
        #[test]
        fn pbundle_should_throw_an_error_when_connective_is_used_at_top_level_of_body_of_bundle() {
            let (inputs, env) = proc_visit_inputs_and_env();
            let proc = Proc::Bundle {
                bundle_type: BundleType::new_bundle_read_write(),
                proc: Box::new(Block::new(Proc::SimpleType(SimpleType::new_uri()))),
                line_num: 0,
                col_num: 0,
            };
            let result = normalize_match_proc(&proc, inputs.clone(), &env);

            assert!(matches!(
                result,
                Err(InterpreterError::UnexpectedBundleContent { .. })
            ));
        }

        /** Example:
         * bundle { @Nil!(Uri) }
         */
        #[test]
        fn pbundle_should_not_throw_an_error_when_connective_is_used_outside_of_top_level_of_body_of_bundle(
        ) {
            let (inputs, env) = proc_visit_inputs_and_env();

            let proc = Proc::Bundle {
                bundle_type: BundleType::new_bundle_read_write(),
                proc: Box::new(Block::new(Proc::Send {
                    name: Name::new_name_quote_nil(),
                    send_type: SendType::new_single(),
                    inputs: ProcList::new(vec![Proc::SimpleType(SimpleType::new_uri())]),
                    line_num: 0,
                    col_num: 0,
                })),
                line_num: 0,
                col_num: 0,
            };
            let result = normalize_match_proc(&proc, inputs.clone(), &env);

            assert!(result.is_ok());
        }

        #[test]
        fn pbundle_should_interpret_bundle_polarization() {
            let (inputs, env) = proc_visit_inputs_and_env();
            //TODO should be implemented
        }

        #[test]
        fn pbundle_should_collapse_nested_bundles_merging_their_polarizations() {
            let (inputs, env) = proc_visit_inputs_and_env();

            let proc = Proc::Bundle {
                bundle_type: BundleType::new_bundle_read_write(),
                proc: Box::new(Block::new(Proc::Bundle {
                    bundle_type: BundleType::new_bundle_read(),
                    proc: Box::new(Block::new(Proc::new_proc_var("x"))),
                    line_num: 0,
                    col_num: 0,
                })),
                line_num: 0,
                col_num: 0,
            };

            let bound_inputs = proc_visit_inputs_with_updated_bound_map_chain(
                inputs.clone(),
                "x",
                VarSort::ProcSort,
            );

            let expected_result = inputs
                .par
                .with_bundles(vec![Bundle {
                    body: Some(new_boundvar_par(0, Vec::new(), false)),
                    write_flag: false,
                    read_flag: true,
                }])
                .with_locally_free(create_bit_vector(&vec![0]));

            let result = normalize_match_proc(&proc, inputs.clone(), &env);

            //TODO fix assertions
            //assert_eq!(result.clone().unwrap().par, expected_result);
            //assert_eq!(result.unwrap().free_map, bound_inputs.free_map);
        }
    }
}
