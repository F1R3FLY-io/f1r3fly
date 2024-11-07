use super::exports::*;
use crate::rust::interpreter::compiler::exports::FreeMap;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, VarSort};
use crate::rust::interpreter::compiler::normalizer::remainder_normalizer_matcher::normalize_remainder;
use crate::rust::interpreter::compiler::rholang_ast::{Collection, KeyValuePair, Proc};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::{EList, ETuple, Expr, Par, Var};
use models::rust::par_map::ParMap;
use models::rust::par_map_type_mapper::ParMapTypeMapper;
use models::rust::par_set::ParSet;
use models::rust::par_set_type_mapper::ParSetTypeMapper;
use models::rust::sorted_par_hash_set::SortedParHashSet;
use models::rust::sorted_par_map::SortedParMap;
use models::rust::utils::union;
use std::collections::HashMap;
use std::result::Result;

pub fn normalize_collection(
    proc: &Collection,
    input: CollectVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<CollectVisitOutputs, InterpreterError> {
    pub fn fold_match<F>(
        known_free: FreeMap<VarSort>,
        elements: &Vec<Proc>,
        constructor: F,
        input: CollectVisitInputs,
        env: &HashMap<String, Par>,
    ) -> Result<CollectVisitOutputs, InterpreterError>
    where
        F: Fn(Vec<Par>, Vec<u8>, bool) -> Expr,
    {
        let init = (vec![], known_free.clone(), Vec::new(), false);
        let (mut acc_pars, mut result_known_free, mut locally_free, mut connective_used) = init;

        for element in elements {
            let result = normalize_match_proc(
                element,
                ProcVisitInputs {
                    par: Par::default(),
                    bound_map_chain: input.bound_map_chain.clone(),
                    free_map: result_known_free.clone(),
                },
                env,
            )?;

            acc_pars.push(result.par.clone());
            result_known_free = result.free_map.clone();
            locally_free = union(locally_free, result.par.locally_free);
            connective_used = connective_used || result.par.connective_used;
        }

        let constructed_expr: Expr = constructor(acc_pars, locally_free, connective_used);
        let expr: Expr = constructed_expr.into();

        Ok(CollectVisitOutputs {
            expr,
            free_map: result_known_free,
        })
    }

    pub fn fold_match_map(
        known_free: FreeMap<VarSort>,
        remainder: Option<Var>,
        pairs: &Vec<KeyValuePair>,
        input: CollectVisitInputs,
        env: &HashMap<String, Par>,
    ) -> Result<CollectVisitOutputs, InterpreterError> {
        let init = (vec![], known_free.clone(), Vec::new(), false);

        let (mut acc_pairs, mut result_known_free, mut locally_free, mut connective_used) = init;

        for key_value_pair in pairs {
            let key_result = normalize_match_proc(
                &key_value_pair.key,
                ProcVisitInputs {
                    par: Par::default(),
                    bound_map_chain: input.bound_map_chain.clone(),
                    free_map: result_known_free.clone(),
                },
                env,
            )?;

            let value_result = normalize_match_proc(
                &key_value_pair.value,
                ProcVisitInputs {
                    par: Par::default(),
                    bound_map_chain: input.bound_map_chain.clone(),
                    free_map: key_result.free_map.clone(),
                },
                env,
            )?;

            acc_pairs.push((key_result.par.clone(), value_result.par.clone()));
            result_known_free = value_result.free_map.clone();
            locally_free = union(
                locally_free,
                union(key_result.par.locally_free, value_result.par.locally_free),
            );
            connective_used = connective_used
                || key_result.par.connective_used
                || value_result.par.connective_used;
        }

        let remainder_connective_used = match remainder {
            Some(ref var) => var.connective_used(var.clone()),
            None => false,
        };

        let remainder_locally_free = match remainder {
            Some(ref var) => var.locally_free(var.clone(), 0),
            None => Vec::new(),
        };

        let expr = Expr {
            expr_instance: Some(ExprInstance::EMapBody(ParMapTypeMapper::par_map_to_emap(
                ParMap {
                    ps: SortedParMap::create_from_vec(
                        acc_pairs.clone().into_iter().rev().collect(),
                    ),
                    connective_used: connective_used || remainder_connective_used,
                    locally_free: union(locally_free, remainder_locally_free),
                    remainder: remainder.clone(),
                },
            ))),
        };

        Ok(CollectVisitOutputs {
            expr,
            free_map: result_known_free,
        })
    }

    match proc {
        Collection::List { elements, cont, .. } => {
            let (optional_remainder, known_free) =
                normalize_remainder(cont, input.free_map.clone())?;

            let constructor =
                |ps: Vec<Par>, locally_free: Vec<u8>, connective_used: bool| -> Expr {
                    let mut tmp_e_list = EList {
                        ps,
                        locally_free,
                        connective_used,
                        remainder: optional_remainder.clone(),
                    };

                    tmp_e_list.connective_used =
                        tmp_e_list.connective_used || optional_remainder.is_some();
                    Expr {
                        expr_instance: Some(ExprInstance::EListBody(tmp_e_list)),
                    }
                };

            fold_match(known_free, elements, constructor, input, env)
        }

        Collection::Tuple { elements, .. } => {
            let constructor =
                |ps: Vec<Par>, locally_free: Vec<u8>, connective_used: bool| -> Expr {
                    let tmp_tuple = ETuple {
                        ps,
                        locally_free,
                        connective_used,
                    };

                    Expr {
                        expr_instance: Some(ExprInstance::ETupleBody(tmp_tuple)),
                    }
                };

            fold_match(input.free_map.clone(), elements, constructor, input, env)
        }

        Collection::Set { elements, cont, .. } => {
            let (optional_remainder, known_free) =
                normalize_remainder(cont, input.free_map.clone())?;

            let constructor =
                |pars: Vec<Par>, locally_free: Vec<u8>, connective_used: bool| -> Expr {
                    let mut tmp_par_set = ParSet {
                        ps: SortedParHashSet::create_from_vec(pars),
                        locally_free,
                        connective_used,
                        remainder: optional_remainder.clone(),
                    };

                    tmp_par_set.connective_used =
                        tmp_par_set.connective_used || optional_remainder.is_some();

                    let eset = ParSetTypeMapper::par_set_to_eset(tmp_par_set);

                    Expr {
                        expr_instance: Some(ExprInstance::ESetBody(eset)),
                    }
                };

            fold_match(known_free, elements, constructor, input, env)
        }

        Collection::Map { pairs, cont, .. } => {
            let (optional_remainder, known_free) =
                normalize_remainder(cont, input.free_map.clone())?;

            fold_match_map(known_free, optional_remainder, pairs, input, env)
        }
    }
}
