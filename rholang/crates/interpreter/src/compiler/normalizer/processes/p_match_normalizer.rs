use crate::compiler::exports::{BoundMapChain, FreeMap};
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::{AnnProc, Case};
use crate::errors::InterpreterError;
use crate::normal_forms::{Match, MatchCase, Par, adjust_bitset, union_inplace};

use std::collections::BTreeMap;

pub fn normalize_p_match(
    expression: AnnProc,
    cases: &[Case],
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
) -> Result<(), InterpreterError> {
    let mut target_par = Par::default();
    normalize_match_proc(
        expression.proc,
        &mut target_par,
        free_map,
        bound_map_chain,
        env,
        expression.pos,
    )?;

    let connective_used = target_par.connective_used;
    let locally_free = target_par.locally_free.clone();
    let mut target_match = Match {
        target: target_par,
        cases: Vec::with_capacity(cases.len()),
        locally_free,
        connective_used,
    };
    for case in cases {
        let pattern = case.pattern;
        let case_body = case.proc;

        let mut pattern_par = Par::default();
        let mut temp_free_map = FreeMap::new();
        bound_map_chain.descend(|bound_map| {
            normalize_match_proc(
                pattern.proc,
                &mut pattern_par,
                &mut temp_free_map,
                bound_map,
                env,
                pattern.pos,
            )
        })?;

        let mut case_body_par = Par::default();
        let bound_count = bound_map_chain.extend(temp_free_map, |bound_count, case_env| {
            normalize_match_proc(
                case_body.proc,
                &mut case_body_par,
                free_map,
                case_env,
                env,
                case_body.pos,
            )?;
            Ok::<u32, InterpreterError>(bound_count)
        })?;

        target_match.connective_used =
            target_match.connective_used || case_body_par.connective_used;
        union_inplace(&mut target_match.locally_free, &pattern_par.locally_free);
        union_inplace(
            &mut target_match.locally_free,
            adjust_bitset(&case_body_par.locally_free, bound_count as usize),
        );

        target_match.cases.push(MatchCase {
            pattern: pattern_par,
            source: case_body_par,
            free_count: bound_count,
        });
    }

    input_par.push_match(target_match);

    Ok(())
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::test_normalize_match_proc;
    use crate::utils::test_utils::utils::with_bindings;
    use crate::{
        compiler::{
            exports::SourcePosition,
            rholang_ast::{
                Case, Id, LinearBind, NIL, Names, Proc, Receipt, SendType, Source, WILD,
            },
        },
        errors::InterpreterError,
        normal_forms::{EListBody, Expr, Match, MatchCase, Par, Receive, ReceiveBind, Send},
    };
    use bitvec::{bitvec, order::Lsb0, vec::BitVec};
    use pretty_assertions::assert_eq;

    #[test]
    fn p_match_should_handle_a_match_inside_a_for_comprehension() {
        // for (@x <- @Nil) { match x { case 42 => Nil ; case y => Nil } | @Nil!(47) }
        let x = Proc::new_var("x", SourcePosition { row: 0, column: 6 });
        let receipts = vec![Receipt::Linear(vec![LinearBind {
            lhs: Names::single(x.quoted().annotated(SourcePosition { row: 0, column: 5 })),
            rhs: Source::Simple {
                name: NIL
                    .quoted()
                    .annotated(SourcePosition { row: 0, column: 11 }),
            },
        }])];

        let inner_x = Proc::new_var("x", SourcePosition { row: 0, column: 25 });
        let y = Proc::new_var("y", SourcePosition { row: 0, column: 51 });
        let _42 = Proc::LongLiteral(42);
        let p_match = Proc::Match {
            expression: inner_x.annotate(SourcePosition { row: 0, column: 25 }),
            cases: vec![
                Case {
                    pattern: _42.annotate(SourcePosition { row: 0, column: 34 }),
                    proc: NIL.annotate(SourcePosition { row: 0, column: 40 }),
                },
                Case {
                    pattern: y.annotate(SourcePosition { row: 0, column: 51 }),
                    proc: NIL.annotate(SourcePosition { row: 0, column: 56 }),
                },
            ],
        };

        let _47 = Proc::LongLiteral(47);
        let p_send = Proc::Send {
            name: NIL.quoted(),
            send_type: SendType::Single,
            inputs: vec![_47.annotate(SourcePosition { row: 0, column: 70 })],
        };

        let body = Proc::Par {
            left: p_match.annotate(SourcePosition { row: 0, column: 19 }),
            right: p_send.annotate(SourcePosition { row: 0, column: 64 }),
        };

        let p_par = Proc::ForComprehension {
            receipts,
            proc: body.annotate(SourcePosition { row: 0, column: 17 }),
        };

        test_normalize_match_proc(
            &p_par,
            defaults(),
            test(
                "expected to handle match inside for-comprehension",
                |actual_par, free_map| {
                    let expected_par = Par {
                        sends: vec![Send {
                            chan: Par::NIL,
                            data: vec![Par::gint(47)],
                            ..Default::default()
                        }],
                        receives: vec![Receive {
                            binds: vec![ReceiveBind {
                                patterns: vec![Par::free_var(0)],
                                source: Par::NIL,
                                remainder: None,
                                free_count: 1,
                            }],
                            body: Par {
                                matches: vec![Match {
                                    target: Par::bound_var(0),
                                    cases: vec![
                                        MatchCase {
                                            pattern: Par::gint(42),
                                            source: Par::NIL,
                                            free_count: 0,
                                        },
                                        MatchCase {
                                            pattern: Par::free_var(0),
                                            source: Par::NIL,
                                            free_count: 1,
                                        },
                                    ],
                                    locally_free: bitvec![1],
                                    connective_used: false,
                                }],
                                locally_free: bitvec![1],
                                ..Default::default()
                            },
                            persistent: false,
                            peek: false,
                            bind_count: 1,
                            locally_free: BitVec::EMPTY,
                            connective_used: false,
                        }],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert!(free_map.is_empty());
                },
            ),
        )
    }

    #[test]
    fn p_match_should_have_a_free_count_of_1_if_the_case_contains_a_wildcard_and_a_free_variable() {
        // match x { case [y, _] => Nil ; case _ => Nil }
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 6 },
        };
        let y = Proc::new_var("y", SourcePosition { row: 0, column: 16 });

        let proc_x = x.as_proc();
        let pattern_1 = Proc::new_list(vec![
            y.annotate(SourcePosition { row: 0, column: 16 }),
            WILD.annotate(SourcePosition { row: 0, column: 19 }),
        ]);

        let p_match = Proc::Match {
            expression: proc_x.annotate(SourcePosition { row: 0, column: 6 }),
            cases: vec![
                Case {
                    pattern: pattern_1.annotate(SourcePosition { row: 0, column: 15 }),
                    proc: NIL.annotate(SourcePosition { row: 0, column: 25 }),
                },
                Case {
                    pattern: WILD.annotate(SourcePosition { row: 0, column: 36 }),
                    proc: NIL.annotate(SourcePosition { row: 0, column: 41 }),
                },
            ],
        };

        test_normalize_match_proc(
            &p_match,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test(
                "expected to have free_count = 1 if a case contains one free variable",
                |actual_result, _| {
                    let expected_result = Par {
                        matches: vec![Match {
                            target: Par::bound_var(0),
                            cases: vec![
                                MatchCase {
                                    pattern: Par {
                                        exprs: vec![Expr::EList(EListBody {
                                            ps: vec![Par::free_var(0), Par::wild()],
                                            connective_used: true,
                                            ..Default::default()
                                        })],
                                        connective_used: true,
                                        ..Default::default()
                                    },
                                    source: Par::NIL,
                                    free_count: 1,
                                },
                                MatchCase {
                                    pattern: Par::wild(),
                                    source: Par::NIL,
                                    free_count: 0,
                                },
                            ],
                            locally_free: bitvec![1],
                            connective_used: false,
                        }],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert_eq!(actual_result, &expected_result);
                },
            ),
        );
    }

    #[test]
    fn p_match_should_fail_if_a_free_variable_is_used_twice_in_the_target() {
        // match 47 { case (y | y) => Nil }
        let _47 = Proc::LongLiteral(47);
        let y_1 = Proc::new_var("y", SourcePosition { row: 0, column: 17 });
        let y_2 = Proc::new_var("y", SourcePosition { row: 0, column: 21 });

        let pattern = Proc::Par {
            left: y_1.annotate(SourcePosition { row: 0, column: 17 }),
            right: y_2.annotate(SourcePosition { row: 0, column: 21 }),
        };

        let p_match = Proc::Match {
            expression: _47.annotate(SourcePosition { row: 0, column: 6 }),
            cases: vec![Case {
                pattern: pattern.annotate(SourcePosition { row: 0, column: 11 }),
                proc: NIL.annotate(SourcePosition { row: 0, column: 27 }),
            }],
        };

        test_normalize_match_proc(&p_match, defaults(), |actual_result| {
            assert_matches!(
            actual_result,
            Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                var_name,
                first_use: SourcePosition { row: 0, column: 17 },
                second_use: SourcePosition { row: 0, column: 21 }
            }) => {
                assert_eq!(var_name, "y")
            });
        })
    }

    #[test]
    fn p_match_should_handle_a_match_inside_a_for_pattern() {
        // for (@{match {x | y} { 47 => Nil }} <- @Nil) { Nil }
        let x = Proc::new_var("x", SourcePosition { row: 0, column: 14 });
        let y = Proc::new_var("y", SourcePosition { row: 0, column: 18 });
        let _47 = Proc::LongLiteral(47);
        let x_par_y = Proc::Par {
            left: x.annotate(SourcePosition { row: 0, column: 14 }),
            right: y.annotate(SourcePosition { row: 0, column: 18 }),
        };

        let p_match = Proc::Match {
            expression: x_par_y.annotate(SourcePosition { row: 0, column: 13 }),
            cases: vec![Case {
                pattern: _47.annotate(SourcePosition { row: 0, column: 23 }),
                proc: NIL.annotate(SourcePosition { row: 0, column: 29 }),
            }],
        };

        let receipts = vec![Receipt::Linear(vec![LinearBind {
            lhs: Names::single(
                p_match
                    .quoted()
                    .annotated(SourcePosition { row: 0, column: 5 }),
            ),
            rhs: Source::Simple {
                name: NIL
                    .quoted()
                    .annotated(SourcePosition { row: 0, column: 39 }),
            },
        }])];

        let p_input = Proc::ForComprehension {
            receipts,
            proc: NIL.annotate(SourcePosition { row: 0, column: 47 }),
        };

        test_normalize_match_proc(
            &p_input,
            defaults(),
            test(
                "expected to handle match inside pattern",
                |actual_par, free_map| {
                    let expected_par = Par {
                        receives: vec![Receive {
                            binds: vec![ReceiveBind {
                                patterns: vec![Par {
                                    matches: vec![Match {
                                        target: Par {
                                            exprs: vec![
                                                Expr::new_free_var(0),
                                                Expr::new_free_var(1),
                                            ],
                                            connective_used: true,
                                            ..Default::default()
                                        },
                                        cases: vec![MatchCase {
                                            pattern: Par::gint(47),
                                            source: Par::NIL,
                                            free_count: 0,
                                        }],
                                        locally_free: BitVec::EMPTY,
                                        connective_used: true,
                                    }],
                                    connective_used: true,
                                    ..Default::default()
                                }],
                                source: Par::NIL,
                                remainder: None,
                                free_count: 2,
                            }],
                            body: Par::NIL,
                            persistent: false,
                            peek: false,
                            bind_count: 2,
                            locally_free: BitVec::EMPTY,
                            connective_used: false,
                        }],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert!(free_map.is_empty());
                },
            ),
        );
    }
}
