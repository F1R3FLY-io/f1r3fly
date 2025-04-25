pub mod collection_normalize_matcher;
pub mod exports;
pub mod name_normalize_matcher;
pub mod parser;
pub mod processes;
pub mod remainder_normalizer_matcher;

use stacker::maybe_grow;

use super::exports::*;
use super::normalizer::collection_normalize_matcher::{
    normalize_c_list, normalize_c_set, normalize_c_tuple,
};
use super::rholang_ast::{Collection, Proc, SimpleType};
use crate::aliases::EnvHashMap;
use crate::compiler::bound_map::BoundContext;
use crate::compiler::normalizer::name_normalize_matcher::normalize_name;
use crate::compiler::normalizer::processes::p_input_normalizer::normalize_p_input;
use crate::compiler::normalizer::processes::p_let_normalizer::{
    normalize_p_concurrent_let, normalize_p_let,
};
use crate::compiler::normalizer::processes::p_var_ref_normalizer::normalize_p_var_ref;
use crate::compiler::rholang_ast::{AnnProc, Var};
use crate::errors::InterpreterError;
use crate::normal_forms::{self, Connective, Expr, Par, Var as NormalizedVar};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundVar {
    ProcBound,
    NameBound(Option<String>),
}

const MINIMUM_STACK_SIZE: usize = 512;
const STACK_ALLOC_SIZE: usize = 4 * 1024 * 1024;

/**
 * Rholang normalizer entry point
 */
#[must_use]
pub(crate) fn normalize_match_proc(
    proc: &Proc,
    input_par: &mut crate::normal_forms::Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap<String, crate::normal_forms::Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    maybe_grow(MINIMUM_STACK_SIZE, STACK_ALLOC_SIZE, || {
        normalize_match_proc_internal(proc, &mut input_par, free_map, bound_map_chain, env, pos)
    })
}

/**
 * Rholang normalizer entry point
 */
