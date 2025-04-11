use bitvec::vec::BitVec;

use crate::compiler::exports::{BoundMapChain, FreeMap, SourcePosition};
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::{AnnProc, Id, Proc};
use crate::errors::InterpreterError;
use crate::normal_forms::{EMethodBody, Expr, Par, union, union_inplace};
use std::collections::BTreeMap;

pub fn normalize_p_method(
    receiver: &Proc,
    name: Id,
    args: &[AnnProc],
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let depth = bound_map_chain.depth();
    let mut receiver_par = Par::default();
    normalize_match_proc(
        receiver,
        &mut receiver_par,
        free_map,
        bound_map_chain,
        env,
        pos,
    )?;

    let arg_result: Result<Vec<_>, _> = args
        .iter()
        .map(|arg| {
            let mut arg_par = Par::default();
            normalize_match_proc(
                arg.proc,
                &mut arg_par,
                free_map,
                bound_map_chain,
                env,
                arg.pos,
            )
            .map(|_| arg_par)
        })
        .collect();
    let args = arg_result?;
    let connective_used =
        receiver_par.connective_used || args.iter().any(|arg| arg.connective_used);
    let locally_free = union(
        &receiver_par.locally_free,
        args.iter().fold(BitVec::default(), |mut bv, arg| {
            union_inplace(&mut bv, &arg.locally_free);
            bv
        }),
    );

    let method = Expr::EMethod(EMethodBody {
        method_name: name.name.to_owned(),
        target: receiver_par,
        arguments: args,
        locally_free,
        connective_used,
    });

    input_par.push_expr(method, depth);
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
            exports::SourcePosition,
            rholang_ast::{Id, Proc},
        },
        normal_forms::{EMethodBody, Expr, Par},
    };
    use bitvec::{bitvec, order::Lsb0};

    #[test]
    fn p_method_should_produce_proper_method_call() {
        // x.nth(0) |
        // x.toByteArray()
        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let _0 = Proc::LongLiteral(0);
        let x_1 = Proc::new_var("x", SourcePosition { row: 0, column: 0 });
        let p_nth = Proc::Method {
            receiver: &x_1,
            name: Id {
                name: "nth",
                pos: SourcePosition { row: 0, column: 2 },
            },
            args: vec![_0.annotate(SourcePosition { row: 0, column: 6 })],
        };
        let x_2 = Proc::new_var("x", SourcePosition { row: 1, column: 0 });
        let p_tba = Proc::Method {
            receiver: &x_2,
            name: Id {
                name: "toByteArray",
                pos: SourcePosition { row: 1, column: 2 },
            },
            args: Vec::new(),
        };

        let p_par = Proc::Par {
            left: p_nth.annotate(SourcePosition { row: 0, column: 0 }),
            right: p_tba.annotate(SourcePosition { row: 1, column: 0 }),
        };

        test_normalize_match_proc(
            &p_par,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&x);
            }),
            test(
                "expected to produce a proper method call",
                |actual_par, free_map| {
                    let expected_par = Par {
                        exprs: vec![
                            Expr::EMethod(EMethodBody {
                                method_name: "nth".to_string(),
                                target: Par::bound_var(0),
                                arguments: vec![Par::gint(0)],
                                locally_free: bitvec![1],
                                connective_used: false,
                            }),
                            Expr::EMethod(EMethodBody {
                                method_name: "toByteArray".to_string(),
                                target: Par::bound_var(0),
                                arguments: Vec::new(),
                                locally_free: bitvec![1],
                                connective_used: false,
                            }),
                        ],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert!(free_map.is_empty());
                },
            ),
        )
    }
}
