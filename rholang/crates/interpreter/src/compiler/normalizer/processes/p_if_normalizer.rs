use crate::aliases::EnvHashMap;
use crate::compiler::exports::{BoundMapChain, FreeMap};
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::AnnProc;
use crate::errors::InterpreterError;
use crate::normal_forms::{Match, MatchCase, Par, union};

pub fn normalize_p_if(
    value_body: AnnProc,
    true_body: AnnProc,
    false_body: Option<AnnProc>,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap,
) -> Result<(), InterpreterError> {
    let mut target_par = Par::default();
    normalize_match_proc(
        value_body.proc,
        &mut target_par,
        free_map,
        bound_map_chain,
        env,
        value_body.pos,
    )?;

    let mut if_true_par = Par::default();
    normalize_match_proc(
        true_body.proc,
        &mut if_true_par,
        free_map,
        bound_map_chain,
        env,
        true_body.pos,
    )?;

    let if_false_par = if let Some(body) = false_body {
        let mut par = Par::default();
        normalize_match_proc(
            body.proc,
            &mut par,
            free_map,
            bound_map_chain,
            env,
            body.pos,
        )?;

        par
    } else {
        Par::default()
    };

    // Construct the desugared if as a Match
    let freevec = union(
        union(&target_par.locally_free, &if_true_par.locally_free),
        &if_false_par.locally_free,
    );
    let connective_used =
        target_par.connective_used || if_true_par.connective_used || if_false_par.connective_used;
    let desugared_if = Match {
        target: target_par,
        cases: vec![
            MatchCase {
                pattern: Par::gtrue(),
                source: if_true_par,
                free_count: 0,
            },
            MatchCase {
                pattern: Par::gfalse(),
                source: if_false_par,
                free_count: 0,
            },
        ],
        locally_free: freevec,
        connective_used,
    };

    input_par.push_match(desugared_if);

    Ok(())
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::test_normalize_match_proc;
    use crate::utils::test_utils::utils::with_par;
    use bitvec::prelude::Lsb0;
    use bitvec::{bitvec, vec::BitVec};
    use pretty_assertions::assert_eq;

    use crate::{
        compiler::{
            exports::SourcePosition,
            rholang_ast::{GTRUE, Id, NIL, NameDecl, Proc, SendType},
        },
        normal_forms::{Expr, Match, MatchCase, New, Par, Send},
    };

    #[test]
    fn p_if_else_should_desugar_to_match_with_true_false_cases() {
        // if (true) { @Nil!(47) }

        let _47 = Proc::LongLiteral(47);
        let if_true = Proc::Send {
            name: NIL.quoted(),
            send_type: SendType::Single,
            inputs: vec![_47.annotate(SourcePosition { row: 0, column: 18 })],
        };

        let p_if = Proc::IfThenElse {
            condition: GTRUE.annotate(SourcePosition { row: 0, column: 4 }),
            if_true: if_true.annotate(SourcePosition { row: 0, column: 11 }),
            if_false: None,
        };

        test_normalize_match_proc(
            &p_if,
            defaults(),
            test(
                "expected to desugar to Match with true/false cases",
                |actual_par, free_map| {
                    let expected_par = Par {
                        matches: vec![Match {
                            target: Par::gtrue(),
                            cases: vec![
                                MatchCase {
                                    pattern: Par::gtrue(),
                                    source: Par {
                                        sends: vec![Send {
                                            chan: Par::NIL,
                                            data: vec![Par::gint(47)],
                                            ..Default::default()
                                        }],
                                        ..Default::default()
                                    },
                                    free_count: 0,
                                },
                                MatchCase {
                                    pattern: Par::gfalse(),
                                    source: Par::default(),
                                    free_count: 0,
                                },
                            ],
                            locally_free: BitVec::EMPTY.into(),
                            connective_used: false,
                        }],
                        ..Default::default()
                    };

                    assert!(free_map.is_empty());
                    assert_eq!(actual_par, &expected_par);
                },
            ),
        )
    }

    #[test]
    fn p_if_else_should_not_mix_par_from_the_input_with_normalized_one() {
        let _10 = Proc::LongLiteral(10);
        let p_if = Proc::IfThenElse {
            condition: GTRUE.annotate(SourcePosition { row: 0, column: 4 }),
            if_true: _10.annotate(SourcePosition { row: 0, column: 11 }),
            if_false: None,
        };

        test_normalize_match_proc(
            &p_if,
            with_par(Par::gint(7)),
            test(
                "expected not to mix a Par from the input with the normalized one",
                |actual_par, free_map| {
                    let expected_par = Par {
                        matches: vec![Match {
                            target: Par::gtrue(),
                            cases: vec![
                                MatchCase {
                                    pattern: Par::gtrue(),
                                    source: Par::gint(10),
                                    free_count: 0,
                                },
                                MatchCase {
                                    pattern: Par::gfalse(),
                                    source: Par::default(),
                                    free_count: 0,
                                },
                            ],
                            locally_free: BitVec::EMPTY.into(),
                            connective_used: false,
                        }],
                        exprs: vec![Expr::GInt(7)],
                        ..Default::default()
                    };

                    assert!(free_map.is_empty());
                    assert_eq!(actual_par, &expected_par);
                },
            ),
        )
    }

    #[test]
    fn p_if_else_should_handle_a_more_complicated_if_statement_with_an_else_clause() {
        // if (47 == 47) { new x in { x!(47) } } else { new y in { y!(47) } }
        let _47 = Proc::LongLiteral(47);
        let condition = Proc::Eq {
            left: _47.annotate(SourcePosition { row: 0, column: 4 }),
            right: _47.annotate(SourcePosition { row: 0, column: 10 }),
        };

        let send_in_then = Proc::Send {
            name: Id {
                name: "x",
                pos: SourcePosition { row: 0, column: 27 },
            }
            .as_name(),
            send_type: SendType::Single,
            inputs: vec![_47.annotate(SourcePosition { row: 0, column: 30 })],
        };
        let p_new_then = Proc::New {
            decls: vec![NameDecl::id("x", SourcePosition { row: 0, column: 20 })],
            proc: &send_in_then,
        };

        let send_in_else = Proc::Send {
            name: Id {
                name: "y",
                pos: SourcePosition { row: 0, column: 56 },
            }
            .as_name(),
            send_type: SendType::Single,
            inputs: vec![_47.annotate(SourcePosition { row: 0, column: 59 })],
        };
        let p_new_else = Proc::New {
            decls: vec![NameDecl::id("y", SourcePosition { row: 0, column: 49 })],
            proc: &send_in_else,
        };

        let p_if = Proc::IfThenElse {
            condition: condition.annotate(SourcePosition { row: 0, column: 4 }),
            if_true: p_new_then.annotate(SourcePosition { row: 0, column: 16 }),
            if_false: Some(p_new_else.annotate(SourcePosition { row: 0, column: 45 })),
        };

        test_normalize_match_proc(
            &p_if,
            defaults(),
            test(
                "expected to handle a more complicated if statement with an else case",
                |actual_par, free_map| {
                    let expected_par = Par {
                        matches: vec![Match {
                            target: Par {
                                exprs: vec![Expr::EEq(Par::gint(47), Par::gint(47))],
                                ..Default::default()
                            },
                            cases: vec![
                                MatchCase {
                                    pattern: Par::gtrue(),
                                    source: Par {
                                        news: vec![New {
                                            bind_count: 1,
                                            p: Par {
                                                sends: vec![Send {
                                                    chan: Par::bound_var(0),
                                                    data: vec![Par::gint(47)],
                                                    locally_free: bitvec![1].into(),
                                                    ..Default::default()
                                                }],
                                                locally_free: bitvec![1].into(),
                                                ..Default::default()
                                            },
                                            uris: vec![],
                                            locally_free: BitVec::EMPTY.into(),
                                        }],
                                        ..Default::default()
                                    },
                                    free_count: 0,
                                },
                                MatchCase {
                                    pattern: Par::gfalse(),
                                    source: Par {
                                        news: vec![New {
                                            bind_count: 1,
                                            p: Par {
                                                sends: vec![Send {
                                                    chan: Par::bound_var(0),
                                                    data: vec![Par::gint(47)],
                                                    locally_free: bitvec![1].into(),
                                                    ..Default::default()
                                                }],
                                                locally_free: bitvec![1].into(),
                                                ..Default::default()
                                            },
                                            uris: vec![],
                                            locally_free: BitVec::EMPTY.into(),
                                        }],
                                        ..Default::default()
                                    },
                                    free_count: 0,
                                },
                            ],
                            locally_free: BitVec::EMPTY.into(),
                            connective_used: false,
                        }],
                        exprs: vec![Expr::GInt(7)],
                        ..Default::default()
                    };

                    assert!(free_map.is_empty());
                    assert_eq!(actual_par, &expected_par);
                },
            ),
        );
    }
}
