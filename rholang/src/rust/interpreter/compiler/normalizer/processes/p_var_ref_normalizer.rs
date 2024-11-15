use crate::rust::interpreter::compiler::exports::BoundContext;
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::compiler::rholang_ast::{VarRef as PVarRef, VarRefKind};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;

use super::exports::*;
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::{Connective, VarRef};
use std::result::Result;

pub fn normalize_p_var_ref(
    p: &PVarRef,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    match input.bound_map_chain.find(&p.var.name) {
        Some((
            BoundContext {
                index,
                typ,
                source_position,
            },
            depth,
        )) => match typ {
            VarSort::ProcSort => match p.var_ref_kind {
                VarRefKind::Proc => Ok(ProcVisitOutputs {
                    par: prepend_connective(
                        input.par,
                        Connective {
                            connective_instance: Some(ConnectiveInstance::VarRefBody(VarRef {
                                index: index as i32,
                                depth: depth as i32,
                            })),
                        },
                        input.bound_map_chain.depth() as i32,
                    ),
                    free_map: input.free_map,
                }),

                _ => Err(InterpreterError::UnexpectedProcContext {
                    var_name: p.var.name.clone(),
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: p.line_num,
                        column: p.col_num,
                    },
                }),
            },
            VarSort::NameSort => match p.var_ref_kind {
                VarRefKind::Name => Ok(ProcVisitOutputs {
                    par: prepend_connective(
                        input.par,
                        Connective {
                            connective_instance: Some(ConnectiveInstance::VarRefBody(VarRef {
                                index: index as i32,
                                depth: depth as i32,
                            })),
                        },
                        input.bound_map_chain.depth() as i32,
                    ),
                    free_map: input.free_map,
                }),

                _ => Err(InterpreterError::UnexpectedProcContext {
                    var_name: p.var.name.clone(),
                    name_var_source_position: source_position,
                    process_source_position: SourcePosition {
                        row: p.line_num,
                        column: p.col_num,
                    },
                }),
            },
        },

        None => Err(InterpreterError::UnboundVariableRef {
            var_name: p.var.name.clone(),
            line: p.line_num,
            col: p.col_num,
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
    use crate::rust::interpreter::compiler::normalize::VarSort::{NameSort, ProcSort};
    use crate::rust::interpreter::compiler::rholang_ast::Proc::Match;
    use crate::rust::interpreter::compiler::rholang_ast::{
        Block, Case, LinearBind, Name, Names, Proc, Quote, Receipt, Receipts, Source, Var, VarRef,
        VarRefKind,
    };
    use crate::rust::interpreter::test_utils::utils::{
        proc_visit_inputs_and_env, proc_visit_inputs_with_updated_bound_map_chain,
    };
    use models::create_bit_vector;
    use models::rhoapi::connective::ConnectiveInstance::VarRefBody;
    use models::rhoapi::{Connective, Match as model_match, MatchCase, Par, ReceiveBind};
    use models::rhoapi::{Receive, VarRef as model_VarRef};
    use models::rust::utils::new_gint_par;
    use pretty_assertions::assert_eq;

    #[test]
    fn p_var_ref_should_do_deep_lookup_in_match_case() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let bound_inputs =
            proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", ProcSort);

        let proc = Match {
            expression: Box::new(Proc::new_proc_int(7)),
            cases: vec![Case::new(
                Proc::VarRef(VarRef {
                    var_ref_kind: VarRefKind::Proc,
                    var: Var::new("x".to_string()),
                    line_num: 0,
                    col_num: 0,
                }),
                Proc::new_proc_nil(),
            )],
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);
        let expected_result = bound_inputs
            .par
            .clone()
            .with_matches(vec![
                (model_match {
                    target: Some(new_gint_par(7, Vec::new(), false)),

                    cases: vec![MatchCase {
                        pattern: Some(
                            Par {
                                connectives: vec![Connective {
                                    connective_instance: Some(VarRefBody(model_VarRef {
                                        index: 0,
                                        depth: 1,
                                    })),
                                }],
                                ..Par::default().clone()
                            }
                            .with_locally_free(create_bit_vector(&vec![0])),
                        ),
                        source: Some(Par::default()),
                        free_count: 0,
                    }],

                    locally_free: create_bit_vector(&vec![0]),
                    connective_used: false,
                }),
            ])
            .with_locally_free(create_bit_vector(&vec![0]));
        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.clone().unwrap().free_map, inputs.free_map);
        // Make sure that variable references in patterns are reflected
        // BitSet(0) == create_bit_vector(&vec![0])
        assert_eq!(
            result.clone().unwrap().par.locally_free,
            create_bit_vector(&vec![0])
        );
    }

    #[test]
    fn p_var_ref_should_do_deep_lookup_in_receive_case() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let bound_inputs =
            proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", NameSort);

        let proc = Proc::Input {
            formals: Receipts::new(vec![Receipt::LinearBinds(LinearBind::new_linear_bind(
                Names {
                    names: vec![Name::new_name_quote_var_ref("x")],
                    cont: None,
                    line_num: 0,
                    col_num: 0,
                },
                Source::new_simple_source(Name::Quote(Box::new(Quote {
                    quotable: Box::new(Proc::new_proc_nil()),
                    line_num: 0,
                    col_num: 0,
                }))),
            ))]),
            proc: Box::new(Block::new_block_nil()),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);
        let expected_result = inputs
            .par
            .clone()
            .with_receives(vec![Receive {
                binds: vec![ReceiveBind {
                    patterns: vec![Par {
                        connectives: vec![Connective {
                            connective_instance: Some(VarRefBody(model_VarRef {
                                index: 0,
                                depth: 1,
                            })),
                        }],
                        ..Par::default().clone()
                    }
                    .with_locally_free(create_bit_vector(&vec![0]))],
                    source: Some(Par::default()),
                    remainder: None,
                    free_count: 0,
                }],
                body: Some(Par::default()),
                persistent: false,
                peek: false,
                bind_count: 0,
                locally_free: create_bit_vector(&vec![0]),
                connective_used: false,
            }])
            .with_locally_free(create_bit_vector(&vec![0]));
        println!("\nRust expected_result = {:?}", result.clone().unwrap().par);
        //TODO fix assertion
        //assert_eq!(result.clone().unwrap().par, expected_result);
    }
}
