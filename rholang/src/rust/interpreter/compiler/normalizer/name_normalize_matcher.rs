use super::processes::exports::FreeMap;
use crate::rust::interpreter::compiler::bound_map::BoundContext;
use crate::rust::interpreter::compiler::exports::BoundMapChain;
use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
use crate::rust::interpreter::compiler::rholang_ast::{Name, Var};
use crate::rust::interpreter::compiler::source_position::SourcePosition;
use crate::rust::interpreter::compiler::Context;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::normal_forms::Par;
use std::collections::HashMap;

pub fn normalize_name(
    n: Name,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &HashMap<String, Par>,
    pos: SourcePosition,
) -> Result<Par, InterpreterError> {
    match n {
        Name::ProcVar(Var::Wildcard) => {
            free_map.add_wildcard(pos);
            Ok(Par::wild())
        }
        Name::ProcVar(Var::Id(id)) => match bound_map_chain.get(id.name) {
            Some((
                level,
                Context {
                    item: BoundContext::Name(None),
                    ..
                },
            )) => Ok(Par::bound_var(level)),
            Some((
                level,
                Context {
                    item: BoundContext::Name(Some(urn)),
                    ..
                },
            )) => match env.get(urn) {
                Some(inj) => Ok(inj.clone()),
                None => Ok(Par::bound_var(level)),
            },
            Some((_, proc_ctx)) => Err(InterpreterError::UnexpectedNameContext {
                var_name: id.name.to_string(),
                proc_var_source_position: proc_ctx.source_position,
                name_source_position: id.pos,
            }),
            None => free_map
                .put_id_in_name_context(&id)
                .map(Par::free_var)
                .map_err(
                    |old_context| InterpreterError::UnexpectedReuseOfNameContextFree {
                        var_name: id.name.to_string(),
                        first_use: old_context.source_position,
                        second_use: id.pos,
                    },
                ),
        },
        Name::Quote(quoted) => {
            let mut output = Par::default();
            normalize_match_proc(quoted, &mut output, free_map, bound_map_chain, env, pos)?;
            Ok(output)
        }
    }
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/NameMatcherSpec.scala
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::interpreter::{
        compiler::{
            free_map::VarSort,
            normalizer::processes::exports::Proc,
            rholang_ast::{Id, Name},
            Context,
        },
        normal_forms::Expr,
    };
    use bitvec::{bitvec, order::Lsb0};
    use pretty_assertions::assert_eq;
    use crate::assert_matches;

    #[test]
    fn name_wildcard_should_add_a_wildcard_count_to_known_free() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();
        let result = normalize_name(
            Name::ProcVar(Var::Wildcard),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::wild();
        let actual_result = result.expect("expected to add a wildcard to known free");

        assert_eq!(actual_result, expected_result);
        assert_eq!(free_map.count(), 1);
        assert_eq!(
            free_map.iter_wildcards().next(),
            Some(SourcePosition::default())
        );
    }

    #[test]
    fn name_var_should_compile_as_bound_var_if_its_in_env() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        bound_map_chain.put_id_as_name(&x);
        let result = normalize_name(
            Name::ProcVar(Var::Id(x)),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::bound_var(0);
        let actual_result = result.expect("expected to compile as a bound var");

        assert_eq!(actual_result, expected_result);
        assert_eq!(free_map.count(), 0);
        assert_eq!(
            free_map.iter_wildcards().next(),
            Some(SourcePosition::default())
        );
    }

    #[test]
    fn name_var_should_compile_as_free_var_if_its_not_in_env() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let result = normalize_name(
            Name::ProcVar(Var::Id(x)),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::free_var(0);
        let actual_result = result.expect("expected to compile as a free var");

        assert_eq!(actual_result, expected_result);
        assert_eq!(free_map.count_no_wildcards(), 1);
        assert_eq!(
            free_map.as_free_vec(),
            vec![(
                "x",
                Context {
                    item: VarSort::NameSort,
                    source_position: SourcePosition::default()
                }
            )]
        );
    }

    #[test]
    fn name_var_should_not_compile_if_its_in_env_of_wrong_sort() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition {
                row: 151,
                column: 13,
            },
        };
        bound_map_chain.put_id_as_proc(&x);
        let result = normalize_name(
            Name::ProcVar(Var::new_id(
                "x",
                SourcePosition {
                    row: 157,
                    column: 40,
                },
            )),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let actual_result =
            result.expect_err("expected not to compile if it is of wrong sort in the env");

        assert_matches!(
            actual_result,
            InterpreterError::UnexpectedNameContext {
                var_name,
                proc_var_source_position: SourcePosition {
                    row: 151,
                    column: 13,
                },
                name_source_position: SourcePosition {
                    row: 157,
                    column: 40,
                }
            } => { assert_eq!(var_name, "x") }
        )
    }

    #[test]
    fn name_var_should_not_compile_if_used_free_somewhere_else() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition {
                row: 197,
                column: 13,
            },
        };
        free_map.put_id_in_name_context(&x);
        let result = normalize_name(
            Name::ProcVar(Var::new_id(
                "x",
                SourcePosition {
                    row: 209,
                    column: 40,
                },
            )),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let actual_result =
            result.expect_err("expected not to compile if used free somewhere else");

        assert_matches!(
            actual_result,
            InterpreterError::UnexpectedReuseOfNameContextFree {
                var_name,
                first_use: SourcePosition {
                    row: 197,
                    column: 13,
                },
                second_use: SourcePosition {
                    row: 209,
                    column: 40,
                }
            } => { assert_eq!(var_name, "x") }
        )
    }

    #[test]
    fn name_quote_should_compile_to_bound_var() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        bound_map_chain.put_id_as_proc(&x);
        let px = x.as_proc();
        let result = normalize_name(
            px.quoted(),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::bound_var(0);
        let actual_result = result.expect("expected to compile to bound var");

        assert_eq!(actual_result, expected_result);
        assert_eq!(free_map.count(), 0);
    }

    #[test]
    fn name_quote_should_return_a_free_use_if_the_quoted_proc_has_a_free_var() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        }
        .as_proc();
        let result = normalize_name(
            x.quoted(),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::free_var(0);
        let actual_result = result.expect("expected to compile to a free var");

        assert_eq!(actual_result, expected_result);
        assert_eq!(
            free_map.as_free_vec(),
            vec![(
                "x",
                Context {
                    item: VarSort::NameSort,
                    source_position: SourcePosition::default()
                }
            )]
        );
    }

    #[test]
    fn name_quote_should_compile_to_a_ground() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let _7 = Proc::LongLiteral(7);
        let result = normalize_name(
            _7.quoted(),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::gint(7);
        let actual_result = result.expect("expected to compile to a ground var");

        assert_eq!(actual_result, expected_result);
        assert!(free_map.is_empty());
    }

    #[test]
    fn name_quote_should_collapse_an_eval() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let eval_x = Proc::Eval {
            name: x.as_name().annotated(SourcePosition::default()),
        };
        bound_map_chain.put_id_as_name(&x);
        let result = normalize_name(
            eval_x.quoted(),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par::bound_var(0);
        let actual_result = result.expect("expected to collapse eval");

        assert_eq!(actual_result, expected_result);
        assert!(free_map.is_empty());
    }

    #[test]
    fn name_quote_should_not_collapse_an_eval_eval() {
        let mut free_map = FreeMap::new();
        let mut bound_map_chain = BoundMapChain::new();
        let env = HashMap::new();

        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let eval_x = Proc::Eval {
            name: x.as_name().annotated(SourcePosition::default()),
        };
        let eval_x_par_eval_x = Proc::Par {
            left: eval_x.annotate(SourcePosition { row: 0, column: 0 }),
            right: eval_x.annotate(SourcePosition { row: 0, column: 4 }),
        };
        bound_map_chain.put_id_as_name(&x);
        let result = normalize_name(
            eval_x_par_eval_x.quoted(),
            &mut free_map,
            &mut bound_map_chain,
            &env,
            SourcePosition::default(),
        );
        let expected_result = Par {
            exprs: vec![Expr::new_bound_var(0), Expr::new_bound_var(0)],
            locally_free: bitvec![1],
            ..Default::default()
        };
        let actual_result = result.expect("expected not to collapse par of evals");

        assert_eq!(actual_result, expected_result);
        assert!(free_map.is_empty());
    }
}
