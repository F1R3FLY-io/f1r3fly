use crate::compiler::Context;
use crate::compiler::bound_map::BoundContext;
use crate::compiler::exports::{BoundMapChain, SourcePosition};
use crate::compiler::rholang_ast::{VarRef, VarRefKind};
use crate::errors::InterpreterError;
use crate::normal_forms::{Connective, Par, VarRef as PVarRef};

use std::collections::BTreeMap;
use std::result::Result;

pub fn normalize_p_var_ref(
    p: VarRef,
    input_par: &mut Par,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let var = p.var.name;
    match bound_map_chain.find(var) {
        None => Err(InterpreterError::UnboundVariableRef {
            var_name: var.to_string(),
            pos,
        }),
        Some((
            (
                idx,
                Context {
                    item: BoundContext::Proc,
                    source_position,
                },
            ),
            depth,
        )) => match p.kind {
            VarRefKind::Proc => {
                input_par.push_connective(
                    Connective::VarRef(PVarRef { index: idx, depth }),
                    bound_map_chain.depth(),
                );
                Ok(())
            }
            _ => Err(InterpreterError::UnexpectedProcContext {
                var_name: var.to_string(),
                name_var_source_position: pos,
                process_source_position: *source_position,
            }),
        },
        Some((
            (
                idx,
                Context {
                    item: BoundContext::Name(maybe_urn),
                    source_position,
                },
            ),
            depth,
        )) => match p.kind {
            VarRefKind::Name => {
                if let Some(urn) = maybe_urn {
                    if let Some(par) = env.get(urn) {
                        input_par.concat_with(par.clone());
                        return Ok(());
                    }
                }
                input_par.push_connective(
                    Connective::VarRef(PVarRef { index: idx, depth }),
                    bound_map_chain.depth(),
                );
                Ok(())
            }
            _ => Err(InterpreterError::UnexpectedNameContext {
                var_name: var.to_string(),
                proc_var_source_position: pos,
                name_source_position: *source_position,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::test_utils::utils::test;
    use pretty_assertions::assert_eq;

    use crate::{
        compiler::{
            exports::SourcePosition,
            rholang_ast::{
                Case, Id, LinearBind, NIL, Names, Proc, Receipt, Source, VarRef, VarRefKind,
            },
        },
        normal_forms::{
            Connective, Match, MatchCase, Par, Receive, ReceiveBind, VarRef as NVarRef,
        },
        utils::test_utils::utils::{test_normalize_match_proc, with_bindings},
    };
    use bitvec::{bitvec, order::Lsb0};

    #[test]
    fn p_var_ref_should_do_deep_lookup_in_match_case() {
        // assuming `x` is bound
        // example: @7!(10) | for (@x <- @7) { … }
        // match 7 { =x => Nil }

        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 25 },
        };
        let _7 = Proc::LongLiteral(7);
        let x_ref = Proc::VarRef(VarRef {
            kind: VarRefKind::Proc,
            var: Id {
                name: "x",
                pos: SourcePosition { row: 1, column: 11 },
            },
        });
        let proc = Proc::Match {
            expression: _7.annotate(SourcePosition { row: 1, column: 6 }),
            cases: vec![Case {
                pattern: x_ref.annotate(SourcePosition { row: 1, column: 10 }),
                proc: NIL.annotate(SourcePosition { row: 1, column: 16 }),
            }],
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test("expected to do deep lookup", |actual_par, _| {
                let expected_par = Par {
                    matches: vec![Match {
                        target: Par::gint(7),
                        cases: vec![MatchCase {
                            pattern: Par {
                                connectives: vec![Connective::VarRef(NVarRef {
                                    index: 0,
                                    depth: 1,
                                })],
                                locally_free: bitvec![1],
                                ..Default::default()
                            },
                            source: Par::NIL,
                            free_count: 0,
                        }],
                        locally_free: bitvec![1],
                        connective_used: false,
                    }],
                    locally_free: bitvec![1],
                    ..Default::default()
                };

                assert_eq!(actual_par, &expected_par);
            }),
        );
    }

    #[test]
    fn p_var_ref_should_do_deep_lookup_in_receive_case() {
        // assuming `x` is bound:
        // example : new x in { … }
        // for(@{=*x} <- @Nil) { Nil }

        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 4 },
        };
        let x_ref = Proc::VarRef(VarRef {
            kind: VarRefKind::Name,
            var: Id {
                name: "x",
                pos: SourcePosition { row: 1, column: 8 },
            },
        });
        let proc = Proc::ForComprehension {
            receipts: vec![Receipt::Linear(vec![LinearBind {
                lhs: Names::single(
                    x_ref
                        .quoted()
                        .annotated(SourcePosition { row: 1, column: 4 }),
                ),
                rhs: Source::Simple {
                    name: NIL
                        .quoted()
                        .annotated(SourcePosition { row: 1, column: 14 }),
                },
            }])],
            proc: NIL.annotate(SourcePosition { row: 1, column: 22 }),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&x);
            }),
            test("expected to do deep lookup", |actual_par, _| {
                let expected_par = Par {
                    receives: vec![Receive {
                        binds: vec![ReceiveBind {
                            patterns: vec![Par {
                                connectives: vec![Connective::VarRef(NVarRef {
                                    index: 0,
                                    depth: 1,
                                })],
                                locally_free: bitvec![1],
                                ..Default::default()
                            }],
                            source: Par::NIL,
                            remainder: None,
                            free_count: 0,
                        }],
                        body: Par::NIL,
                        persistent: false,
                        peek: false,
                        bind_count: 0,
                        locally_free: bitvec![1],
                        connective_used: false,
                    }],
                    locally_free: bitvec![1],
                    ..Default::default()
                };

                assert_eq!(actual_par, &expected_par);
            }),
        );
    }
}