#[must_use]
fn normalize_match_proc_internal(
    proc: &Proc,
    input_par: &mut crate::normal_forms::Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &EnvHashMap,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    fn binary_exp<C>(
        left: AnnProc,
        right: AnnProc,
        input_par: &mut Par,
        constr: C,
        free_map: &mut FreeMap,
        bound_map_chain: &mut BoundMapChain,
        env: &EnvHashMap,
    ) -> Result<(), InterpreterError>
    where
        C: FnOnce(Par, Par) -> Expr,
    {
        let depth = bound_map_chain.depth();
        let mut p1 = Par::default();
        normalize_match_proc(left.proc, &mut p1, free_map, bound_map_chain, env, left.pos)?;
        let mut p2 = Par::default();
        // these operations are left leaning, so there is no need to check for remaining stack on
        // the right branch
        normalize_match_proc_internal(
            right.proc,
            &mut p2,
            free_map,
            bound_map_chain,
            env,
            right.pos,
        )?;
        let e = constr(p1, p2);
        input_par.push_expr(e, depth);
        Ok(())
    }

    let depth = bound_map_chain.depth();

    match proc {
        Proc::Par { left, right } => {
            normalize_match_proc(
                left.proc,
                input_par,
                free_map,
                bound_map_chain,
                env,
                left.pos,
            )?;
            // these operations are left leaning, so there is no need to check for remaining stack on
            // the right branch
            normalize_match_proc_internal(
                right.proc,
                input_par,
                free_map,
                bound_map_chain,
                env,
                right.pos,
            )?;
            Ok(())
        }

        Proc::SendSync {
            name,
            messages,
            cont,
        } => normalize_p_send_sync(
            *name,
            messages,
            *cont,
            input_par,
            free_map,
            bound_map_chain,
            env,
            pos,
        ),

        Proc::New { decls, proc } => {
            normalize_p_new(decls, *proc, input_par, free_map, bound_map_chain, env, pos)
        }

        Proc::IfThenElse {
            condition,
            if_true,
            if_false,
        } => normalize_p_if(
            *condition,
            *if_true,
            *if_false,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),

        Proc::Let {
            bindings,
            body,
            concurrent,
        } => {
            if *concurrent {
                normalize_p_concurrent_let(
                    bindings,
                    *body,
                    input_par,
                    free_map,
                    bound_map_chain,
                    env,
                    pos,
                )
            } else {
                normalize_p_let(bindings, *body, input_par, free_map, bound_map_chain, env)
            }
        }

        Proc::Bundle { bundle_type, proc } => normalize_p_bundle(
            *proc,
            *bundle_type,
            input_par,
            free_map,
            bound_map_chain,
            env,
            pos,
        ),

        Proc::Match { expression, cases } => normalize_p_match(
            *expression,
            cases,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),

        // I don't think the previous scala developers implemented a normalize function for this
        Proc::Choice { branches } => todo!(),

        Proc::Contract {
            name,
            formals,
            body,
        } => normalize_p_contr(
            *name,
            *formals,
            *body,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),

        Proc::ForComprehension {
            receipts: formals,
            proc,
        } => normalize_p_input(formals, proc, *line_num, *col_num, input, env),

        Proc::Send {
            name,
            send_type,
            inputs,
        } => normalize_p_send(
            *name,
            *send_type,
            inputs,
            input_par,
            free_map,
            bound_map_chain,
            env,
            pos,
        ),

        Proc::Matches { target, pattern } => {
            normalize_p_matches(*target, *pattern, input_par, free_map, bound_map_chain, env)
        }

        // binary
        Proc::Mult { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EMult,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Interpolation { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EPercentPercent,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Sub { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EMinus,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Concat { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EMult,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Diff { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EMult,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Div { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EDiv,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Mod { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EMod,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Add { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EPlus,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Or { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EOr,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::And { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EAnd,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Eq { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EEq,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Neq { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::ENeq,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Lt { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::ELt,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Lte { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::ELte,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Gt { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EGt,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Gte { left, right } => binary_exp(
            *left,
            *right,
            input_par,
            Expr::EGte,
            free_map,
            bound_map_chain,
            env,
        ),

        // unary
        Proc::Not(sub_proc) => {
            let mut sub = Par::default();
            normalize_match_proc_internal(
                *sub_proc,
                &mut sub,
                free_map,
                bound_map_chain,
                env,
                pos,
            )?;
            input_par.push_expr(Expr::ENot(sub), depth);
            Ok(())
        }
        Proc::Neg(sub_proc) => {
            let mut sub = Par::default();
            normalize_match_proc_internal(
                *sub_proc,
                &mut sub,
                free_map,
                bound_map_chain,
                env,
                pos,
            )?;
            input_par.push_expr(Expr::ENeg(sub), depth);
            Ok(())
        }

        Proc::Method {
            receiver,
            name,
            args,
        } => normalize_p_method(
            *receiver,
            *name,
            args,
            input_par,
            free_map,
            bound_map_chain,
            env,
            pos,
        ),

        Proc::Eval { name } => {
            let name_match_result = normalize_name(name.0, free_map, bound_map_chain, env, name.1)?;
            input_par.concat_with(name_match_result);
            Ok(())
        }
        Proc::Quote(quote) => {
            normalize_match_proc_internal(*quote, input_par, free_map, bound_map_chain, env, pos)
        }

        Proc::Disjunction { left, right } => normalize_p_disjunction(
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
            pos,
        ),
        Proc::Conjunction { left, right } => normalize_p_conjunction(
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
            pos,
        ),
        Proc::Negation(body) => {
            normalize_p_negation(*body, input_par, free_map, bound_map_chain, env, pos)
        }
        Proc::SimpleType(t) => {
            match t {
                SimpleType::Bool => {
                    input_par.push_connective(Connective::ConnBool(Default::default()), depth)
                }
                SimpleType::Int => {
                    input_par.push_connective(Connective::ConnInt(Default::default()), depth)
                }
                SimpleType::String => {
                    input_par.push_connective(Connective::ConnString(Default::default()), depth)
                }
                SimpleType::Uri => {
                    input_par.push_connective(Connective::ConnUri(Default::default()), depth)
                }
                SimpleType::ByteArray => {
                    input_par.push_connective(Connective::ConnByteArray(Default::default()), depth)
                }
            };
            Ok(())
        }

        Proc::Collection(collection) => {
            let collection_expr = match collection {
                Collection::List { elements, cont } => {
                    normalize_c_list(elements, *cont, free_map, bound_map_chain, env)
                }
                Collection::Tuple { elements } => {
                    normalize_c_tuple(elements, free_map, bound_map_chain, env)
                }
                Collection::Set { elements, cont } => {
                    normalize_c_set(elements, *cont, free_map, bound_map_chain, env)
                }
                Collection::Map { pairs, cont } => todo!(),
            }?;
            input_par.push_expr(collection_expr, depth);
            Ok(())
        }

        Proc::BoolLiteral(value) => {
            input_par.push_expr(if *value { Expr::GTRUE } else { Expr::GFALSE }, depth);
            Ok(())
        }
        Proc::LongLiteral(value) => {
            input_par.push_expr(Expr::GInt(*value), depth);
            Ok(())
        }
        Proc::StringLiteral(value) => {
            input_par.push_expr(Expr::GString(value.to_string()), depth);
            Ok(())
        }
        Proc::UriLiteral(value) => {
            input_par.push_expr(Expr::GUri(value.to_string()), depth);
            Ok(())
        }

        Proc::Nil => Ok(()),

        Proc::ProcVar(Var::Wildcard) => {
            input_par.push_expr(Expr::WILDCARD, depth);
            free_map.add_wildcard(pos);
            Ok(())
        }
        Proc::ProcVar(Var::Id(id)) => {
            let normalized_var = match bound_map_chain.get(id.name) {
                Some((level, ctx)) if ctx.item == BoundContext::Proc => {
                    Ok(NormalizedVar::BoundVar(level))
                }
                Some((_, name_ctx)) => Err(InterpreterError::UnexpectedProcContext {
                    var_name: id.name.to_string(),
                    name_var_source_position: name_ctx.source_position,
                    process_source_position: id.pos,
                }),
                None => free_map
                    .put_id_in_proc_context(id)
                    .map(NormalizedVar::FreeVar)
                    .map_err(
                        |old_context| InterpreterError::UnexpectedReuseOfProcContextFree {
                            var_name: id.name.to_string(),
                            first_use: old_context.source_position,
                            second_use: id.pos,
                        },
                    ),
            };
            input_par.push_expr(Expr::EVar(normalized_var?), depth);
            Ok(())
        }

        Proc::VarRef(var_ref) => {
            normalize_p_var_ref(*var_ref, input_par, bound_map_chain, env, pos)
        }
    }
}

// TODO Uncomment when everything will be ready and fix test
// // See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
// // inside this source file we tested unary and binary operations, because we don't have separate normalizers for them.
// #[cfg(test)]
// mod tests {
//     use crate::compiler::compiler::Compiler;
//     use crate::compiler::normalizer::normalize_match_proc;
//     use crate::compiler::normalizer::processes::proc_visit_input_end_env::proc_visit_inputs_and_env;
//     use crate::compiler::rholang_ast::{Collection, KeyValuePair, Proc};
//     use crate::compiler::source_position::SourcePosition;
//     use models::create_bit_vector;
//     use models::rhoapi::expr::ExprInstance;
//     use models::rhoapi::{
//         EDiv, EMinus, EMinusMinus, EMult, ENeg, ENot, EPercentPercent, EPlus, EPlusPlus, Expr, Par,
//         expr,
//     };
//     use models::rust::utils::{
//         new_boundvar_expr, new_boundvar_par, new_emap_par, new_freevar_par, new_gint_par,
//         new_gstring_par, new_key_value_pair,
//     };
//     use pretty_assertions::assert_eq;

//     #[test]
//     #[ignore]
//     fn p_nil_should_compile_as_no_modification() {
//         todo!();
//         // let (inputs, env) = proc_visit_inputs_and_env();

//         // let proc = Proc::Nil;
//         // let result = normalize_match_proc(&proc, inputs.clone(), &env);

//         // assert_eq!(result.clone().unwrap().par, inputs.par);
//         // assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     //unary operations:
//     #[test]
//     fn p_not_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();
//         let proc = Proc::Not {
//             proc: Box::new(Proc::new_proc_bool(false)),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, inputs.clone(), &env);
//         let expected_par = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(expr::ExprInstance::ENotBody(ENot {
//                     p: Some(Par {
//                         exprs: vec![Expr {
//                             expr_instance: Some(expr::ExprInstance::GBool(false)),
//                         }],
//                         ..Par::default()
//                     }),
//                 })),
//             },
//             0,
//         );

//         assert_eq!(result.clone().unwrap().par, expected_par);
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn p_neg_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();

//         let bound_inputs =
//             proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", ProcSort);
//         let proc = Proc::Neg {
//             proc: Box::new(Proc::new_proc_var("x")),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);
//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(expr::ExprInstance::ENegBody(ENeg {
//                     p: Some(Par {
//                         exprs: vec![new_boundvar_expr(0)],
//                         locally_free: create_bit_vector(&vec![0]),
//                         ..Par::default()
//                     }),
//                 })),
//             },
//             0,
//         );

//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     //binary operations:
//     #[test]
//     fn p_mult_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();

//         let bound_inputs =
//             proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", ProcSort);

//         let proc = Proc::Mult {
//             left: Box::new(Proc::new_proc_var("x")),
//             right: Box::new(Proc::new_proc_var("y")),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);

//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(expr::ExprInstance::EMultBody(EMult {
//                     p1: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
//                     p2: Some(new_freevar_par(0, Vec::new())),
//                 })),
//             },
//             0,
//         );
//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(
//             result.unwrap().free_map,
//             bound_inputs
//                 .free_map
//                 .put(("y".to_string(), ProcSort, SourcePosition::new(0, 0)))
//         );
//     }

//     #[test]
//     fn p_div_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();

//         let proc = Proc::Div {
//             left: Box::new(Proc::new_proc_int(7)),
//             right: Box::new(Proc::new_proc_int(2)),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, inputs.clone(), &env);

//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(expr::ExprInstance::EDivBody(EDiv {
//                     p1: Some(new_gint_par(7, Vec::new(), false)),
//                     p2: Some(new_gint_par(2, Vec::new(), false)),
//                 })),
//             },
//             0,
//         );

//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn p_percent_percent_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();

//         let map_data = Proc::Collection(Collection::Map {
//             pairs: vec![KeyValuePair {
//                 key: Proc::new_proc_string("name".to_string()),
//                 value: Proc::new_proc_string("Alice".to_string()),
//                 line_num: 0,
//                 col_num: 0,
//             }],
//             cont: None,
//             line_num: 0,
//             col_num: 0,
//         });

//         let proc = Proc::PercentPercent {
//             left: Box::new(Proc::new_proc_string("Hi ${name}".to_string())),
//             right: Box::new(map_data),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, inputs.clone(), &env);
//         assert_eq!(
//             result.clone().unwrap().par,
//             prepend_expr(
//                 inputs.par,
//                 Expr {
//                     expr_instance: Some(ExprInstance::EPercentPercentBody(EPercentPercent {
//                         p1: Some(new_gstring_par("Hi ${name}".to_string(), Vec::new(), false)),
//                         p2: Some(new_emap_par(
//                             vec![new_key_value_pair(
//                                 new_gstring_par("name".to_string(), Vec::new(), false),
//                                 new_gstring_par("Alice".to_string(), Vec::new(), false),
//                             )],
//                             Vec::new(),
//                             false,
//                             None,
//                             Vec::new(),
//                             false,
//                         ))
//                     }))
//                 },
//                 0
//             )
//         );

//         assert_eq!(result.unwrap().free_map, inputs.free_map)
//     }

//     #[test]
//     fn p_add_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();
//         let bound_inputs = proc_visit_inputs_with_updated_vec_bound_map_chain(
//             inputs.clone(),
//             vec![("x".into(), ProcSort), ("y".into(), ProcSort)],
//         );
//         let proc = Proc::Add {
//             left: Box::new(Proc::new_proc_var("x")),
//             right: Box::new(Proc::new_proc_var("y")),
//             line_num: 0,
//             col_num: 0,
//         };
//         let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);

//         // Формуємо очікуваний результат за допомогою хелперів
//         let p1 = new_boundvar_par(1, vec![1], false);
//         let p2 = new_boundvar_par(0, vec![0], false);

//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(ExprInstance::EPlusBody(EPlus {
//                     p1: Some(new_boundvar_par(1, create_bit_vector(&vec![1]), false)),
//                     p2: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
//                 })),
//             },
//             0,
//         );
//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(result.unwrap().free_map, bound_inputs.free_map);
//     }

//     #[test]
//     fn p_minus_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();
//         let bound_inputs = proc_visit_inputs_with_updated_vec_bound_map_chain(
//             inputs.clone(),
//             vec![
//                 ("x".into(), ProcSort),
//                 ("y".into(), ProcSort),
//                 ("z".into(), ProcSort),
//             ],
//         );

//         let proc = Proc::Minus {
//             left: Box::new(Proc::new_proc_var("x")),
//             right: Box::new(Proc::Mult {
//                 left: Box::new(Proc::new_proc_var("y")),
//                 right: Box::new(Proc::new_proc_var("z")),
//                 line_num: 0,
//                 col_num: 0,
//             }),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);

//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(ExprInstance::EMinusBody(EMinus {
//                     p1: Some(new_boundvar_par(2, create_bit_vector(&vec![2]), false)),
//                     p2: Some(Par {
//                         exprs: vec![Expr {
//                             expr_instance: Some(ExprInstance::EMultBody(EMult {
//                                 p1: Some(new_boundvar_par(1, create_bit_vector(&vec![1]), false)),
//                                 p2: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
//                             })),
//                         }],
//                         locally_free: create_bit_vector(&vec![0, 1]),
//                         connective_used: false,
//                         ..Par::default()
//                     }),
//                 })),
//             },
//             0,
//         );

//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn p_plus_plus_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();
//         let proc = Proc::Concat {
//             left: Box::new(Proc::new_proc_string("abc".to_string())),
//             right: Box::new(Proc::new_proc_string("def".to_string())),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, inputs.clone(), &env);
//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(ExprInstance::EPlusPlusBody(EPlusPlus {
//                     p1: Some(new_gstring_par("abc".to_string(), Vec::new(), false)),
//                     p2: Some(new_gstring_par("def".to_string(), Vec::new(), false)),
//                 })),
//             },
//             0,
//         );

//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn p_minus_minus_should_delegate() {
//         let (inputs, env) = proc_visit_inputs_and_env();

//         let proc = Proc::MinusMinus {
//             left: Box::new(Proc::new_proc_string("abc".to_string())),
//             right: Box::new(Proc::new_proc_string("def".to_string())),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&proc, inputs.clone(), &env);
//         let expected_result = prepend_expr(
//             inputs.par.clone(),
//             Expr {
//                 expr_instance: Some(ExprInstance::EMinusMinusBody(EMinusMinus {
//                     p1: Some(new_gstring_par("abc".to_string(), vec![], false)),
//                     p2: Some(new_gstring_par("def".to_string(), vec![], false)),
//                 })),
//             },
//             0,
//         );

//         assert_eq!(result.clone().unwrap().par, expected_result);
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn patterns_should_compile_not_in_top_level() {
//         fn check(typ: &str, position: &str, pattern: &str) {
//             /*
//              We use double curly braces to avoid conflicts with string formatting.
//              In Rust, `format!` uses `{}` for inserting values, so to output a literal curly brace,
//              we need to use `{{` and `}}`
//             */
//             let rho = format!(
//                 r#"
//         new x in {{
//             for(@y <- x) {{
//                 match y {{
//                     {} => Nil
//                 }}
//             }}
//         }}
//         "#,
//                 pattern
//             );

//             match Compiler::source_to_adt(&rho) {
//                 Ok(_) => assert!(true),
//                 Err(e) => panic!(
//                     "{} in the {} '{}' should not throw errors: {:?}",
//                     typ, position, pattern, e
//                 ),
//             }
//         }

//         let cases = vec![
//             ("wildcard", "send channel", "{_!(1)}"),
//             ("wildcard", "send data", "{@=*x!(_)}"),
//             ("wildcard", "send data", "{@Nil!(_)}"),
//             ("logical AND", "send data", "{@Nil!(1 /\\ 2)}"),
//             ("logical OR", "send data", "{@Nil!(1 \\/ 2)}"),
//             ("logical NOT", "send data", "{@Nil!(~1)}"),
//             ("logical AND", "send channel", "{@{Nil /\\ Nil}!(Nil)}"),
//             ("logical OR", "send channel", "{@{Nil \\/ Nil}!(Nil)}"),
//             ("logical NOT", "send channel", "{@{~Nil}!(Nil)}"),
//             (
//                 "wildcard",
//                 "receive pattern of the consume",
//                 "{for (_ <- x) { 1 }} ",
//             ),
//             (
//                 "wildcard",
//                 "body of the continuation",
//                 "{for (@1 <- x) { _ }} ",
//             ),
//             (
//                 "logical OR",
//                 "body of the continuation",
//                 "{for (@1 <- x) { 10 \\/ 20 }} ",
//             ),
//             (
//                 "logical AND",
//                 "body of the continuation",
//                 "{for(@1 <- x) { 10 /\\ 20 }} ",
//             ),
//             (
//                 "logical NOT",
//                 "body of the continuation",
//                 "{for(@1 <- x) { ~10 }} ",
//             ),
//             (
//                 "logical OR",
//                 "channel of the consume",
//                 "{for (@1 <- @{Nil /\\ Nil}) { Nil }} ",
//             ),
//             (
//                 "logical AND",
//                 "channel of the consume",
//                 "{for(@1 <- @{Nil \\/ Nil}) { Nil }} ",
//             ),
//             (
//                 "logical NOT",
//                 "channel of the consume",
//                 "{for(@1 <- @{~Nil}) { Nil }} ",
//             ),
//             (
//                 "wildcard",
//                 "channel of the consume",
//                 "{for(@1 <- _) { Nil }} ",
//             ),
//         ];

//         for (typ, position, pattern) in cases {
//             check(typ, position, pattern);
//         }
//     }

//     #[test]
//     fn p_var_should_compile_as_bound_var_if_its_in_env() {
//         let bound_inputs = {
//             let mut inputs = inputs();
//             inputs.bound_map_chain = inputs.bound_map_chain.put((
//                 "x".to_string(),
//                 VarSort::ProcSort,
//                 SourcePosition::new(0, 0),
//             ));
//             inputs
//         };

//         let result = normalize_p_var(&p_var(), bound_inputs);
//         assert!(result.is_ok());
//         assert_eq!(
//             result.clone().unwrap().par,
//             prepend_expr(inputs().par, new_boundvar_expr(0), 0)
//         );

//         assert_eq!(result.clone().unwrap().free_map, inputs().free_map);
//         assert_eq!(
//             result.unwrap().par.locally_free,
//             create_bit_vector(&vec![0])
//         );
//     }

//     #[test]
//     fn p_var_should_compile_as_free_var_if_its_not_in_env() {
//         let result = normalize_p_var(&p_var(), inputs());
//         assert!(result.is_ok());
//         assert_eq!(
//             result.clone().unwrap().par,
//             prepend_expr(inputs().par, new_freevar_expr(0), 0)
//         );

//         assert_eq!(
//             result.clone().unwrap().free_map,
//             inputs().free_map.put((
//                 "x".to_string(),
//                 VarSort::ProcSort,
//                 SourcePosition::new(0, 0)
//             ))
//         );
//     }

//     #[test]
//     fn p_var_should_not_compile_if_its_in_env_of_the_wrong_sort() {
//         let bound_inputs = {
//             let mut inputs = inputs();
//             inputs.bound_map_chain = inputs.bound_map_chain.put((
//                 "x".to_string(),
//                 VarSort::NameSort,
//                 SourcePosition::new(0, 0),
//             ));
//             inputs
//         };

//         let result = normalize_p_var(&p_var(), bound_inputs);
//         assert!(result.is_err());
//         assert_eq!(
//             result,
//             Err(InterpreterError::UnexpectedProcContext {
//                 var_name: "x".to_string(),
//                 name_var_source_position: SourcePosition::new(0, 0),
//                 process_source_position: SourcePosition::new(0, 0),
//             })
//         )
//     }

//     #[test]
//     fn p_var_should_not_compile_if_its_used_free_somewhere_else() {
//         let bound_inputs = {
//             let mut inputs = inputs();
//             inputs.free_map = inputs.free_map.put((
//                 "x".to_string(),
//                 VarSort::ProcSort,
//                 SourcePosition::new(0, 0),
//             ));
//             inputs
//         };

//         let result = normalize_p_var(&p_var(), bound_inputs);
//         assert!(result.is_err());
//         assert_eq!(
//             result,
//             Err(InterpreterError::UnexpectedReuseOfProcContextFree {
//                 var_name: "x".to_string(),
//                 first_use: SourcePosition::new(0, 0),
//                 second_use: SourcePosition::new(0, 0)
//             })
//         )
//     }

//     #[test]
//     fn p_eval_should_handle_a_bound_name_variable() {
//         let p_eval = Proc::Eval(Eval {
//             name: Name::new_name_var("x"),
//             line_num: 0,
//             col_num: 0,
//         });

//         let (mut inputs, env) = proc_visit_inputs_and_env();
//         inputs.bound_map_chain = inputs.bound_map_chain.put((
//             "x".to_string(),
//             VarSort::NameSort,
//             SourcePosition::new(0, 0),
//         ));

//         let result = normalize_match_proc(&p_eval, inputs.clone(), &env);
//         assert!(result.is_ok());
//         assert_eq!(
//             result.clone().unwrap().par,
//             prepend_expr(inputs.par, new_boundvar_expr(0), 0)
//         );
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn p_eval_should_collapse_a_quote() {
//         let p_eval = Proc::Eval(Eval {
//             name: Name::Quote(Box::new(Quote {
//                 quotable: Box::new(Proc::Par {
//                     left: Box::new(Proc::new_proc_var("x")),
//                     right: Box::new(Proc::new_proc_var("x")),
//                     line_num: 0,
//                     col_num: 0,
//                 }),
//                 line_num: 0,
//                 col_num: 0,
//             })),
//             line_num: 0,
//             col_num: 0,
//         });

//         let (mut inputs, env) = proc_visit_inputs_and_env();
//         inputs.bound_map_chain = inputs.bound_map_chain.put((
//             "x".to_string(),
//             VarSort::ProcSort,
//             SourcePosition::new(0, 0),
//         ));

//         let result = normalize_match_proc(&p_eval, inputs.clone(), &env);
//         assert!(result.is_ok());
//         assert_eq!(
//             result.clone().unwrap().par,
//             prepend_expr(
//                 prepend_expr(inputs.par, new_boundvar_expr(0), 0),
//                 new_boundvar_expr(0),
//                 0
//             )
//         );
//         assert_eq!(result.unwrap().free_map, inputs.free_map);
//     }

//     #[test]
//     fn p_par_should_compile_both_branches_into_a_par_object() {
//         let par_ground = Proc::Par {
//             left: Box::new(Proc::new_proc_int(7)),
//             right: Box::new(Proc::new_proc_int(8)),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result = normalize_match_proc(&par_ground, ProcVisitInputs::new(), &HashMap::new());
//         assert!(result.is_ok());
//         assert_eq!(
//             result.clone().unwrap().par,
//             Par::default().with_exprs(vec![new_gint_expr(8), new_gint_expr(7)])
//         );
//         assert_eq!(result.unwrap().free_map, ProcVisitInputs::new().free_map);
//     }

//     #[test]
//     fn p_par_should_compile_both_branches_with_the_same_environment() {
//         let par_double_bound = Proc::Par {
//             left: Box::new(Proc::new_proc_var("x")),
//             right: Box::new(Proc::new_proc_var("x")),
//             line_num: 0,
//             col_num: 0,
//         };

//         let (mut inputs, env) = proc_visit_inputs_and_env();
//         inputs.bound_map_chain = inputs.bound_map_chain.put((
//             "x".to_string(),
//             VarSort::ProcSort,
//             SourcePosition::new(0, 0),
//         ));

//         let result = normalize_match_proc(&par_double_bound, inputs, &env);
//         assert!(result.is_ok());
//         assert_eq!(result.clone().unwrap().par, {
//             let mut par =
//                 Par::default().with_exprs(vec![new_boundvar_expr(0), new_boundvar_expr(0)]);
//             par.locally_free = create_bit_vector(&vec![0]);
//             par
//         });
//         assert_eq!(result.unwrap().free_map, ProcVisitInputs::new().free_map);
//     }

//     #[test]
//     fn p_par_should_not_compile_if_both_branches_use_the_same_free_variable() {
//         let par_double_free = Proc::Par {
//             left: Box::new(Proc::new_proc_var("x")),
//             right: Box::new(Proc::new_proc_var("x")),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result =
//             normalize_match_proc(&par_double_free, ProcVisitInputs::new(), &HashMap::new());
//         assert!(result.is_err());
//         assert_eq!(
//             result,
//             Err(InterpreterError::UnexpectedReuseOfProcContextFree {
//                 var_name: "x".to_string(),
//                 first_use: SourcePosition::new(0, 0),
//                 second_use: SourcePosition::new(0, 0)
//             })
//         );
//     }

//     #[test]
//     fn p_par_should_accumulate_free_counts_from_both_branches() {
//         let par_double_free = Proc::Par {
//             left: Box::new(Proc::new_proc_var("x")),
//             right: Box::new(Proc::new_proc_var("y")),
//             line_num: 0,
//             col_num: 0,
//         };

//         let result =
//             normalize_match_proc(&par_double_free, ProcVisitInputs::new(), &HashMap::new());
//         assert!(result.is_ok());
//         assert_eq!(result.clone().unwrap().par, {
//             let mut par = Par::default().with_exprs(vec![new_freevar_expr(1), new_freevar_expr(0)]);
//             par.connective_used = true;
//             par
//         });
//         assert_eq!(
//             result.unwrap().free_map,
//             ProcVisitInputs::new().free_map.put_all(vec![
//                 ("x".to_owned(), VarSort::ProcSort, SourcePosition::new(0, 0)),
//                 ("y".to_owned(), VarSort::ProcSort, SourcePosition::new(0, 0))
//             ])
//         )
//     }

//     /*
//      * In this test case, 'huge_p_par' should iterate up to '50000'
//      * Without passing 'RUST_MIN_STACK' env variable, this test case will fail with StackOverflowError
//      * To test this correctly, change '50' to '50000' and run test with this command: 'RUST_MIN_STACK=2147483648 cargo test'
//      *
//      * 'RUST_MIN_STACK=2147483648' sets stack size to 2GB for rust program
//      * 'RUST_MIN_STACK=1073741824' sets stack size to 1GB
//      * 'RUST_MIN_STACK=536870912' sets stack size to 512MB
//      */
//     #[test]
//     fn p_par_should_normalize_without_stack_overflow_error_even_for_huge_program() {
//         let huge_p_par = (1..=50)
//             .map(|x| Proc::new_proc_int(x as i64))
//             .reduce(|l, r| Proc::Par {
//                 left: Box::new(l),
//                 right: Box::new(r),
//                 line_num: 0,
//                 col_num: 0,
//             })
//             .expect("Failed to create huge Proc::Par");

//         let result = normalize_match_proc(&huge_p_par, ProcVisitInputs::new(), &HashMap::new());
//         assert!(result.is_ok());
//     }
// }
