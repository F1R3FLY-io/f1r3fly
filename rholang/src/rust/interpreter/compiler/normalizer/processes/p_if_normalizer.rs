use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{
    normalize_match_proc, ProcVisitInputs, ProcVisitOutputs,
};
use crate::rust::interpreter::errors::InterpreterError;
use f1r3fly_models::rhoapi::{Match, MatchCase, Par};
use f1r3fly_models::rust::utils::{new_gbool_par, union};
use std::collections::HashMap;

pub fn normalize_p_if(
    value_proc: &Proc,
    true_body_proc: &Proc,
    false_body_proc: &Proc,
    mut input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let target_result =
        normalize_match_proc(&value_proc, ProcVisitInputs { ..input.clone() }, env)?;

    let true_case_body = normalize_match_proc(
        &true_body_proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: target_result.free_map.clone(),
        },
        env,
    )?;

    let false_case_body = normalize_match_proc(
        &false_body_proc,
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: input.bound_map_chain.clone(),
            free_map: true_case_body.free_map.clone(),
        },
        env,
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

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};

    use f1r3fly_models::{
        create_bit_vector,
        rhoapi::{expr::ExprInstance, EEq, Expr, Match, MatchCase, Par, Send},
        rust::utils::{
            new_boundvar_par, new_gbool_par, new_gint_expr, new_gint_par, new_new_par, new_send_par,
        },
    };

    use crate::rust::interpreter::{
        compiler::{
            normalize::{normalize_match_proc, ProcVisitInputs},
            rholang_ast::{Decls, Name, NameDecl, Proc, ProcList, SendType},
        },
        test_utils::utils::proc_visit_inputs_and_env,
    };

    #[test]
    fn p_if_else_should_desugar_to_match_with_true_false_cases() {
        // if (true) { @Nil!(47) }

        let p_if = Proc::IfElse {
            condition: Box::new(Proc::BoolLiteral {
                value: true,
                line_num: 0,
                col_num: 0,
            }),
            if_true: Box::new(Proc::Send {
                name: Name::new_name_quote_nil(),
                send_type: SendType::new_single(),
                inputs: ProcList::new(vec![Proc::new_proc_int(47)]),
                line_num: 0,
                col_num: 0,
            }),
            alternative: None,
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&p_if, ProcVisitInputs::new(), &HashMap::new());
        assert!(result.is_ok());

        let expected_result = Par::default().prepend_match(Match {
            target: Some(new_gbool_par(true, Vec::new(), false)),
            cases: vec![
                MatchCase {
                    pattern: Some(new_gbool_par(true, Vec::new(), false)),
                    source: Some(Par::default().with_sends(vec![Send {
                        chan: Some(Par::default()),
                        data: vec![new_gint_par(47, Vec::new(), false)],
                        persistent: false,
                        locally_free: Vec::new(),
                        connective_used: false,
                    }])),
                    free_count: 0,
                },
                MatchCase {
                    pattern: Some(new_gbool_par(false, Vec::new(), false)),
                    source: Some(Par::default()),
                    free_count: 0,
                },
            ],
            locally_free: Vec::new(),
            connective_used: false,
        });

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, ProcVisitInputs::new().free_map);
    }

    #[test]
    fn p_if_else_should_not_mix_par_from_the_input_with_normalized_one() {
        let p_if = Proc::IfElse {
            condition: Box::new(Proc::BoolLiteral {
                value: true,
                line_num: 0,
                col_num: 0,
            }),
            if_true: Box::new(Proc::new_proc_int(10)),
            alternative: None,
            line_num: 0,
            col_num: 0,
        };

        let (mut inputs, env) = proc_visit_inputs_and_env();
        inputs.par = Par::default().with_exprs(vec![new_gint_expr(7)]);

        let result = normalize_match_proc(&p_if, inputs.clone(), &env);
        assert!(result.is_ok());

        let expected_result = Par::default()
            .with_matches(vec![Match {
                target: Some(new_gbool_par(true, Vec::new(), false)),
                cases: vec![
                    MatchCase {
                        pattern: Some(new_gbool_par(true, Vec::new(), false)),
                        source: Some(new_gint_par(10, Vec::new(), false)),
                        free_count: 0,
                    },
                    MatchCase {
                        pattern: Some(new_gbool_par(false, Vec::new(), false)),
                        source: Some(Par::default()),
                        free_count: 0,
                    },
                ],
                locally_free: Vec::new(),
                connective_used: false,
            }])
            .with_exprs(vec![new_gint_expr(7)]);

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_if_else_should_handle_a_more_complicated_if_statement_with_an_else_clause() {
        // if (47 == 47) { new x in { x!(47) } } else { new y in { y!(47) } }
        let condition = Proc::Eq {
            left: Box::new(Proc::new_proc_int(47)),
            right: Box::new(Proc::new_proc_int(47)),
            line_num: 0,
            col_num: 0,
        };

        let p_new_if = Proc::New {
            decls: Decls {
                decls: vec![NameDecl::new("x", None)],
                line_num: 0,
                col_num: 0,
            },
            proc: Box::new(Proc::Send {
                name: Name::new_name_var("x"),
                send_type: SendType::new_single(),
                inputs: ProcList::new(vec![Proc::new_proc_int(47)]),
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        };

        let p_new_else = Proc::New {
            decls: Decls {
                decls: vec![NameDecl::new("y", None)],
                line_num: 0,
                col_num: 0,
            },
            proc: Box::new(Proc::Send {
                name: Name::new_name_var("y"),
                send_type: SendType::new_single(),
                inputs: ProcList::new(vec![Proc::new_proc_int(47)]),
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        };

        let p_if = Proc::IfElse {
            condition: Box::new(condition),
            if_true: Box::new(p_new_if),
            alternative: Some(Box::new(p_new_else)),
            line_num: 0,
            col_num: 0,
        };

        let (inputs, env) = proc_visit_inputs_and_env();
        let result = normalize_match_proc(&p_if, inputs.clone(), &env);
        assert!(result.is_ok());

        let expected_result = Par::default().with_matches(vec![Match {
            target: Some(Par::default().with_exprs(vec![Expr {
                expr_instance: Some(ExprInstance::EEqBody(EEq {
                    p1: Some(new_gint_par(47, Vec::new(), false)),
                    p2: Some(new_gint_par(47, Vec::new(), false)),
                })),
            }])),
            cases: vec![
                MatchCase {
                    pattern: Some(new_gbool_par(true, Vec::new(), false)),
                    source: Some(new_new_par(
                        1,
                        new_send_par(
                            new_boundvar_par(0, create_bit_vector(&vec![0]), false),
                            vec![new_gint_par(47, Vec::new(), false)],
                            false,
                            create_bit_vector(&vec![0]),
                            false,
                            create_bit_vector(&vec![0]),
                            false,
                        ),
                        vec![],
                        BTreeMap::new(),
                        Vec::new(),
                        Vec::new(),
                        false,
                    )),
                    free_count: 0,
                },
                MatchCase {
                    pattern: Some(new_gbool_par(false, Vec::new(), false)),
                    source: Some(new_new_par(
                        1,
                        new_send_par(
                            new_boundvar_par(0, create_bit_vector(&vec![0]), false),
                            vec![new_gint_par(47, Vec::new(), false)],
                            false,
                            create_bit_vector(&vec![0]),
                            false,
                            create_bit_vector(&vec![0]),
                            false,
                        ),
                        vec![],
                        BTreeMap::new(),
                        Vec::new(),
                        Vec::new(),
                        false,
                    )),
                    free_count: 0,
                },
            ],
            locally_free: Vec::new(),
            connective_used: false,
        }]);

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }
}
