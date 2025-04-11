use super::normalize_match_proc;
use super::remainder_normalizer_matcher::handle_proc_remainder;
use crate::compiler::exports::{BoundMapChain, FreeMap};
use crate::compiler::rholang_ast::{AnnProc, KeyValuePair, ProcRemainder};
use crate::errors::InterpreterError;
use crate::normal_forms::{
    EListBody, EMapBody, ESetBody, ETupleBody, Expr, Par, union, union_inplace,
};
use crate::sort_matcher::Sortable;
use bitvec::vec::BitVec;
use std::collections::BTreeMap;
use std::result::Result;

pub fn normalize_c_list(
    elements: &[AnnProc],
    remainder: Option<ProcRemainder>,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
) -> Result<Expr, InterpreterError> {
    let mut list_body = EListBody {
        ps: Vec::with_capacity(elements.len()),
        ..Default::default()
    };
    for element in elements {
        let mut result_par = Par::default();
        normalize_match_proc(
            element.proc,
            &mut result_par,
            free_map,
            bound_map_chain,
            env,
            element.pos,
        )?;

        union_inplace(&mut list_body.locally_free, &result_par.locally_free);
        list_body.connective_used = list_body.connective_used || result_par.connective_used;
        list_body.ps.push(result_par);
    }

    if let Some(r) = remainder {
        let result = handle_proc_remainder(r, free_map)?;
        list_body.connective_used = true;
        list_body.remainder = Some(result);
    }

    Ok(Expr::EList(list_body))
}

pub fn normalize_c_tuple(
    elements: &[AnnProc],
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
) -> Result<Expr, InterpreterError> {
    let mut tuple_body = ETupleBody {
        ps: Vec::with_capacity(elements.len()),
        locally_free: BitVec::default(),
        connective_used: false,
    };
    for element in elements {
        let mut result_par = Par::default();
        normalize_match_proc(
            element.proc,
            &mut result_par,
            free_map,
            bound_map_chain,
            env,
            element.pos,
        )?;

        union_inplace(&mut tuple_body.locally_free, &result_par.locally_free);
        tuple_body.connective_used = tuple_body.connective_used || result_par.connective_used;
        tuple_body.ps.push(result_par);
    }

    Ok(Expr::ETuple(tuple_body))
}

pub fn normalize_c_set(
    elements: &[AnnProc],
    remainder: Option<ProcRemainder>,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
) -> Result<Expr, InterpreterError> {
    let mut set_body = ESetBody::default();
    for element in elements {
        let mut result_par = Par::default();
        normalize_match_proc(
            element.proc,
            &mut result_par,
            free_map,
            bound_map_chain,
            env,
            element.pos,
        )?;

        union_inplace(&mut set_body.locally_free, &result_par.locally_free);
        set_body.connective_used = set_body.connective_used || result_par.connective_used;
        set_body.ps.insert(result_par.sort_match());
    }

    if let Some(r) = remainder {
        let result = handle_proc_remainder(r, free_map)?;
        set_body.connective_used = true;
        set_body.remainder = Some(result);
    }

    Ok(Expr::ESet(set_body))
}

