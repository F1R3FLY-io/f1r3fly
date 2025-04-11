use std::collections::BTreeMap;

use crate::compiler::exports::{BoundMapChain, FreeMap};
use crate::compiler::free_map::ConnectiveInstance;
use crate::compiler::normalizer::name_normalize_matcher::normalize_name;
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::normalizer::remainder_normalizer_matcher::handle_name_remainder;
use crate::compiler::rholang_ast::{AnnName, AnnProc, Names};
use crate::errors::InterpreterError;
use crate::normal_forms::{Par, Receive, ReceiveBind, adjust_bitset, union_inplace};

pub fn normalize_p_contr(
    name: AnnName,
    formals: Names,
    body: AnnProc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
) -> Result<(), InterpreterError> {
    let contract_depth = bound_map_chain.depth();
    let name_match_par = normalize_name(name.0, free_map, bound_map_chain, env, name.1)?;

    // A free variable can only be used once in any of the parameters. And we start with the empty
    // free variable map because these free variables aren't free in the surrounding context:
    // they're binders
    let mut binders = Vec::with_capacity(formals.names.len());
    let mut freevec = name_match_par.locally_free.clone();
    let mut binders_free_map = FreeMap::new();
    for n in formals.names {
        let binder: Par = bound_map_chain
            .descend(|bound_map| normalize_name(n.0, &mut binders_free_map, bound_map, env, n.1))?;

        union_inplace(&mut freevec, &binder.locally_free);
        binders.push(binder);
    }
    if contract_depth == 0 {
        if let Some(invalid) = binders_free_map
            .iter_connectives()
            .find(|c| c.item == ConnectiveInstance::Or || c.item == ConnectiveInstance::Not)
        {
            return Err(InterpreterError::PatternReceiveError(invalid));
        }
    }
    let remainder = formals
        .remainder
        .map(|r| handle_name_remainder(r, &mut binders_free_map))
        .transpose()?;

    let mut body_par = Par::default();

    let bind_count = bound_map_chain.extend(binders_free_map, |bind_count, new_env| {
        normalize_match_proc(body.proc, &mut body_par, free_map, new_env, env, body.pos)?;
        Ok::<u32, InterpreterError>(bind_count)
    })?;

    union_inplace(
        &mut freevec,
        adjust_bitset(&body_par.locally_free, bind_count as usize),
    );
    let connective_used = name_match_par.connective_used || body_par.connective_used;
    input_par.push_receive(Receive {
        binds: vec![ReceiveBind {
            patterns: binders,
            source: name_match_par,
            remainder,
            free_count: bind_count,
        }],
        body: body_par,
        persistent: true,
        peek: false,
        bind_count,
        locally_free: freevec,
        connective_used,
    });

    Ok(())
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {

    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::test_normalize_match_proc;
    use crate::utils::test_utils::utils::with_bindings;
    use crate::{
        compiler::{
            Context,
            compiler::Compiler,
            exports::SourcePosition,
            free_map::ConnectiveInstance,
            rholang_ast::{AnnName, Id, Names, Proc, SendType},
        },
        errors::InterpreterError,
        normal_forms::{Expr, Par, Receive, ReceiveBind, Send},
    };

    use bitvec::{bitvec, order::Lsb0};
    use pretty_assertions::assert_eq;

    #[test]
    fn p_contr_should_handle_a_basic_contract() {
        /*  new add in {
             contract add(ret, @x, @y) = {
               ret!(x + y)
             }
           }
           // new is simulated by bindings.
        */
        let add = Id {
            name: "add",
            pos: SourcePosition { row: 0, column: 4 },
        };
        let formal_x = Id {
            name: "x",
            pos: SourcePosition { row: 1, column: 20 },
        }
        .as_proc();
        let formal_y = Id {
            name: "y",
            pos: SourcePosition { row: 1, column: 24 },
        }
        .as_proc();
        let x = Proc::new_var("x", SourcePosition { row: 2, column: 8 });
        let y = Proc::new_var("y", SourcePosition { row: 2, column: 12 });
        let x_plus_y = Proc::Add {
            left: x.annotate(SourcePosition { row: 2, column: 8 }),
            right: y.annotate(SourcePosition { row: 2, column: 12 }),
        };
        let p_body = Proc::Send {
            name: Id {
                name: "ret",
                pos: SourcePosition { row: 2, column: 3 },
            }
            .as_name(),
            send_type: SendType::Single,
            inputs: vec![x_plus_y.annotate(SourcePosition { row: 2, column: 10 })],
        };

        let p_contract = Proc::Contract {
            name: AnnName::declare("add", SourcePosition { row: 1, column: 10 }),
            formals: Names {
                names: vec![
                    AnnName::declare("ret", SourcePosition { row: 1, column: 14 }),
                    formal_x
                        .quoted()
                        .annotated(SourcePosition { row: 1, column: 19 }),
                    formal_y
                        .quoted()
                        .annotated(SourcePosition { row: 1, column: 23 }),
                ],
                remainder: None,
            },
            body: p_body.annotate(SourcePosition { row: 1, column: 30 }),
        };

        test_normalize_match_proc(
            &p_contract,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&add);
            }),
            test(
                "expected to handle a basic contract",
                |actual_par, free_map| {
                    let expected_par = Par {
                        receives: vec![Receive {
                            binds: vec![ReceiveBind {
                                patterns: Par::free_vars(3),
                                source: Par::bound_var(0),
                                remainder: None,
                                free_count: 3,
                            }],
                            body: Par {
                                sends: vec![Send {
                                    chan: Par::bound_var(2),
                                    data: vec![Par {
                                        exprs: vec![Expr::EPlus(
                                            Par::bound_var(1),
                                            Par::bound_var(0),
                                        )],
                                        ..Default::default()
                                    }],
                                    persistent: false,
                                    locally_free: bitvec![1, 1, 1],
                                    connective_used: false,
                                }],
                                locally_free: bitvec![1, 1, 1],
                                ..Default::default()
                            },
                            persistent: true,
                            peek: false,
                            bind_count: 3,
                            locally_free: bitvec![1],
                            connective_used: false,
                        }],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert!(free_map.is_empty());
                },
            ),
        );
    }

    #[test]
    fn p_contr_should_not_count_ground_values_in_the_formals_towards_the_bind_count() {
        /*  new ret5 in {
             contract ret5(ret, @5) = {
               ret!(5)
             }
           }
           // new is simulated by bindings.
        */
        let ret5 = Id {
            name: "ret5",
            pos: SourcePosition { row: 0, column: 4 },
        };
        let _5 = Proc::LongLiteral(5);
        let p_body = Proc::Send {
            name: Id {
                name: "ret",
                pos: SourcePosition { row: 2, column: 3 },
            }
            .as_name(),
            send_type: SendType::Single,
            inputs: vec![_5.annotate(SourcePosition { row: 2, column: 8 })],
        };

        let p_contract = Proc::Contract {
            name: AnnName::declare("ret5", SourcePosition { row: 1, column: 10 }),
            formals: Names {
                names: vec![
                    AnnName::declare("ret", SourcePosition { row: 1, column: 14 }),
                    _5.quoted().annotated(SourcePosition { row: 1, column: 20 }),
                ],
                remainder: None,
            },
            body: p_body.annotate(SourcePosition { row: 1, column: 26 }),
        };

        test_normalize_match_proc(
            &p_contract,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&ret5);
            }),
            test(
                "expected not to count ground values in the formals towards the bind count",
                |actual_par, free_map| {
                    let expected_par = Par {
                        receives: vec![Receive {
                            binds: vec![ReceiveBind {
                                patterns: vec![Par::free_var(0), Par::gint(5)],
                                source: Par::bound_var(0),
                                remainder: None,
                                free_count: 1,
                            }],
                            body: Par {
                                sends: vec![Send {
                                    chan: Par::bound_var(0),
                                    data: vec![Par::gint(5)],
                                    persistent: false,
                                    locally_free: bitvec![1],
                                    connective_used: false,
                                }],
                                locally_free: bitvec![1],
                                ..Default::default()
                            },
                            persistent: true,
                            peek: false,
                            bind_count: 1,
                            locally_free: bitvec![1],
                            connective_used: false,
                        }],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert!(free_map.is_empty());
                },
            ),
        );
    }

    #[test]
    fn p_contr_should_not_compile_when_logical_or_or_not_is_used_in_the_pattern_of_the_receive() {
        let mut result =
            Compiler::new(r#"new x in { contract x(@{ y /\ {Nil \/ Nil}}) = { Nil } }"#)
                .compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::PatternReceiveError(Context {
                item: ConnectiveInstance::Or,
                source_position: SourcePosition { row: 0, column: 31 }
            }))
        );

        result =
            Compiler::new(r#"new x in { contract x(@{ y /\ ~Nil}) = { Nil } }"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::PatternReceiveError(Context {
                item: ConnectiveInstance::Not,
                source_position: SourcePosition { row: 0, column: 30 }
            }))
        );
    }

    #[test]
    fn p_contr_should_compile_when_logical_and_is_used_in_the_pattern_of_the_receive() {
        let result = Compiler::new(r#"new x in { contract x(@{ y /\ {Nil /\ Nil}}) = { Nil } }"#)
            .compile_to_adt();
        assert!(result.is_ok());
    }
}
