use bitvec::vec::BitVec;

use super::exports::*;
use crate::rust::interpreter::compiler::exports::BoundMapChain;
use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
use crate::rust::interpreter::compiler::rholang_ast::{Name, SendType};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::normal_forms::{union, union_inplace, Par, Send};
use std::collections::HashMap;

pub fn normalize_p_send(
    name: Name,
    send_type: SendType,
    inputs: &[AnnProc],
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &HashMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let name_match_result = normalize_name(name, free_map, bound_map_chain, env, pos)?;

    let data_result: Result<Vec<_>, _> = inputs
        .iter()
        .map(|p| {
            let mut proc_match_result = Par::default();
            normalize_match_proc(
                p.proc,
                &mut proc_match_result,
                free_map,
                bound_map_chain,
                env,
                p.pos,
            )
            .map(|_| proc_match_result)
        })
        .collect();
    let data = data_result?;

    let persistent = match send_type {
        SendType::Single => false,
        SendType::Multiple => true,
    };
    let connective_used =
        name_match_result.connective_used || data.iter().any(|p| p.connective_used);
    let locally_free = union(
        &name_match_result.locally_free,
        data.iter().fold(BitVec::default(), |mut bv, p| {
            union_inplace(&mut bv, &p.locally_free);
            bv
        }),
    );

    let send = Send {
        chan: name_match_result,
        data,
        persistent,
        locally_free,
        connective_used,
    };
    input_par.push_send(send);
    Ok(())
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::{
        compiler::{
            compiler::Compiler,
            free_map::ConnectiveInstance,
            rholang_ast::{Id, SendType, NIL},
            Context,
        },
        errors::InterpreterError,
        normal_forms::{Par, Send},
        test_utils::utils::*,
    };

    use super::{Proc, SourcePosition};
    use bitvec::{bitvec, order::Lsb0};
    use pretty_assertions::assert_eq;
    use crate::assert_matches;

    #[test]
    fn p_send_should_handle_a_basic_send() {
        // @Nil!(7, 8)
        let _7 = Proc::LongLiteral(7);
        let _8 = Proc::LongLiteral(8);
        let p_send = Proc::Send {
            name: NIL.quoted(),
            send_type: SendType::Single,
            inputs: vec![
                _7.annotate(SourcePosition { row: 0, column: 7 }),
                _8.annotate(SourcePosition { row: 0, column: 9 }),
            ],
        };

        test_normalize_match_proc(
            &p_send,
            defaults(),
            test("expected to handle a basic send", |actual_par, free_map| {
                let expected_par = Par {
                    sends: vec![Send {
                        chan: Par::NIL,
                        data: vec![Par::gint(7), Par::gint(8)],
                        ..Default::default()
                    }],
                    ..Default::default()
                };

                assert_eq!(actual_par, &expected_par);
                assert!(free_map.is_empty());
            }),
        )
    }

    #[test]
    fn p_send_should_handle_a_name_var() {
        // x!(7,8)
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 0 },
        };
        let _7 = Proc::LongLiteral(7);
        let _8 = Proc::LongLiteral(8);
        let p_send = Proc::Send {
            name: x.as_name(),
            send_type: SendType::Single,
            inputs: vec![
                _7.annotate(SourcePosition { row: 0, column: 7 }),
                _8.annotate(SourcePosition { row: 0, column: 9 }),
            ],
        };

        test_normalize_match_proc(
            &p_send,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&x);
            }),
            test("expected to handle a name", |actual_par, free_map| {
                let expected_par = Par {
                    sends: vec![Send {
                        chan: Par::bound_var(0),
                        data: vec![Par::gint(7), Par::gint(8)],
                        ..Default::default()
                    }],
                    locally_free: bitvec![1],
                    ..Default::default()
                };

                assert_eq!(actual_par, &expected_par);
                assert!(free_map.is_empty());
            }),
        )
    }

    #[test]
    fn p_send_should_propagate_known_free() {
        // x!(7, x)
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 0 },
        };
        let _7 = Proc::LongLiteral(7);
        let proc_x = Proc::new_var("x", SourcePosition { row: 0, column: 6 });
        let p_send = Proc::Send {
            name: x.as_name(),
            send_type: SendType::Single,
            inputs: vec![
                _7.annotate(SourcePosition { row: 0, column: 3 }),
                proc_x.annotate(SourcePosition { row: 0, column: 6 }),
            ],
        };

        test_normalize_match_proc(&p_send, defaults(), |result| {
            let actual_result = result.expect_err("expected to propagate known free");

            assert_matches!(
                actual_result,
                InterpreterError::UnexpectedReuseOfProcContextFree {
                    var_name,
                    first_use: SourcePosition { row: 0, column: 0 },
                    second_use: SourcePosition { row: 0, column: 6 }
                } =>  { assert_eq!(var_name, "x"); }
            );
        })
    }

    #[test]
    fn p_send_should_not_compile_if_data_contains_negation() {
        let result = Compiler::new(r#"new x in { x!(~1) }"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                vec![Context {
                    item: ConnectiveInstance::Not,
                    source_position: SourcePosition::new(0, 14)
                }]
            ))
        )
    }

    #[test]
    fn p_send_should_not_compile_if_data_contains_conjuction() {
        let result = Compiler::new(r#"new x in { x!(1 /\ 2) }"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                vec![Context {
                    item: ConnectiveInstance::And,
                    source_position: SourcePosition::new(0, 16)
                }]
            ))
        )
    }

    #[test]
    fn p_send_should_not_compile_if_data_contains_disjunction() {
        let result = Compiler::new(r#"new x in { x!(1 \/ 2) }"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                vec![Context {
                    item: ConnectiveInstance::Or,
                    source_position: SourcePosition::new(0, 16)
                }]
            ))
        )
    }

    #[test]
    fn p_send_should_not_compile_if_data_contains_wildcard() {
        let result = Compiler::new(r#"@"x"!(_)"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelWildcardsNotAllowedError(vec![
                SourcePosition { row: 0, column: 6 }
            ]))
        )
    }

    #[test]
    fn p_send_should_not_compile_if_data_contains_free_variable() {
        let result = Compiler::new(r#"@"x"!(y)"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelFreeVariablesNotAllowedError(
                vec![Context {
                    item: "y".to_string(),
                    source_position: SourcePosition::new(0, 6)
                }]
            ))
        )
    }

    #[test]
    fn p_send_should_not_compile_if_name_contains_connectives() {
        let mut result = Compiler::new(r#"@{Nil /\ Nil}!(1)"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                vec![Context {
                    item: ConnectiveInstance::And,
                    source_position: SourcePosition::new(0, 6)
                }]
            ))
        );

        result = Compiler::new(r#"@{Nil \/ Nil}!(1)"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                vec![Context {
                    item: ConnectiveInstance::Or,
                    source_position: SourcePosition::new(0, 6)
                }]
            ))
        );

        result = Compiler::new(r#"@{~Nil}!(1)"#).compile_to_adt();
        assert_eq!(
            result,
            Err(InterpreterError::TopLevelLogicalConnectivesNotAllowedError(
                vec![Context {
                    item: ConnectiveInstance::Not,
                    source_position: SourcePosition::new(0, 2)
                }]
            ))
        );
    }
}
