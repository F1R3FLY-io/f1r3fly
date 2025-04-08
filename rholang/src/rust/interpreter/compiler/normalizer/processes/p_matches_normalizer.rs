use super::exports::{AnnProc, FreeMap, InterpreterError};
use crate::rust::interpreter::{
    compiler::{exports::BoundMapChain, normalize::normalize_match_proc},
    normal_forms::{EMatchesBody, Expr, Par},
};
use std::collections::HashMap;

pub fn normalize_p_matches(
    target: AnnProc,
    pattern: AnnProc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &HashMap<String, Par>,
) -> Result<(), InterpreterError> {
    // In case of 'matches' expression the free variables from the pattern are thrown away and only
    // the ones from the target are used. This is because the "target matches pattern" should have
    // the same semantics as "match target { pattern => true ; _ => false} so free variables from
    // pattern should not be visible at the top level

    let mut target_par = Par::default();
    normalize_match_proc(
        target.proc,
        &mut target_par,
        free_map,
        bound_map_chain,
        env,
        target.pos,
    )?;

    let mut pattern_par = Par::default();

    let depth = bound_map_chain.depth();
    bound_map_chain.descend(|bound_map| {
        let mut temp_free_map = FreeMap::new();

        normalize_match_proc(
            pattern.proc,
            &mut pattern_par,
            &mut temp_free_map,
            bound_map,
            env,
            pattern.pos,
        )
    })?;

    input_par.push_expr(
        Expr::EMatches(EMatchesBody {
            pattern: pattern_par,
            target: target_par,
        }),
        depth,
    );

    Ok(())
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::normalizer::processes::exports::SourcePosition;
    use crate::rust::interpreter::compiler::rholang_ast::{Proc, WILD};
    use crate::rust::interpreter::normal_forms::{Connective, EMatchesBody, Expr, Par};
    use crate::rust::interpreter::test_utils::utils::{defaults, test, test_normalize_match_proc};

    use pretty_assertions::assert_eq;

    // 1 matches _
    #[test]
    fn p_matches_should_normalize_one_matches_wildcard() {
        let _1 = Proc::LongLiteral(1);

        let proc = Proc::Matches {
            target: _1.annotate(SourcePosition { row: 0, column: 0 }),
            pattern: WILD.annotate(SourcePosition { row: 0, column: 10 }),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to normalize '1 matches _'", |actual_result, _| {
                let expected_result = Par {
                    exprs: vec![Expr::EMatches(EMatchesBody {
                        target: Par::gint(1),
                        pattern: Par::wild(),
                    })],
                    ..Default::default()
                };

                assert_eq!(actual_result, &expected_result);
            }),
        );
    }

    // 1 matches 2
    #[test]
    fn p_matches_should_normalize_correctly_one_matches_two() {
        let _1 = Proc::LongLiteral(1);
        let _2 = Proc::LongLiteral(2);

        let proc = Proc::Matches {
            target: _1.annotate(SourcePosition { row: 0, column: 0 }),
            pattern: _2.annotate(SourcePosition { row: 0, column: 10 }),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test(
                "expected to normalize correctly '1 matches 2'",
                |actual_result, _| {
                    let expected_result = Par {
                        exprs: vec![Expr::EMatches(EMatchesBody {
                            target: Par::gint(1),
                            pattern: Par::gint(2),
                        })],
                        ..Default::default()
                    };

                    assert_eq!(actual_result, &expected_result);
                },
            ),
        );
    }

    // 1 matches ~1
    #[test]
    fn p_matches_should_normalize_one_matches_tilda_with_connective_used_false() {
        let _1 = Proc::LongLiteral(1);
        let neg_1 = Proc::Negation(&_1);

        let proc = Proc::Matches {
            target: _1.annotate(SourcePosition { row: 0, column: 0 }),
            pattern: neg_1.annotate(SourcePosition { row: 0, column: 10 }),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test(
                "expected to normalize '1 matches ~1'",
                |actual_result, _| {
                    let expected_result = Par {
                        exprs: vec![Expr::EMatches(EMatchesBody {
                            target: Par::gint(1),
                            pattern: Par {
                                connectives: vec![Connective::ConnNot(Par::gint(1))],
                                connective_used: true,
                                ..Default::default()
                            },
                        })],
                        ..Default::default()
                    };

                    assert_eq!(actual_result, &expected_result);
                },
            ),
        );
    }

    // ~1 matches 1
    #[test]
    fn p_matches_should_normalize_tilda_one_matches_one_with_connective_used_true() {
        let _1 = Proc::LongLiteral(1);
        let neg_1 = Proc::Negation(&_1);

        let proc = Proc::Matches {
            target: neg_1.annotate(SourcePosition { row: 0, column: 0 }),
            pattern: _1.annotate(SourcePosition { row: 0, column: 10 }),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test(
                "expected to normalize '~1 matches 1'",
                |actual_result, _| {
                    let expected_result = Par {
                        exprs: vec![Expr::EMatches(EMatchesBody {
                            pattern: Par::gint(1),
                            target: Par {
                                connectives: vec![Connective::ConnNot(Par::gint(1))],
                                connective_used: true,
                                ..Default::default()
                            },
                        })],
                        connective_used: true,
                        ..Default::default()
                    };

                    assert_eq!(actual_result, &expected_result);
                },
            ),
        );
    }
}
