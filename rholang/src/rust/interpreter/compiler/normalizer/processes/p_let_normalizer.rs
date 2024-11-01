// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/normalizer/processes/PLetNormalizer.scala

use std::collections::HashMap;

use models::{
    rhoapi::{MatchCase, Par},
    rust::utils::{new_elist_par, new_match_par, union},
};
use uuid::Uuid;

use crate::rust::interpreter::{
    compiler::{
        exports::FreeMap,
        normalize::{normalize_match_proc, NameVisitInputs, NameVisitOutputs, VarSort},
        normalizer::{
            name_normalize_matcher::normalize_name,
            remainder_normalizer_matcher::normalize_match_name,
        },
        rholang_ast::{
            Block, Decl, Decls, DeclsChoice, LinearBind, Name, NameDecl, Names, Proc, ProcList,
            Receipt, Receipts, SendType, Source, Var,
        },
    },
    matcher::has_locally_free::HasLocallyFree,
    util::filter_and_adjust_bitset,
};

use super::exports::{InterpreterError, ProcVisitInputs, ProcVisitOutputs};

// TODO: This file is going to need a review because we do not have a single 'decl' on our 'let' rule
pub fn normalize_p_let(
    decls_choice: &DeclsChoice,
    body: &Block,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    type ListName = Vec<Name>;
    type NameRemainder = Option<Box<Proc>>;
    type ListProc = Vec<Proc>;

    match decls_choice.clone() {
        DeclsChoice::ConcDecls { decls, .. } => {
            fn extract_names_and_procs(decl: Decl) -> (ListName, NameRemainder, ListProc) {
                (decl.names.names, decl.names.cont, decl.procs)
            }

            // We don't have a single 'decl' on 'let' rule in grammar, so I assume we just map over 'decls'
            let (list_names, list_name_remainders, list_procs): (
                Vec<ListName>,
                Vec<NameRemainder>,
                Vec<ListProc>,
            ) = {
                let tuples: Vec<(ListName, NameRemainder, ListProc)> = decls
                    .into_iter()
                    .map(|conc_decl_impl| extract_names_and_procs(conc_decl_impl))
                    .collect();
                unzip3(tuples)
            };

            /*
            It is not necessary to use UUIDs to achieve concurrent let declarations.
            While there is the possibility for collisions with either variables declared by the user
            or variables declared within this translation, the chances for collision are astronomically
            small (see analysis here: https://towardsdatascience.com/are-uuids-really-unique-57eb80fc2a87).
            A strictly correct approach would be one that performs a ADT rather than an AST translation, which
            was not done here due to time constraints. - OLD
            */
            let variable_names: Vec<String> = (0..list_names.len())
                .map(|_| Uuid::new_v4().to_string())
                .collect();

            let p_sends: Vec<Proc> = variable_names
                .clone()
                .into_iter()
                .zip(list_procs)
                .map(|(variable_name, list_proc)| Proc::Send {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: variable_name,
                        line_num: 0,
                        col_num: 0,
                    }))),
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
                })
                .collect();

            let p_input = {
                let list_linear_bind: Vec<Receipt> = variable_names
                    .clone()
                    .into_iter()
                    .zip(list_names)
                    .zip(list_name_remainders)
                    .map(|((variable_name, list_name), name_remainder)| {
                        Receipt::LinearBinds(LinearBind {
                            names: Names {
                                names: list_name,
                                cont: name_remainder,
                                line_num: 0,
                                col_num: 0,
                            },
                            input: Source::Simple {
                                name: Name::ProcVar(Box::new(Proc::Var(Var {
                                    name: variable_name,
                                    line_num: 0,
                                    col_num: 0,
                                }))),
                                line_num: 0,
                                col_num: 0,
                            },
                            line_num: 0,
                            col_num: 0,
                        })
                    })
                    .collect();

                let list_receipt = Receipts {
                    receipts: list_linear_bind,
                    line_num: 0,
                    col_num: 0,
                };

                Proc::Input {
                    formals: list_receipt,
                    proc: Box::new(body.clone()),
                    line_num: 0,
                    col_num: 0,
                }
            };

            let p_par = {
                let procs = {
                    let mut combined = p_sends;
                    combined.push(p_input);
                    combined
                };

                procs.clone().into_iter().skip(2).fold(
                    Proc::Par {
                        left: Box::new(procs[0].clone()),
                        right: Box::new(procs[1].clone()),
                        line_num: 0,
                        col_num: 0,
                    },
                    |p_par, proc| Proc::Par {
                        left: Box::new(p_par),
                        right: Box::new(proc),
                        line_num: 0,
                        col_num: 0,
                    },
                )
            };

            let p_new = Proc::New {
                decls: Decls {
                    decls: variable_names
                        .into_iter()
                        .map(|name| NameDecl {
                            var: Var {
                                name,
                                line_num: 0,
                                col_num: 0,
                            },
                            uri: None,
                            line_num: 0,
                            col_num: 0,
                        })
                        .collect(),
                    line_num: 0,
                    col_num: 0,
                },
                proc: Box::new(p_par),
                line_num: 0,
                col_num: 0,
            };

            normalize_match_proc(&p_new, input, env)
        }

        /*
        Let processes with a single bind or with sequential binds ";" are converted into match processes rather
        than input processes, so that each sequential bind doesn't add a new unforgeable name to the tuplespace.
        The Rholang 1.1 spec defines them as the latter. Because the Rholang 1.1 spec defines let processes in terms
        of a output process in concurrent composition with an input process, the let process appears to quote the
        process on the RHS of "<-" and bind it to the pattern on LHS. For example, in
            let x <- 1 in { Nil }
        the process (value) "1" is quoted and bound to "x" as a name. There is no way to perform an AST transformation
        of sequential let into a match process and still preserve these semantics, so we have to do an ADT transformation.
         */
        DeclsChoice::LinearDecls { decls, .. } => {
            let new_continuation = if decls.is_empty() {
                body.proc.clone()
            } else {
                // Similarly here, we don't have a single 'decl' field on `let' rule.
                let new_decls: Vec<Decl> = if decls.len() == 1 {
                    decls.clone()
                } else {
                    let mut new_linear_decls = Vec::new();
                    for decl in decls.clone().into_iter().skip(1) {
                        new_linear_decls.push(decl);
                    }
                    new_linear_decls
                };

                Proc::Let {
                    decls: DeclsChoice::LinearDecls {
                        decls: new_decls,
                        line_num: 0,
                        col_num: 0,
                    },
                    body: Box::new(body.clone()),
                    line_num: 0,
                    col_num: 0,
                }
            };

            fn list_proc_to_elist(
                list_proc: Vec<Proc>,
                known_free: FreeMap<VarSort>,
                input: ProcVisitInputs,
                env: &HashMap<String, Par>,
            ) -> Result<ProcVisitOutputs, InterpreterError> {
                let mut vector_par = Vec::new();
                let mut current_known_free = known_free;
                let mut locally_free = Vec::new();
                let mut connective_used = false;

                for proc in list_proc {
                    let ProcVisitOutputs {
                        par,
                        free_map: updated_known_free,
                    } = normalize_match_proc(
                        &proc,
                        ProcVisitInputs {
                            par: Par::default(),
                            bound_map_chain: input.bound_map_chain.clone(),
                            free_map: current_known_free,
                        },
                        env,
                    )?;

                    vector_par.insert(0, par.clone());
                    current_known_free = updated_known_free;
                    locally_free = union(locally_free, par.locally_free);
                    connective_used |= par.connective_used;
                }

                Ok(ProcVisitOutputs {
                    par: new_elist_par(
                        vector_par.into_iter().rev().collect(),
                        locally_free,
                        connective_used,
                        None,
                        Vec::new(),
                        false,
                    ),
                    free_map: current_known_free,
                })
            }

            fn list_name_to_elist(
                list_name: Vec<Name>,
                name_remainder: &NameRemainder,
                input: ProcVisitInputs,
                env: &HashMap<String, Par>,
            ) -> Result<ProcVisitOutputs, InterpreterError> {
                let (optional_var, remainder_known_free) =
                    normalize_match_name(name_remainder, FreeMap::new())?;

                let mut vector_par = Vec::new();
                let mut current_known_free = remainder_known_free;
                let mut locally_free = Vec::new();

                for name in list_name {
                    let NameVisitOutputs {
                        par,
                        free_map: updated_known_free,
                    } = normalize_name(
                        &name,
                        NameVisitInputs {
                            bound_map_chain: input.bound_map_chain.push(),
                            free_map: current_known_free,
                        },
                        env,
                    )?;

                    vector_par.insert(0, par.clone());
                    current_known_free = updated_known_free;
                    // Use input.env.depth + 1 because the pattern was evaluated w.r.t input.env.push,
                    // and more generally because locally free variables become binders in the pattern position
                    locally_free = union(
                        locally_free,
                        par.clone()
                            .locally_free(par, input.bound_map_chain.depth() as i32 + 1),
                    );
                }

                Ok(ProcVisitOutputs {
                    par: new_elist_par(
                        vector_par.into_iter().rev().collect(),
                        locally_free,
                        true,
                        optional_var,
                        Vec::new(),
                        false,
                    ),
                    free_map: current_known_free,
                })
            }

            // Again, we don't have a single 'decl' field on 'let' I am using the first 'decl' element
            let decl = decls[0].clone();

            list_proc_to_elist(decl.procs, input.clone().free_map, input.clone(), env).and_then(
                |ProcVisitOutputs {
                     par: value_list_par,
                     free_map: value_known_free,
                 }| {
                    list_name_to_elist(decl.names.names, &decl.names.cont, input.clone(), env)
                        .and_then(
                            |ProcVisitOutputs {
                                 par: pattern_list_par,
                                 free_map: pattern_known_free,
                             }| {
                                normalize_match_proc(
                                    &new_continuation,
                                    ProcVisitInputs {
                                        par: Par::default(),
                                        bound_map_chain: input
                                            .bound_map_chain
                                            .absorb_free(pattern_known_free.clone()),
                                        free_map: value_known_free,
                                    },
                                    env,
                                )
                                .map(
                                    |ProcVisitOutputs {
                                         par: continuation_par,
                                         free_map: continuation_known_free,
                                     }| ProcVisitOutputs {
                                        par: new_match_par(
                                            value_list_par.clone(),
                                            vec![MatchCase {
                                                pattern: Some(pattern_list_par.clone()),
                                                source: Some(continuation_par.clone()),
                                                free_count: pattern_known_free.count_no_wildcards()
                                                    as i32,
                                            }],
                                            union(
                                                value_list_par.locally_free,
                                                union(
                                                    pattern_list_par.locally_free,
                                                    filter_and_adjust_bitset(
                                                        continuation_par.locally_free,
                                                        pattern_known_free.count_no_wildcards(),
                                                    ),
                                                ),
                                            ),
                                            value_list_par.connective_used
                                                || continuation_par.connective_used,
                                            Vec::new(),
                                            false,
                                        ),
                                        free_map: continuation_known_free,
                                    },
                                )
                            },
                        )
                },
            )
        }
    }
}

fn unzip3<T, U, V>(tuples: Vec<(Vec<T>, U, Vec<V>)>) -> (Vec<Vec<T>>, Vec<U>, Vec<Vec<V>>) {
    let mut vec_t = Vec::with_capacity(tuples.len());
    let mut vec_u = Vec::with_capacity(tuples.len());
    let mut vec_v = Vec::with_capacity(tuples.len());

    for (t, u, v) in tuples {
        vec_t.push(t);
        vec_u.push(u);
        vec_v.push(v);
    }

    (vec_t, vec_u, vec_v)
}
