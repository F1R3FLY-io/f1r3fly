// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/normalizer/processes/PInputNormalizer.scala

use std::collections::HashMap;

use models::{rhoapi::Par, rust::utils::union, BitSet};
use uuid::Uuid;

use crate::rust::interpreter::{
    compiler::{
        exports::FreeMap,
        normalize::{normalize_match_proc, NameVisitInputs, NameVisitOutputs, VarSort},
        normalizer::{
            name_normalize_matcher::normalize_name, processes::utils::fail_on_invalid_connective,
            remainder_normalizer_matcher::normalize_match_name,
        },
        rholang_ast::{
            Block, Decls, Eval, LinearBind, Name, NameDecl, Names, ProcList, Receipt, Receipts,
            SendType, Source, Var,
        },
    },
    matcher::has_locally_free::HasLocallyFree,
};

use super::exports::{InterpreterError, Proc, ProcVisitInputs, ProcVisitOutputs};

// TODO: Review
pub fn normalize_p_input(
    formals: &Receipts,
    body: &Block,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    if formals.receipts.len() > 1 {
        let folded_proc: Proc = formals
            .clone()
            .receipts
            .into_iter()
            .rev()
            // I'm not sure if this is corrrectly ported, should 'receipts' be better collected?
            .fold(body.proc.clone(), |proc, receipt| Proc::Input {
                formals: Receipts {
                    receipts: vec![receipt],
                    line_num: 0,
                    col_num: 0,
                },
                proc: Box::new(Block {
                    proc,
                    line_num: 0,
                    col_num: 0,
                }),
                line_num: 0,
                col_num: 0,
            });

        normalize_match_proc(&folded_proc, input, env)
    } else {
        let receipt_contains_complex_source = match &formals.receipts[0] {
            Receipt::LinearBinds(linear_bind) => match linear_bind.input {
                Source::Simple { .. } => false,
                _ => true,
            },
            _ => false,
        };

        if receipt_contains_complex_source {
            match &formals.receipts[0] {
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

            // If we get to this point, we know p.listreceipt.size() == 1
            // TODO: Implement
            let (consumes, persistent, peek): (
                Vec<((Vec<Name>, Option<Box<Proc>>), Name)>,
                bool,
                bool,
            ) = (Vec::new(), false, false);

            let (patterns, names): (Vec<(Vec<Name>, Option<Box<Proc>>)>, Vec<Name>) =
                consumes.into_iter().unzip();

            let processed_sources = process_sources(names, input.clone(), env)?;
            let (sources, sources_free, sources_locally_free, sources_connective_used) =
                processed_sources;
            let processed_patterns = process_patterns(patterns, input, env)?;

            todo!()
        }
    }
}
