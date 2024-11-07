use super::exports::*;
use crate::rust::interpreter::compiler::bound_context::BoundContext;
use crate::rust::interpreter::compiler::exports::FreeContext;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, VarSort};
use crate::rust::interpreter::compiler::rholang_ast::{Name, Proc, Quote, Var};
use crate::rust::interpreter::compiler::source_position::SourcePosition;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;
use models::rhoapi::{expr, var, EVar, Expr, Par, Var as model_var};
use std::collections::HashMap;

pub fn normalize_name(
    proc: &Name,
    mut input: NameVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<NameVisitOutputs, InterpreterError> {
    match proc {
        Name::ProcVar(boxed_proc) => match *boxed_proc.clone() {
            Proc::Wildcard { line_num, col_num } => {
                let wildcard_bind_result = input.free_map.add_wildcard(SourcePosition {
                    row: line_num,
                    column: col_num,
                });

                let new_expr = Expr {
                    expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
                        v: Some(model_var {
                            var_instance: Some(var::VarInstance::Wildcard(var::WildcardMsg {})),
                        }),
                    })),
                };

                Ok(NameVisitOutputs {
                    par: prepend_expr(
                        Par::default(),
                        new_expr,
                        input.bound_map_chain.depth() as i32,
                    ),
                    free_map: wildcard_bind_result,
                })
            }

            Proc::Var(Var {
                name,
                line_num,
                col_num,
            }) => match input.bound_map_chain.get(&name) {
                Some(bound_context) => match bound_context {
                    BoundContext {
                        index: level,
                        typ: VarSort::NameSort,
                        ..
                    } => {
                        let new_expr = Expr {
                            expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
                                v: Some(model_var {
                                    var_instance: Some(var::VarInstance::BoundVar(level as i32)),
                                }),
                            })),
                        };

                        return Ok(NameVisitOutputs {
                            par: prepend_expr(
                                Par::default(),
                                new_expr,
                                input.bound_map_chain.depth() as i32,
                            ),
                            free_map: input.free_map.clone(),
                        });
                    }
                    BoundContext {
                        typ: VarSort::ProcSort,
                        source_position,
                        ..
                    } => {
                        return Err(InterpreterError::UnexpectedNameContext {
                            var_name: name.to_string(),
                            proc_var_source_position: source_position.to_string(),
                            name_source_position: SourcePosition {
                                row: line_num,
                                column: col_num,
                            }
                            .to_string(),
                        }
                        .into());
                    }
                },
                None => match input.free_map.get(&name) {
                    None => {
                        let updated_free_map = input.free_map.put((
                            name.to_string(),
                            VarSort::NameSort,
                            SourcePosition {
                                row: line_num,
                                column: col_num,
                            },
                        ));
                        let new_expr = Expr {
                            expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
                                v: Some(model_var {
                                    var_instance: Some(var::VarInstance::FreeVar(
                                        input.free_map.next_level as i32,
                                    )),
                                }),
                            })),
                        };

                        Ok(NameVisitOutputs {
                            par: prepend_expr(
                                Par::default(),
                                new_expr,
                                input.bound_map_chain.depth() as i32,
                            ),
                            free_map: updated_free_map,
                        })
                    }
                    Some(FreeContext {
                        source_position, ..
                    }) => Err(InterpreterError::UnexpectedReuseOfNameContextFree {
                        var_name: name.to_string(),
                        first_use: source_position.to_string(),
                        second_use: SourcePosition {
                            row: line_num,
                            column: col_num,
                        }
                        .to_string(),
                    }
                    .into()),
                },
            },

            _ => Err(InterpreterError::BugFoundError(format!(
                "Expected Proc::Var or Proc::Wildcard, found {:?}",
                boxed_proc
            ))),
        },

        Name::Quote(boxed_quote) => {
            let Quote { ref quotable, .. } = *boxed_quote.clone();
            let proc_visit_result = normalize_match_proc(
                &quotable,
                ProcVisitInputs {
                    par: Par::default(),
                    bound_map_chain: input.bound_map_chain.clone(),
                    free_map: input.free_map.clone(),
                },
                env,
            )?;

            Ok(NameVisitOutputs {
                par: proc_visit_result.par,
                free_map: proc_visit_result.free_map,
            })
        }
    }
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/NameMatcherSpec.scala
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::interpreter::compiler::rholang_ast::Name;
    use crate::rust::interpreter::compiler::test_utils::name_visit_inputs_and_env;
    use models::create_bit_vector;
    use models::rust::utils::{new_boundvar_par, new_freevar_par, new_gint_par, new_wildcard_par};

    fn bound_name_inputs_with_bound_map_chain(
        input: NameVisitInputs,
        name: &str,
        v_type: VarSort,
        line_num: usize,
        col_num: usize,
    ) -> NameVisitInputs {
        NameVisitInputs {
            bound_map_chain: {
                let updated_bound_map_chain = input.bound_map_chain.put((
                    name.to_string(),
                    v_type,
                    SourcePosition {
                        row: line_num,
                        column: col_num,
                    },
                ));
                updated_bound_map_chain
            },
            ..input.clone()
        }
    }

    fn bound_name_inputs_with_free_map(
        input: NameVisitInputs,
        name: &str,
        v_type: VarSort,
        line_num: usize,
        col_num: usize,
    ) -> NameVisitInputs {
        NameVisitInputs {
            free_map: {
                let updated_free_map = input.clone().free_map.put((
                    name.to_string(),
                    v_type,
                    SourcePosition {
                        row: line_num,
                        column: col_num,
                    },
                ));
                updated_free_map
            },
            ..input.clone()
        }
    }

    #[test]
    fn name_wildcard_should_add_a_wildcard_count_to_known_free() {
        let nw = Name::new_name_wildcard(1, 1);
        let (input, env) = name_visit_inputs_and_env();

        let result = normalize_name(&nw, input, &env);
        let expected_result = new_wildcard_par(Vec::new(), true);

        let unwrap_result = result.clone().unwrap();
        assert_eq!(unwrap_result.clone().par, expected_result);
        assert_eq!(unwrap_result.clone().free_map.count(), 1);
    }

    #[test]
    fn name_var_should_compile_as_bound_var_if_its_in_env() {
        let (input, env) = name_visit_inputs_and_env();
        let n_var: Name = Name::new_name_var("x", 1, 1);
        let bound_inputs =
            bound_name_inputs_with_bound_map_chain(input.clone(), "x", VarSort::NameSort, 0, 0);

        let result = normalize_name(&n_var, bound_inputs.clone(), &env);
        let expected_result = new_boundvar_par(0, create_bit_vector(&vec![0]), false);

        let unwrap_result: NameVisitOutputs = result.clone().unwrap();
        println!("Rust BoundVar result par: {:?}", unwrap_result.clone().par);
        assert_eq!(unwrap_result.par, expected_result);
        assert_eq!(
            unwrap_result.clone().free_map,
            bound_inputs.clone().free_map
        );
    }

    #[test]
    fn name_var_should_compile_as_free_var_if_its_not_in_env() {
        let n_var: Name = Name::new_name_var("x", 1, 1);
        let (input, env) = name_visit_inputs_and_env();

        let result = normalize_name(&n_var, input.clone(), &env);
        let expected_result = new_freevar_par(0, Vec::new());

        let unwrap_result = result.clone().unwrap();
        assert_eq!(unwrap_result.par, expected_result);
        let bound_inputs =
            bound_name_inputs_with_free_map(input.clone(), "x", VarSort::NameSort, 1, 1);
        assert_eq!(result.unwrap().free_map, bound_inputs.free_map);
    }

    #[test]
    fn name_var_should_not_compile_if_its_in_env_of_wrong_sort() {
        let (input, env) = name_visit_inputs_and_env();
        let n_var: Name = Name::new_name_var("x", 1, 1);
        let bound_inputs =
            bound_name_inputs_with_bound_map_chain(input.clone(), "x", VarSort::ProcSort, 0, 0);

        let result = normalize_name(&n_var, bound_inputs, &env);
        assert!(matches!(
            result,
            Err(InterpreterError::UnexpectedNameContext { .. })
        ));
    }

    #[test]
    fn name_var_should_not_compile_if_used_free_somewhere_else() {
        let (input, env) = name_visit_inputs_and_env();
        let n_var: Name = Name::new_name_var("x", 1, 1);
        let bound_inputs =
            bound_name_inputs_with_free_map(input.clone(), "x", VarSort::NameSort, 0, 0);

        let result = normalize_name(&n_var, bound_inputs, &env);
        assert!(matches!(
            result,
            Err(InterpreterError::UnexpectedReuseOfNameContextFree { .. })
        ));
    }

    #[test]
    fn name_quote_should_compile_to_bound_var() {
        let n_q_var = Name::new_name_quote_var("x", 1, 1);
        let (input, env) = name_visit_inputs_and_env();
        let bound_inputs =
            bound_name_inputs_with_bound_map_chain(input.clone(), "x", VarSort::ProcSort, 0, 0);

        let result = normalize_name(&n_q_var, bound_inputs.clone(), &env);
        let expected_result: Par = new_boundvar_par(0, create_bit_vector(&vec![0]), false);

        let unwrap_result = result.clone().unwrap();
        println!("Rust Quote BoundVar result par: {:?}", unwrap_result.par);
        assert_eq!(unwrap_result.clone().par, expected_result);
        assert_eq!(unwrap_result.clone().free_map, bound_inputs.free_map);
    }

    #[test]
    fn name_quote_should_return_a_free_use_if_the_quoted_proc_has_a_free_var() {
        let n_q_var = Name::new_name_quote_var("x", 1, 1);
        let (input, env) = name_visit_inputs_and_env();

        let result = normalize_name(&n_q_var, input.clone(), &env);
        let expected_result = new_freevar_par(0, Vec::new());

        let unwrap_result = result.clone().unwrap();
        assert_eq!(unwrap_result.clone().par, expected_result);

        let bound_inputs =
            bound_name_inputs_with_free_map(input.clone(), "x", VarSort::ProcSort, 1, 1);
        assert_eq!(unwrap_result.clone().free_map, bound_inputs.free_map);
    }

    #[test]
    fn name_quote_should_compile_to_a_ground() {
        let n_q_ground = Name::new_name_quote_ground_long_literal(7, 0, 0);
        let (input, env) = name_visit_inputs_and_env();

        let result = normalize_name(&n_q_ground, input.clone(), &env);
        let expected_result = new_gint_par(7, Vec::new(), false);

        let unwrap_result = result.clone().unwrap();

        assert_eq!(unwrap_result.clone().par, expected_result);
        assert_eq!(unwrap_result.clone().free_map, input.free_map);
    }

    #[test]
    fn name_quote_should_collapse_an_eval() {
        let n_q_eval = Name::new_name_quote_eval("x", 0, 0);
        let (input, env) = name_visit_inputs_and_env();
        let bound_inputs =
            bound_name_inputs_with_bound_map_chain(input.clone(), "x", VarSort::NameSort, 0, 0);

        let result = normalize_name(&n_q_eval, bound_inputs.clone(), &env);
        let expected_result = new_boundvar_par(0, create_bit_vector(&vec![0]), false);

        let unwrap_result = result.clone().unwrap();
        assert_eq!(unwrap_result.clone().par, expected_result);
        assert_eq!(unwrap_result.clone().free_map, bound_inputs.free_map);
    }

    #[test]
    fn name_quote_should_not_collapse_an_eval_eval() {
        let n_q_eval = Name::new_name_quote_par_of_evals("x", 0, 0);
        let (input, env) = name_visit_inputs_and_env();
        let bound_inputs =
            bound_name_inputs_with_bound_map_chain(input.clone(), "x", VarSort::NameSort, 0, 0);

        let result = normalize_name(&n_q_eval, bound_inputs.clone(), &env);

        //TODO not sure here -> val expectedResult: Par = EVar(BoundVar(0)).prepend(EVar(BoundVar(0)), 0)
        let bound_var_expr = new_boundvar_par(0, create_bit_vector(&vec![0]), false);
        let expected_result =
            prepend_expr(bound_var_expr.clone(), bound_var_expr.exprs[0].clone(), 0);

        let unwrap_result = result.clone().unwrap();
        assert_eq!(unwrap_result.clone().par, expected_result);
        assert_eq!(unwrap_result.clone().free_map, bound_inputs.free_map);
    }
}
