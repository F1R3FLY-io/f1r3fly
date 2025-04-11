use crate::compiler::exports::{BoundMapChain, FreeMap, SourcePosition};
use crate::compiler::normalizer::normalize_match_proc;
use crate::compiler::rholang_ast::{BundleFlags, BundleType, Proc};
use crate::errors::InterpreterError;
use crate::normal_forms::{Bundle, Par};
use std::collections::BTreeMap;
use std::result::Result;

pub fn normalize_p_bundle<'region>(
    block: &Proc,
    bundle_type: BundleType,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &BTreeMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let mut target_par = Par::default();
    normalize_match_proc(block, &mut target_par, free_map, bound_map_chain, env, pos)?;

    if !target_par.connectives.is_empty() {
        Err(InterpreterError::TopLevelConnectiveInBundle(pos))
    } else if free_map.has_wildcards() || !free_map.is_empty() {
        Err(InterpreterError::UnexpectedBundleContent {
            wildcards: free_map.iter_wildcards().collect(),
            free_vars: free_map.iter_free_vars().collect(),
        })
    } else {
        let this_flags: BundleFlags = bundle_type.into();
        let bundle = if let Some(inner) = target_par.single_bundle_mut() {
            inner.read_flag = inner.read_flag && this_flags.read_flag;
            inner.write_flag = inner.write_flag && this_flags.write_flag;
            target_par.cast_to_bundle()
        } else {
            Bundle {
                body: target_par,
                write_flag: this_flags.write_flag,
                read_flag: this_flags.read_flag,
            }
        };
        input_par.push_bundle(bundle);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::rholang_ast::{
        BundleFlags, BundleType, Id, NIL, Proc, SendType, TYPE_URI, WILD,
    };
    use crate::normal_forms::*;
    use crate::utils::test_utils::utils::defaults;
    use crate::utils::test_utils::utils::test;
    use crate::utils::test_utils::utils::{test_normalize_match_proc, with_bindings};
    use bitvec::{bitvec, order::Lsb0};
    use pretty_assertions::assert_eq;

    use super::SourcePosition;

    /** Example:
     * bundle { x }
     */
    #[test]
    fn p_bundle_should_normalize_terms_inside() {
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 9 },
        };
        let proc_x = x.as_proc();
        let proc = Proc::Bundle {
            bundle_type: BundleType::BundleReadWrite,
            proc: &proc_x,
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bindings| {
                bindings.put_id_as_proc(&x);
            }),
            test(
                "expected to normalize terms inside the bundle",
                |actual_par, free_map| {
                    let expected_par = Par {
                        bundles: vec![Bundle {
                            body: Par::bound_var(0),
                            read_flag: true,
                            write_flag: true,
                        }],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert!(free_map.is_empty());
                    assert_eq!(actual_par, &expected_par);
                },
            ),
        );
    }

    /** Example:
     * bundle { _ | x }
     */
    #[test]
    fn p_bundle_should_throw_an_error_when_wildcard_or_free_variable_is_found_inside_body_of_bundle()
     {
        let x = Proc::new_var("x", SourcePosition { row: 0, column: 13 });
        let par_with_wildcard_and_var = Proc::Par {
            left: WILD.annotate(SourcePosition { row: 0, column: 9 }),
            right: x.annotate(SourcePosition { row: 0, column: 13 }),
        };
        let proc = Proc::Bundle {
            bundle_type: BundleType::BundleReadWrite,
            proc: &par_with_wildcard_and_var,
        };

        test_normalize_match_proc(&proc, defaults(), |result| {
            result.expect_err("expected to throw an error when wildcard or a free variable is found inside the body of the bundle");
        })
    }

    /** Example:
     * bundle { Uri }
     */
    #[test]
    fn p_bundle_should_throw_an_error_when_connective_is_used_at_top_level_of_body_of_bundle() {
        let proc = Proc::Bundle {
            bundle_type: BundleType::BundleReadWrite,
            proc: &TYPE_URI,
        };

        test_normalize_match_proc(&proc, defaults(), |result| {
            result.expect_err("expected to throw an error when a connective is used at the top level of the body of the bundle");
        })
    }

    /** Example:
     * bundle { @Nil!(Uri) }
     */
    #[test]
    fn p_bundle_should_not_throw_an_error_when_connective_is_used_outside_of_top_level_of_body_of_bundle()
     {
        let send = Proc::Send {
            name: NIL.quoted(),
            send_type: SendType::Single,
            inputs: vec![TYPE_URI.annotate(SourcePosition { row: 0, column: 15 })],
        };
        let proc = Proc::Bundle {
            bundle_type: BundleType::BundleReadWrite,
            proc: &send,
        };

        test_normalize_match_proc(&proc, defaults(), |result| {
            result.expect("expected not to throw an error when a connective is used outside of the top level of the body of the bundle");
        })
    }

    #[test]
    fn p_bundle_should_interpret_bundle_polarization() {
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 9 },
        };
        let proc_x = x.as_proc();
        for bundle_type in vec![
            BundleType::BundleReadWrite,
            BundleType::BundleRead,
            BundleType::BundleWrite,
            BundleType::BundleEquiv,
        ] {
            let flags: BundleFlags = bundle_type.into();
            let new_bundle = Proc::Bundle {
                bundle_type,
                proc: &proc_x,
            };

            test_normalize_match_proc(
                &new_bundle,
                with_bindings(|bindings| {
                    bindings.put_id_as_proc(&x);
                }),
                |actual_result| {
                    let (actual_par, _) =
                        actual_result.expect("expected to interpret bundle polarization");

                    let expected_par = Par {
                        bundles: vec![Bundle {
                            body: Par::bound_var(0),
                            read_flag: flags.read_flag,
                            write_flag: flags.write_flag,
                        }],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert_eq!(actual_par, &expected_par);
                },
            );
        }
    }

    #[test]
    fn p_bundle_should_collapse_nested_bundles_merging_their_polarizations() {
        let x = Id {
            name: "x",
            pos: SourcePosition { row: 0, column: 9 },
        };
        let proc_x = x.as_proc();
        let inner_bundle = Proc::Bundle {
            bundle_type: BundleType::BundleReadWrite,
            proc: &proc_x,
        };
        let outer_bundle = Proc::Bundle {
            bundle_type: BundleType::BundleRead,
            proc: &inner_bundle,
        };

        test_normalize_match_proc(
            &outer_bundle,
            with_bindings(|bindings| {
                bindings.put_id_as_proc(&x);
            }),
            test(
                "expected to collapse nested bundles, merging their polarizations",
                |actual_par, free_map| {
                    let expected_par = Par {
                        bundles: vec![Bundle {
                            body: Par::bound_var(0),
                            read_flag: true,
                            write_flag: false,
                        }],
                        locally_free: bitvec![1],
                        ..Default::default()
                    };

                    assert!(free_map.is_empty());
                    assert_eq!(actual_par, &expected_par);
                },
            ),
        );
    }
}
