use crate::aliases::EnvHashMap;
use crate::compiler::exports::{BoundMapChain, FreeMap, SourcePosition};
use crate::compiler::free_map::ConnectiveInstance;
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::Proc;
use crate::errors::InterpreterError;
use crate::normal_forms::{Connective, Par};

pub fn normalize_p_negation(
    body: &Proc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let depth = bound_map_chain.depth();

    let mut conn_body = Par::default();
    let mut temp_free_map = FreeMap::new();
    normalize_match_proc(
        body,
        &mut conn_body,
        &mut temp_free_map,
        bound_map_chain,
        env,
        pos,
    )?;

    free_map.remember_connective(ConnectiveInstance::Not, pos);
    input_par.push_connective(Connective::ConnNot(conn_body), depth);

    Ok(())
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::normal_forms::{Connective, Par};
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::test_normalize_match_proc;
    use pretty_assertions::assert_eq;

    use super::{Proc, SourcePosition};

    #[test]
    /** Example:
     * ~x
     */
    fn p_negation_should_delegate_but_not_count_any_free_variables_inside() {
        let x = Proc::new_var("x", SourcePosition { row: 0, column: 1 });
        let proc = Proc::Negation(&x);

        test_normalize_match_proc(
            &proc,
            defaults(),
            test(
                "expected to delegate but not count any free variables inside",
                |actual_par, free_map| {
                    let expected_par = Par {
                        connective_used: true,
                        connectives: vec![Connective::ConnNot(Par::free_var(0))],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                    assert!(free_map.is_empty())
                },
            ),
        );
    }
}
