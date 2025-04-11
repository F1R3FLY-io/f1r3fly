use crate::compiler::exports::{BoundMapChain, FreeMap, SourcePosition};
use crate::compiler::free_map::ConnectiveInstance;
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::AnnProc;
use crate::errors::InterpreterError;
use crate::normal_forms::{Connective, Par};
use std::collections::BTreeMap;

pub fn normalize_p_conjunction(
    left: AnnProc,
    right: AnnProc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let depth = bound_map_chain.depth();

    let mut left_par = Par::default();
    normalize_match_proc(
        left.proc,
        &mut left_par,
        free_map,
        bound_map_chain,
        env,
        left.pos,
    )?;

    let mut right_par = Par::default();
    normalize_match_proc(
        right.proc,
        &mut right_par,
        free_map,
        bound_map_chain,
        env,
        right.pos,
    )?;

    let connective = if let Some(Connective::ConnAnd(conn_body)) = left_par.single_connective_mut()
    {
        conn_body.push(right_par);
        left_par.cast_to_connective()
    } else {
        Connective::ConnAnd(vec![left_par, right_par])
    };
    input_par.push_connective(connective, depth);
    free_map.remember_connective(ConnectiveInstance::And, pos);

    Ok(())
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::compiler::Context;
    use crate::compiler::free_map::VarSort;
    use crate::compiler::rholang_ast::Proc;
    use crate::compiler::source_position::SourcePosition;
    use crate::normal_forms::{Connective, Par};
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::test_normalize_match_proc;
    use pretty_assertions::assert_eq;

    #[test]
    /** Example:
     * x /\ y
     */
    fn p_conjunction_should_delegate_and_count_any_free_variables_inside() {
        let x = Proc::new_var("x", SourcePosition { row: 0, column: 0 });
        let y = Proc::new_var("y", SourcePosition { row: 0, column: 5 });
        let proc = Proc::Conjunction {
            left: x.annotate(SourcePosition { row: 0, column: 0 }),
            right: y.annotate(SourcePosition { row: 0, column: 5 }),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test(
                "expected to delegate and count any free variables inside",
                |actual_par, free_map| {
                    let expected_par = Par {
                        connective_used: true,
                        connectives: vec![Connective::ConnAnd(Par::free_vars(2))],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert_eq!(
                        free_map.as_free_vec(),
                        vec![
                            (
                                "x",
                                Context {
                                    item: VarSort::ProcSort,
                                    source_position: SourcePosition { row: 0, column: 0 }
                                }
                            ),
                            (
                                "y",
                                Context {
                                    item: VarSort::ProcSort,
                                    source_position: SourcePosition { row: 0, column: 5 }
                                }
                            )
                        ]
                    )
                },
            ),
        );
    }
}
