// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/normalizer/processes/PInputNormalizer.scala

use std::collections::{HashMap, HashSet};

use models::{
    rhoapi::{Par, ReceiveBind},
    rust::utils::{new_receive_par, union},
    BitSet,
};
use uuid::Uuid;

use crate::rust::interpreter::{
    compiler::{
        exports::FreeMap,
        normalize::{normalize_match_proc, NameVisitInputs, NameVisitOutputs, VarSort},
        normalizer::{
            name_normalize_matcher::normalize_name, processes::utils::fail_on_invalid_connective,
            remainder_normalizer_matcher::normalize_match_name,
        },
        receive_binds_sort_matcher::pre_sort_binds,
        rholang_ast::{
            Block, Decls, Eval, LinearBind, Name, NameDecl, Names, ProcList, Receipt, Receipts,
            SendType, Source, Var,
        },
    },
    matcher::has_locally_free::HasLocallyFree,
    unwrap_option_safe,
    util::filter_and_adjust_bitset,
};

use super::exports::{InterpreterError, Proc, ProcVisitInputs, ProcVisitOutputs};

pub fn normalize_p_input(
    formals: &Receipts,
    body: &Block,
    line_num: usize,
    col_num: usize,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    // Do I return an error for this case?
    if formals.receipts.is_empty() {
        return Err(InterpreterError::BugFoundError(
            "Exepected at least one receipt".to_string(),
        ));
    } else {
        let head_receipt = &formals.receipts[0];

        let receipt_contains_complex_source = match head_receipt {
            Receipt::LinearBinds(linear_bind) => match linear_bind.input {
                Source::Simple { .. } => false,
                _ => true,
            },
            _ => false,
        };

        if receipt_contains_complex_source {
            match head_receipt {
                Receipt::LinearBinds(linear_bind) => match &linear_bind.input {
                    Source::Simple { .. } => {
                        let list_receipt = Receipts {
                            receipts: Vec::new(),
                            line_num: 0,
                            col_num: 0,
                        };
                        let mut list_linear_bind: Vec<Receipt> = Vec::new();
                        let mut list_name_decl = Decls {
                            decls: Vec::new(),
                            line_num: 0,
                            col_num: 0,
                        };

                        let (sends, continuation): (Proc, Proc) =
                            formals.clone().receipts.into_iter().try_fold(
                                (
                                    Proc::Nil {
                                        line_num: 0,
                                        col_num: 0,
                                    },
                                    body.proc.clone(),
                                ),
                                |(sends, continuation), lb| match lb {
                                    Receipt::LinearBinds(linear_bind) => {
                                        let identifier = Uuid::new_v4().to_string();
                                        let r = Proc::Var(Var {
                                            name: identifier.clone(),
                                            line_num: 0,
                                            col_num: 0,
                                        });

                                        match linear_bind.input {
                                            Source::Simple { .. } => {
                                                list_linear_bind
                                                    .push(Receipt::LinearBinds(linear_bind));
                                                Ok((sends, continuation))
                                            }

                                            Source::ReceiveSend { name, .. } => {
                                                let mut list_name = linear_bind.names.names;
                                                list_name.push(Name::ProcVar(Box::new(r.clone())));

                                                list_linear_bind.push(Receipt::LinearBinds(
                                                    LinearBind {
                                                        names: Names {
                                                            names: list_name,
                                                            cont: linear_bind.names.cont,
                                                            line_num: 0,
                                                            col_num: 0,
                                                        },
                                                        input: Source::Simple {
                                                            name,
                                                            line_num: 0,
                                                            col_num: 0,
                                                        },
                                                        line_num: 0,
                                                        col_num: 0,
                                                    },
                                                ));

                                                Ok((
                                                    sends,
                                                    Proc::Par {
                                                        left: Box::new(Proc::Send {
                                                            name: Name::ProcVar(Box::new(r)),
                                                            send_type: SendType::Single {
                                                                line_num: 0,
                                                                col_num: 0,
                                                            },
                                                            inputs: ProcList {
                                                                procs: Vec::new(),
                                                                line_num: 0,
                                                                col_num: 0,
                                                            },
                                                            line_num: 0,
                                                            col_num: 0,
                                                        }),
                                                        right: Box::new(continuation),
                                                        line_num: 0,
                                                        col_num: 0,
                                                    },
                                                ))
                                            }

                                            Source::SendReceive { name, inputs, .. } => {
                                                list_name_decl.decls.push(NameDecl {
                                                    var: Var {
                                                        name: identifier,
                                                        line_num: 0,
                                                        col_num: 0,
                                                    },
                                                    uri: None,
                                                    line_num: 0,
                                                    col_num: 0,
                                                });

                                                list_linear_bind.push(Receipt::LinearBinds(
                                                    LinearBind {
                                                        names: Names {
                                                            names: linear_bind.names.names,
                                                            cont: linear_bind.names.cont,
                                                            line_num: 0,
                                                            col_num: 0,
                                                        },
                                                        input: Source::Simple {
                                                            name: Name::ProcVar(Box::new(
                                                                r.clone(),
                                                            )),
                                                            line_num: 0,
                                                            col_num: 0,
                                                        },
                                                        line_num: 0,
                                                        col_num: 0,
                                                    },
                                                ));

                                                let mut list_proc = inputs.procs;
                                                list_proc.insert(
                                                    0,
                                                    Proc::Eval(Eval {
                                                        name: Name::ProcVar(Box::new(r)),
                                                        line_num: 0,
                                                        col_num: 0,
                                                    }),
                                                );

                                                Ok((
                                                    Proc::Par {
                                                        left: Box::new(Proc::Send {
                                                            name,
                                                            send_type: SendType::Single {
                                                                line_num: 0,
                                                                col_num: 0,
                                                            },
                                                            inputs: ProcList {
                                                                procs: list_proc,
                                                                line_num: 0,
                                                                col_num: 0,
                                                            },
                                                            line_num: 0,
                                                            col_num: 0,
                                                        }),
                                                        right: Box::new(sends),
                                                        line_num: 0,
                                                        col_num: 0,
                                                    },
                                                    continuation,
                                                ))
                                            }
                                        }
                                    }

                                    _ => Err(InterpreterError::BugFoundError(format!(
                                        "Expected LinearBinds, found {:?}",
                                        &lb
                                    ))),
                                },
                            )?;

                        let p_input = Proc::Input {
                            formals: list_receipt,
                            proc: Box::new(Block {
                                proc: continuation,
                                line_num: 0,
                                col_num: 0,
                            }),
                            line_num: 0,
                            col_num: 0,
                        };

                        let p_new = Proc::New {
                            decls: list_name_decl.clone(),
                            proc: Box::new(Proc::Par {
                                left: Box::new(sends),
                                right: Box::new(p_input.clone()),
                                line_num: 0,
                                col_num: 0,
                            }),
                            line_num: 0,
                            col_num: 0,
                        };

                        normalize_match_proc(
                            {
                                if list_name_decl.decls.is_empty() {
                                    &p_input
                                } else {
                                    &p_new
                                }
                            },
                            input,
                            env,
                        )
                    }

                    _ => {
                        return Err(InterpreterError::BugFoundError(format!(
                            "Expected SimpleSource, found {:?}",
                            &linear_bind.input
                        )))
                    }
                },
                _ => {
                    return Err(InterpreterError::BugFoundError(format!(
                        "Expected LinearBinds, found {:?}",
                        &formals.receipts[0]
                    )))
                }
            }
        } else {
            // To handle the most common case where we can sort the binds because
            // they're from different sources, Each channel's list of patterns starts its free variables at 0.
            // We check for overlap at the end after sorting. We could check before, but it'd be an extra step.
            // We split this into parts. First we process all the sources, then we process all the bindings.
            fn process_sources(
                sources: Vec<Name>,
                input: ProcVisitInputs,
                env: &HashMap<String, Par>,
            ) -> Result<(Vec<Par>, FreeMap<VarSort>, BitSet, bool), InterpreterError> {
                let mut vector_par = Vec::new();
                let mut current_known_free = input.free_map;
                let mut locally_free = Vec::new();
                let mut connective_used = false;

                for name in sources {
                    let NameVisitOutputs {
                        par,
                        free_map: updated_known_free,
                    } = normalize_name(
                        &name,
                        NameVisitInputs {
                            bound_map_chain: input.bound_map_chain.clone(),
                            free_map: current_known_free,
                        },
                        env,
                    )?;

                    vector_par.push(par.clone());
                    current_known_free = updated_known_free;
                    locally_free = union(
                        locally_free,
                        par.locally_free(par.clone(), input.bound_map_chain.depth() as i32),
                    );
                    connective_used = connective_used || par.clone().connective_used(par);
                }

                Ok((
                    vector_par,
                    current_known_free,
                    locally_free,
                    connective_used,
                ))
            }

            fn process_patterns(
                patterns: Vec<(Vec<Name>, Option<Box<Proc>>)>,
                input: ProcVisitInputs,
                env: &HashMap<String, Par>,
            ) -> Result<
                Vec<(
                    Vec<Par>,
                    Option<models::rhoapi::Var>,
                    FreeMap<VarSort>,
                    BitSet,
                )>,
                InterpreterError,
            > {
                patterns
                    .into_iter()
                    .map(|(names, name_remainder)| {
                        let mut vector_par = Vec::new();
                        let mut known_free = FreeMap::new();
                        let mut locally_free = Vec::new();

                        for name in names {
                            let NameVisitOutputs {
                                par,
                                free_map: updated_known_free,
                            } = normalize_name(
                                &name,
                                NameVisitInputs {
                                    bound_map_chain: input.bound_map_chain.push(),
                                    free_map: known_free,
                                },
                                env,
                            )?;

                            fail_on_invalid_connective(
                                &input,
                                &NameVisitOutputs {
                                    par: par.clone(),
                                    free_map: updated_known_free.clone(),
                                },
                            )?;

                            vector_par.push(par.clone());
                            known_free = updated_known_free;
                            locally_free = union(
                                locally_free,
                                par.locally_free(
                                    par.clone(),
                                    input.bound_map_chain.depth() as i32 + 1,
                                ),
                            );
                        }

                        let (optional_var, known_free) =
                            normalize_match_name(&name_remainder, known_free)?;
                        Ok((vector_par, optional_var, known_free, locally_free))
                    })
                    .collect()
            }

            let (consumes, persistent, peek): (
                Vec<((Vec<Name>, Option<Box<Proc>>), Name)>,
                bool,
                bool,
            ) = {
                let consumes: Vec<((Vec<Name>, Option<Box<Proc>>), Name)> = formals
                    .receipts
                    .clone()
                    .into_iter()
                    .map(|receipt| match receipt {
                        Receipt::LinearBinds(linear_bind) => {
                            ((linear_bind.names.names, linear_bind.names.cont), {
                                match linear_bind.input {
                                    Source::Simple { name, .. } => name,
                                    Source::ReceiveSend { name, .. } => name,
                                    Source::SendReceive { name, .. } => name,
                                }
                            })
                        }

                        Receipt::RepeatedBinds(repeated_bind) => (
                            (repeated_bind.names.names, repeated_bind.names.cont),
                            repeated_bind.input,
                        ),

                        Receipt::PeekBinds(peek_bind) => (
                            (peek_bind.names.names, peek_bind.names.cont),
                            peek_bind.input,
                        ),
                    })
                    .collect();

                match head_receipt {
                    Receipt::LinearBinds(_) => (consumes, false, false),
                    Receipt::RepeatedBinds(_) => (consumes, true, false),
                    Receipt::PeekBinds(_) => (consumes, false, true),
                }
            };

            let (patterns, names): (Vec<(Vec<Name>, Option<Box<Proc>>)>, Vec<Name>) =
                consumes.into_iter().unzip();

            let processed_patterns = process_patterns(patterns, input.clone(), env)?;
            let processed_sources = process_sources(names, input.clone(), env)?;
            let (sources, sources_free, sources_locally_free, sources_connective_used) =
                processed_sources;

            let receive_binds_and_free_maps = pre_sort_binds(
                processed_patterns
                    .clone()
                    .into_iter()
                    .zip(sources)
                    .into_iter()
                    .map(|((a, b, c, _), e)| (a, b, e, c))
                    .collect(),
            )?;

            let (receive_binds, receive_bind_free_maps): (Vec<ReceiveBind>, Vec<FreeMap<VarSort>>) =
                receive_binds_and_free_maps.into_iter().unzip();

            let channels: Vec<Par> = receive_binds
                .clone()
                .into_iter()
                .map(|rb| rb.source.unwrap())
                .collect();

            let channels_set: HashSet<Par> = channels.clone().into_iter().collect();
            let has_same_channels = channels.len() > channels_set.len();

            if has_same_channels {
                return Err(InterpreterError::ReceiveOnSameChannelsError {
                    line: line_num,
                    col: col_num,
                });
            }

            let receive_binds_free_map = receive_bind_free_maps.into_iter().try_fold(
                FreeMap::new(),
                |mut known_free, receive_bind_free_map| {
                    let (updated_known_free, conflicts) = known_free.merge(receive_bind_free_map);

                    if conflicts.is_empty() {
                        Ok(updated_known_free)
                    } else {
                        let (shadowing_var, source_position) = &conflicts[0];
                        let original_position =
                            unwrap_option_safe(known_free.get(shadowing_var))?.source_position;
                        Err(InterpreterError::UnexpectedReuseOfNameContextFree {
                            var_name: shadowing_var.to_string(),
                            first_use: original_position.to_string(),
                            second_use: source_position.to_string(),
                        })
                    }
                },
            )?;

            // println!(
            //     "\nfree_map: {:?}",
            //     input
            //         .bound_map_chain
            //         .absorb_free(receive_binds_free_map.clone()),
            // );
            // println!("\nsources_free: {:?}", sources_free);

            let proc_visit_outputs = normalize_match_proc(
                &body.proc,
                ProcVisitInputs {
                    par: Par::default(),
                    bound_map_chain: input
                        .bound_map_chain
                        .absorb_free(receive_binds_free_map.clone()),
                    free_map: sources_free,
                },
                env,
            )?;

            let bind_count = receive_binds_free_map.count_no_wildcards();

            Ok(ProcVisitOutputs {
                par: new_receive_par(
                    receive_binds,
                    proc_visit_outputs.clone().par,
                    persistent,
                    peek,
                    bind_count as i32,
                    {
                        union(
                            sources_locally_free,
                            filter_and_adjust_bitset(
                                processed_patterns
                                    .into_iter()
                                    .map(|pattern| pattern.3)
                                    .fold(Vec::new(), |locally_free1, locally_free2| {
                                        union(locally_free1, locally_free2)
                                    }),
                                bind_count,
                            ),
                        )
                    },
                    sources_connective_used || proc_visit_outputs.par.connective_used,
                    Vec::new(),
                    false,
                ),
                free_map: proc_visit_outputs.free_map,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use models::{
        create_bit_vector,
        rhoapi::Receive,
        rust::utils::{new_boundvar_par, new_freevar_par, new_send_par},
    };

    use crate::rust::interpreter::compiler::exports::BoundMapChain;

    use super::*;

    fn inputs() -> ProcVisitInputs {
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: BoundMapChain::new(),
            free_map: FreeMap::new(),
        }
    }

    #[test]
    fn p_input_should_handle_a_simple_receive() {
        // for ( x, y <- @Nil ) { x!(*y) }
        let mut list_bindings: Vec<Name> = Vec::new();
        list_bindings.push(Name::new_name_var("x", 0, 0));
        list_bindings.push(Name::new_name_var("y", 0, 0));

        let mut list_linear_binds: Vec<Receipt> = Vec::new();
        list_linear_binds.push(Receipt::LinearBinds(LinearBind {
            names: Names::create(list_bindings, None, 0, 0),
            input: Source::new_simple_source(Name::new_name_quote_nil(0, 0), 0, 0),
            line_num: 0,
            col_num: 0,
        }));

        let body = Proc::Send {
            name: Name::new_name_var("x", 0, 0),
            send_type: SendType::Single {
                line_num: 0,
                col_num: 0,
            },
            inputs: ProcList::create(
                vec![Proc::Eval(Eval {
                    name: Name::new_name_var("y", 0, 0),
                    line_num: 0,
                    col_num: 0,
                })],
                0,
                0,
            ),
            line_num: 0,
            col_num: 0,
        };

        let basic_input = Proc::Input {
            formals: Receipts {
                receipts: list_linear_binds,
                line_num: 0,
                col_num: 0,
            },
            proc: Box::new(Block {
                proc: body,
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        };

        let bind_count = 2;

        let result = normalize_match_proc(&basic_input, inputs(), &HashMap::new());
        assert!(result.is_ok());

        let expected_result = inputs().par.prepend_receive(Receive {
            binds: vec![ReceiveBind {
                patterns: vec![
                    new_freevar_par(0, Vec::new()),
                    new_freevar_par(1, Vec::new()),
                ],
                source: Some(Par::default()),
                remainder: None,
                free_count: 2,
            }],
            body: Some(new_send_par(
                new_boundvar_par(1, create_bit_vector(&vec![1]), false),
                vec![new_boundvar_par(0, create_bit_vector(&vec![0]), false)],
                false,
                create_bit_vector(&vec![0, 1]),
                false,
                create_bit_vector(&vec![0, 1]),
                false,
            )),
            persistent: false,
            peek: false,
            bind_count,
            locally_free: Vec::new(),
            connective_used: false,
        });

        // println!("\nresult: {:#?}", result.clone().unwrap().par);
        // println!("\nexpected_result: {:#?}", expected_result);

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs().free_map);
    }
}
