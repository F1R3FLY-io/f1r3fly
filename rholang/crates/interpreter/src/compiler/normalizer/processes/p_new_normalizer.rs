use bitvec::vec::BitVec;

use crate::aliases::EnvHashMap;
use crate::compiler::exports::{BoundMapChain, FreeMap, SourcePosition};
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::{NameDecl, Proc};
use crate::errors::InterpreterError;
use crate::normal_forms::{New, Par, adjust_bitset};

pub fn normalize_p_new(
    decls: &[NameDecl],
    body: &Proc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let mut uris = Vec::new(); //no point to overallocate
    for decl in decls {
        if let Some(old_binding) = bound_map_chain.put_name(&decl.id, &decl.uri) {
            return Err(InterpreterError::UnexpectedReuseOfNameContextFree {
                var_name: decl.id.name.to_string(),
                first_use: old_binding.source_position,
                second_use: decl.id.pos,
            });
        }

        if let Some(ref uri) = decl.uri {
            uris.push(uri.to_string());
        }
    }
    let mut body_par = Par::default();
    normalize_match_proc(body, &mut body_par, free_map, bound_map_chain, env, pos)?;

    let new_count = decls.len();
    let locally_free = BitVec::from_bitslice(adjust_bitset(&body_par.locally_free, new_count));
    let result_new = New {
        bind_count: new_count as u32,
        p: body_par,
        uris,
        locally_free,
    };

    input_par.push_new(result_new);
    bound_map_chain.shift(new_count);

    Ok(())
}

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::test_normalize_match_proc;
    use bitvec::{bitvec, order::Lsb0, vec::BitVec};
    use pretty_assertions::assert_eq;

    use crate::{
        compiler::{
            exports::SourcePosition,
            rholang_ast::{ASTBuilder, Id, Name, NameDecl, Proc, SendType},
        },
        normal_forms::{New, Par, Send},
    };

    #[test]
    fn p_new_should_bind_new_variables() {
        /* Example:
        new x, y, z in {
            z ! 9 | y ! 8 | x ! 7
        }
        */
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 4 },
        };
        let y = Id {
            name: "y",
            pos: SourcePosition { row: 0, column: 7 },
        };
        let z = Id {
            name: "z",
            pos: SourcePosition { row: 0, column: 10 },
        };
        let _9 = Proc::LongLiteral(9);
        let z_send_9 = Proc::Send {
            name: Name::declare("z"),
            send_type: SendType::Single,
            inputs: vec![_9.annotate(SourcePosition { row: 1, column: 8 })],
        };
        let _8 = Proc::LongLiteral(8);
        let y_send_8 = Proc::Send {
            name: Name::declare("y"),
            send_type: SendType::Single,
            inputs: vec![_8.annotate(SourcePosition { row: 1, column: 16 })],
        };
        let _7 = Proc::LongLiteral(7);
        let x_send_7 = Proc::Send {
            name: Name::declare("x"),
            send_type: SendType::Single,
            inputs: vec![_7.annotate(SourcePosition { row: 1, column: 24 })],
        };
        let ppar_1 = Proc::Par {
            left: z_send_9.annotate(SourcePosition { row: 1, column: 4 }),
            right: y_send_8.annotate(SourcePosition { row: 1, column: 12 }),
        };
        let ppar = Proc::Par {
            left: ppar_1.annotate(SourcePosition { row: 1, column: 10 }),
            right: x_send_7.annotate(SourcePosition { row: 1, column: 20 }),
        };
        let p_new = Proc::New {
            decls: vec![
                NameDecl { id: x, uri: None },
                NameDecl { id: y, uri: None },
                NameDecl { id: z, uri: None },
            ],
            proc: &ppar,
        };

        test_normalize_match_proc(
            &p_new,
            defaults(),
            test("expected to bind new variables", |actual_par, free_map| {
                let expected_par = Par {
                    news: vec![New {
                        bind_count: 3,
                        p: Par {
                            sends: vec![
                                Send {
                                    chan: Par::bound_var(2),
                                    data: vec![Par::gint(9)],
                                    persistent: false,
                                    locally_free: bitvec![0, 0, 1].into(),
                                    connective_used: false,
                                },
                                Send {
                                    chan: Par::bound_var(1),
                                    data: vec![Par::gint(8)],
                                    persistent: false,
                                    locally_free: bitvec![0, 1].into(),
                                    connective_used: false,
                                },
                                Send {
                                    chan: Par::bound_var(0),
                                    data: vec![Par::gint(7)],
                                    persistent: false,
                                    locally_free: bitvec![1].into(),
                                    connective_used: false,
                                },
                            ],
                            locally_free: bitvec![1, 1, 1].into(),
                            ..Default::default()
                        },
                        uris: Vec::new(),
                        locally_free: BitVec::EMPTY.into(),
                    }],
                    locally_free: bitvec![1, 1, 1].into(),
                    ..Default::default()
                };

                assert_eq!(actual_par, &expected_par);
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_new_should_sort_uris_and_place_them_at_the_end() {
        /* Example:
        new x, y, r(`rho:registry`), out(`rho:stdout`), z in {
            x ! 7 | y ! 8 | r ! 9 | out ! 10 | z ! 11
        }
        */
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 4 },
        };
        let y = Id {
            name: "y",
            pos: SourcePosition { row: 0, column: 7 },
        };
        let r = Id {
            name: "r",
            pos: SourcePosition { row: 0, column: 10 },
        };
        let out = Id {
            name: "out",
            pos: SourcePosition { row: 0, column: 29 },
        };
        let z = Id {
            name: "z",
            pos: SourcePosition { row: 0, column: 48 },
        };
        let _7 = Proc::LongLiteral(7);
        let x_send_7 = Proc::Send {
            name: Name::declare("x"),
            send_type: SendType::Single,
            inputs: vec![_7.annotate(SourcePosition { row: 1, column: 8 })],
        };
        let _8 = Proc::LongLiteral(8);
        let y_send_8 = Proc::Send {
            name: Name::declare("y"),
            send_type: SendType::Single,
            inputs: vec![_8.annotate(SourcePosition { row: 1, column: 16 })],
        };
        let _9 = Proc::LongLiteral(9);
        let r_send_9 = Proc::Send {
            name: Name::declare("r"),
            send_type: SendType::Single,
            inputs: vec![_9.annotate(SourcePosition { row: 1, column: 24 })],
        };
        let _10 = Proc::LongLiteral(10);
        let out_send_10 = Proc::Send {
            name: Name::declare("out"),
            send_type: SendType::Single,
            inputs: vec![_10.annotate(SourcePosition { row: 1, column: 34 })],
        };
        let _11 = Proc::LongLiteral(11);
        let z_send_11 = Proc::Send {
            name: Name::declare("z"),
            send_type: SendType::Single,
            inputs: vec![_11.annotate(SourcePosition { row: 1, column: 43 })],
        };
        let procs = vec![x_send_7, y_send_8, r_send_9, out_send_10, z_send_11];
        let builder = ASTBuilder::with_capacity(4);
        let ppar = builder.fold_procs_into_par(
            &procs,
            vec![
                SourcePosition { row: 1, column: 4 },
                SourcePosition { row: 1, column: 12 },
                SourcePosition { row: 1, column: 10 },
                SourcePosition { row: 1, column: 20 },
                SourcePosition { row: 1, column: 18 },
                SourcePosition { row: 1, column: 28 },
                SourcePosition { row: 1, column: 26 },
                SourcePosition { row: 1, column: 39 },
                SourcePosition { row: 1, column: 37 },
            ],
        );
        let p_new = Proc::New {
            decls: vec![
                NameDecl { id: x, uri: None },
                NameDecl { id: y, uri: None },
                NameDecl {
                    id: r,
                    uri: Some("rho:registry".into()),
                },
                NameDecl {
                    id: out,
                    uri: Some("rho:stdout".into()),
                },
                NameDecl { id: z, uri: None },
            ],
            proc: &ppar,
        };

        test_normalize_match_proc(
            &p_new,
            defaults(),
            test("expected to bind new variables", |actual_par, free_map| {
                let expected_par = Par {
                    news: vec![New {
                        bind_count: 5,
                        p: Par {
                            sends: vec![
                                Send {
                                    chan: Par::bound_var(0),
                                    data: vec![Par::gint(7)],
                                    persistent: false,
                                    locally_free: bitvec![1].into(),
                                    connective_used: false,
                                },
                                Send {
                                    chan: Par::bound_var(1),
                                    data: vec![Par::gint(8)],
                                    persistent: false,
                                    locally_free: bitvec![0, 1].into(),
                                    connective_used: false,
                                },
                                Send {
                                    chan: Par::bound_var(3),
                                    data: vec![Par::gint(9)],
                                    persistent: false,
                                    locally_free: bitvec![0, 0, 0, 1].into(),
                                    connective_used: false,
                                },
                                Send {
                                    chan: Par::bound_var(4),
                                    data: vec![Par::gint(10)],
                                    persistent: false,
                                    locally_free: bitvec![0, 0, 0, 0, 1].into(),
                                    connective_used: false,
                                },
                                Send {
                                    chan: Par::bound_var(2),
                                    data: vec![Par::gint(11)],
                                    persistent: false,
                                    locally_free: bitvec![0, 0, 1].into(),
                                    connective_used: false,
                                },
                            ],
                            locally_free: bitvec![1, 1, 1, 1, 1].into(),
                            ..Default::default()
                        },
                        uris: Vec::new(),
                        locally_free: BitVec::EMPTY.into(),
                    }],
                    locally_free: bitvec![1, 1, 1, 1, 1].into(),
                    ..Default::default()
                };

                assert_eq!(actual_par, &expected_par);
                assert!(free_map.is_empty());
            }),
        );
    }
}