pub fn normalize_c_map(
    elements: &[KeyValuePair],
    remainder: Option<ProcRemainder>,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
) -> Result<Expr, InterpreterError> {
    let mut map_body = EMapBody::default();
    for KeyValuePair { key, value } in elements {
        let mut key_result = Par::default();
        let mut value_result = Par::default();

        normalize_match_proc(
            key.proc,
            &mut key_result,
            free_map,
            bound_map_chain,
            env,
            key.pos,
        )?;
        normalize_match_proc(
            value.proc,
            &mut value_result,
            free_map,
            bound_map_chain,
            env,
            value.pos,
        )?;

        union_inplace(
            &mut map_body.locally_free,
            union(&key_result.locally_free, &value_result.locally_free),
        );
        map_body.connective_used =
            map_body.connective_used || key_result.connective_used || value_result.connective_used;
        map_body
            .ps
            .insert(key_result.sort_match(), value_result.sort_match().term);
    }

    if let Some(r) = remainder {
        let result = handle_proc_remainder(r, free_map)?;
        map_body.connective_used = map_body.connective_used || result.connective_used();
        union_inplace(&mut map_body.locally_free, result.locally_free(0));
        map_body.remainder = Some(result);
    }

    Ok(Expr::EMap(map_body))
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/CollectMatcherSpec.scala
#[cfg(test)]
mod tests {

    use crate::compiler::Context;
    use crate::compiler::exports::SourcePosition;
    use crate::compiler::free_map::VarSort;
    use crate::compiler::rholang_ast::KeyValuePair;
    use crate::compiler::rholang_ast::{Collection, Id, Proc, ProcRemainder, Var};
    use crate::errors::InterpreterError;
    use crate::normal_forms::{
        EListBody, EMapBody, ESetBody, ETupleBody, Expr, Par, Var as NormalizedVar,
    };
    use crate::utils::test_utils::utils::assert_equal_normalized;
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::map;
    use crate::utils::test_utils::utils::set;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::{test_normalize_match_proc, with_bindings};
    use bitvec::vec::BitVec;
    use bitvec::{bitvec, order::Lsb0};
    use pretty_assertions::assert_eq;

    #[test]
    fn list_should_delegate() {
        // [P, *x, 7]

        let p = Id {
            name: "P",
            pos: SourcePosition { row: 0, column: 1 },
        };
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 5 },
        };
        let _7 = Proc::LongLiteral(7);
        let proc_p = p.as_proc();
        let eval_x = Proc::Eval {
            name: x.as_name().annotated(SourcePosition { row: 0, column: 5 }),
        };
        let proc = Proc::new_list(vec![
            proc_p.annotate(SourcePosition { row: 0, column: 1 }),
            eval_x.annotate(SourcePosition { row: 0, column: 4 }),
            _7.annotate(SourcePosition { row: 0, column: 8 }),
        ]);

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&p);
                bound_map.put_id_as_name(&x);
            }),
            test("expected to delegate", |actual_result, free_map| {
                let expected_result = Par {
                    exprs: vec![Expr::EList(EListBody {
                        ps: vec![Par::bound_var(1), Par::bound_var(0), Par::gint(7)],
                        locally_free: bitvec![1, 1],
                        ..Default::default()
                    })],
                    locally_free: bitvec![1, 1],
                    ..Default::default()
                };

                assert_eq!(actual_result, &expected_result);
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn list_should_sort_the_insides_ot_their_elements() {
        assert_equal_normalized("@0!([{1 | 2}])", "@0!([{2 | 1}])");
    }

    #[test]
    fn list_should_sort_the_insides_of_send_encoded_as_byte_array() {
        let rho1 = r#"
        new x in {
          x!(
            [
              @"a"!(
                @"x"!("abc") |
                @"y"!(1)
              )
            ].toByteArray()
          )
        }
    "#;

        let rho2 = r#"
        new x in {
          x!(
            [
              @"a"!(
                @"y"!(1) |
                @"x"!("abc")
              )
            ].toByteArray()
          )
        }
    "#;
        assert_equal_normalized(&rho1, &rho2);
    }

    #[test]
    fn tuple_should_delegate() {
        // (*y, Q)
        let q = Id {
            name: "Q",
            pos: SourcePosition { row: 0, column: 5 },
        }
        .as_proc();
        let y = Id {
            name: "y",
            pos: SourcePosition { row: 0, column: 2 },
        }
        .as_name();
        let eval_y = Proc::Eval {
            name: y.annotated(SourcePosition { row: 0, column: 2 }),
        };
        let proc = Proc::new_tuple(vec![
            eval_y.annotate(SourcePosition { row: 0, column: 1 }),
            q.annotate(SourcePosition { row: 0, column: 5 }),
        ]);

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to delegate", |actual_result, free_map| {
                let expected_result = Par {
                    exprs: vec![Expr::ETuple(ETupleBody {
                        ps: Par::free_vars(2),
                        locally_free: BitVec::EMPTY,
                        connective_used: false,
                    })],
                    ..Default::default()
                };

                assert_eq!(actual_result, &expected_result);
                assert_eq!(
                    free_map.as_free_vec(),
                    vec![
                        (
                            "y",
                            Context {
                                item: VarSort::NameSort,
                                source_position: SourcePosition { row: 0, column: 2 }
                            }
                        ),
                        (
                            "Q",
                            Context {
                                item: VarSort::ProcSort,
                                source_position: SourcePosition { row: 0, column: 5 }
                            }
                        )
                    ]
                )
            }),
        );
    }

    #[test]
    fn tuple_should_propagate_free_variables() {
        // (7, 7 | Q, Q)
        let q_1 = Id {
            name: "Q",
            pos: SourcePosition { row: 0, column: 8 },
        }
        .as_proc();
        let q_2 = Id {
            name: "Q",
            pos: SourcePosition { row: 0, column: 11 },
        }
        .as_proc();
        let _7 = Proc::LongLiteral(7);
        let _7_par_q = Proc::Par {
            left: _7.annotate(SourcePosition { row: 0, column: 4 }),
            right: q_1.annotate(SourcePosition { row: 0, column: 8 }),
        };
        let proc = Proc::new_tuple(vec![
            _7.annotate(SourcePosition { row: 0, column: 1 }),
            _7_par_q.annotate(SourcePosition { row: 0, column: 4 }),
            q_2.annotate(SourcePosition { row: 0, column: 11 }),
        ]);

        test_normalize_match_proc(&proc, defaults(), |actual_result| {
            assert_matches!(
            actual_result,
            Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                var_name,
                first_use: SourcePosition { row: 0, column: 8 },
                second_use: SourcePosition { row: 0, column: 11 }
            }) => {
                assert_eq!(var_name, "Q")
            });
        })
    }

    #[test]
    fn tuple_should_sort_the_insides_of_their_elements() {
        assert_equal_normalized("@0!(({1 | 2}))", "@0!(({2 | 1}))");
    }

    #[test]
    fn set_should_delegate() {
        // Set(P | R, 7, 8 | Q ... Z)
        let p = Id {
            name: "P",
            pos: SourcePosition { row: 0, column: 4 },
        };
        let r = Id {
            name: "R",
            pos: SourcePosition { row: 0, column: 8 },
        }
        .as_proc();
        let q = Id {
            name: "Q",
            pos: SourcePosition { row: 0, column: 18 },
        }
        .as_proc();

        let _7 = Proc::LongLiteral(7);
        let _8 = Proc::LongLiteral(8);

        let proc_p = p.as_proc();
        let p_par_r = Proc::Par {
            left: proc_p.annotate(SourcePosition { row: 0, column: 4 }),
            right: r.annotate(SourcePosition { row: 0, column: 8 }),
        };

        let _8_par_q = Proc::Par {
            left: _8.annotate(SourcePosition { row: 0, column: 14 }),
            right: q.annotate(SourcePosition { row: 0, column: 18 }),
        };

        let proc = Proc::Collection(Collection::Set {
            elements: vec![
                p_par_r.annotate(SourcePosition { row: 0, column: 4 }),
                _7.annotate(SourcePosition { row: 0, column: 11 }),
                _8_par_q.annotate(SourcePosition { row: 0, column: 14 }),
            ],
            cont: Some(ProcRemainder {
                var: Var::new_id("Z", SourcePosition { row: 0, column: 24 }),
                pos: SourcePosition { row: 0, column: 20 },
            }),
        });

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&p);
            }),
            test("expected to delegate", |actual_result, free_map| {
                let expected_result = Par {
                    exprs: vec![Expr::ESet(ESetBody {
                        ps: set(vec![
                            Par {
                                exprs: vec![Expr::new_bound_var(0), Expr::new_free_var(0)],
                                locally_free: bitvec![1],
                                ..Default::default()
                            }, /* P | R */
                            Par::gint(7),
                            Par {
                                exprs: vec![Expr::GInt(8), Expr::new_free_var(1)],
                                ..Default::default()
                            }, /* 8 | Q */
                        ]),
                        locally_free: bitvec![1],
                        remainder: Some(NormalizedVar::FreeVar(2)),
                        connective_used: true,
                    })],
                    locally_free: bitvec![1],
                    connective_used: true,
                    ..Default::default()
                };

                assert_eq!(actual_result, &expected_result);
                assert_eq!(
                    free_map.as_free_vec(),
                    vec![
                        (
                            "R",
                            Context {
                                item: VarSort::ProcSort,
                                source_position: SourcePosition { row: 0, column: 8 }
                            }
                        ),
                        (
                            "Q",
                            Context {
                                item: VarSort::ProcSort,
                                source_position: SourcePosition { row: 0, column: 19 }
                            }
                        ),
                        (
                            "Z",
                            Context {
                                item: VarSort::ProcSort,
                                source_position: SourcePosition { row: 0, column: 24 }
                            }
                        )
                    ]
                )
            }),
        );
    }

    #[test]
    fn set_should_sort_the_insides_of_their_elements() {
        assert_equal_normalized("@0!(Set({1 | 2}))", "@0!(Set({2 | 1}))");
    }

    #[test]
    fn map_should_delegate() {
        // { 7: "Seven", P: *Q ... Z }
        let _7 = Proc::LongLiteral(7);
        let seven = Proc::StringLiteral("Seven");
        let p = Id {
            name: "P",
            pos: SourcePosition { row: 0, column: 14 },
        };
        let q = Id {
            name: "Q",
            pos: SourcePosition { row: 0, column: 18 },
        }
        .as_name();

        let proc_p = p.as_proc();
        let eval_q = Proc::Eval {
            name: q.annotated(SourcePosition { row: 0, column: 18 }),
        };

        let proc = Proc::Collection(Collection::Map {
            pairs: vec![
                KeyValuePair {
                    key: _7.annotate(SourcePosition { row: 0, column: 2 }),
                    value: seven.annotate(SourcePosition { row: 0, column: 5 }),
                },
                KeyValuePair {
                    key: proc_p.annotate(SourcePosition { row: 0, column: 14 }),
                    value: eval_q.annotate(SourcePosition { row: 0, column: 17 }),
                },
            ],
            cont: Some(ProcRemainder {
                var: Var::new_id("Z", SourcePosition { row: 0, column: 24 }),
                pos: SourcePosition { row: 0, column: 20 },
            }),
        });

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&p);
            }),
            test("expected to delegate", |actual_result, free_map| {
                let expected_result = Par {
                    exprs: vec![Expr::EMap(EMapBody {
                        ps: map(vec![
                            (Par::gint(7), Par::gstr("Seven".to_string())),
                            (Par::bound_var(0), Par::free_var(0)),
                        ]),
                        locally_free: bitvec![1],
                        remainder: Some(NormalizedVar::FreeVar(1)),
                        connective_used: true,
                    })],
                    locally_free: bitvec![1],
                    connective_used: true,
                    ..Default::default()
                };

                assert_eq!(actual_result, &expected_result);
                assert_eq!(
                    free_map.as_free_vec(),
                    vec![
                        (
                            "Q",
                            Context {
                                item: VarSort::NameSort,
                                source_position: SourcePosition { row: 0, column: 18 }
                            }
                        ),
                        (
                            "Z",
                            Context {
                                item: VarSort::ProcSort,
                                source_position: SourcePosition { row: 0, column: 24 }
                            }
                        )
                    ]
                )
            }),
        );
    }

    #[test]
    fn map_should_sort_the_insides_of_their_keys() {
        assert_equal_normalized("@0!({{1 | 2} : 0})", "@0!({{2 | 1} : 0})")
    }

    #[test]
    fn map_should_sort_the_insides_of_their_values() {
        assert_equal_normalized("@0!({0 : {1 | 2}})", "@0!({0 : {2 | 1}})")
    }
}
