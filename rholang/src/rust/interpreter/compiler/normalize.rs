use super::exports::*;
use super::normalizer::processes::p_var_normalizer::normalize_p_var;
use super::rholang_ast::Proc;
use crate::rust::interpreter::compiler::normalizer::processes::p_input_normalizer::normalize_p_input;
use crate::rust::interpreter::compiler::normalizer::processes::p_let_normalizer::normalize_p_let;
use crate::rust::interpreter::compiler::normalizer::processes::p_var_ref_normalizer::normalize_p_var_ref;
use crate::rust::interpreter::compiler::utils::{BinaryExpr, UnaryExpr};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;
use models::rhoapi::{
    EAnd, EDiv, EEq, EGt, EGte, ELt, ELte, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr,
    EPercentPercent, EPlus, EPlusPlus, Expr, Par,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum VarSort {
    ProcSort,
    NameSort,
}

/**
 * Input data to the normalizer
 *
 * @param par collection of things that might be run in parallel
 * @param env
 * @param knownFree
 */
#[derive(Clone, Debug, PartialEq)]
pub struct ProcVisitInputs {
    pub par: Par,
    pub bound_map_chain: BoundMapChain<VarSort>,
    pub free_map: FreeMap<VarSort>,
}

impl ProcVisitInputs {
    pub fn new() -> Self {
        ProcVisitInputs {
            par: Par::default(),
            bound_map_chain: BoundMapChain::new(),
            free_map: FreeMap::new(),
        }
    }
}

// Returns the update Par and an updated map of free variables.
#[derive(Clone, Debug, PartialEq)]
pub struct ProcVisitOutputs {
    pub par: Par,
    pub free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct NameVisitInputs {
    pub(crate) bound_map_chain: BoundMapChain<VarSort>,
    pub(crate) free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct NameVisitOutputs {
    pub(crate) par: Par,
    pub(crate) free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct CollectVisitInputs {
    pub(crate) bound_map_chain: BoundMapChain<VarSort>,
    pub(crate) free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct CollectVisitOutputs {
    pub(crate) expr: Expr,
    pub(crate) free_map: FreeMap<VarSort>,
}

/**
 * Rholang normalizer entry point
 */
pub fn normalize_match_proc(
    proc: &Proc,
    input: ProcVisitInputs,
    env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {
    fn unary_exp(
        sub_proc: &Proc,
        input: ProcVisitInputs,
        constructor: Box<dyn UnaryExpr>,
        env: &HashMap<String, Par>,
    ) -> Result<ProcVisitOutputs, InterpreterError> {
        let sub_result = normalize_match_proc(sub_proc, input.clone(), env)?;
        let expr = constructor.from_par(sub_result.par.clone());

        Ok(ProcVisitOutputs {
            par: prepend_expr(input.par, expr, input.bound_map_chain.depth() as i32),
            free_map: sub_result.free_map,
        })
    }

    fn binary_exp(
        left_proc: &Proc,
        right_proc: &Proc,
        input: ProcVisitInputs,
        constructor: Box<dyn BinaryExpr>,
        env: &HashMap<String, Par>,
    ) -> Result<ProcVisitOutputs, InterpreterError> {
        let left_result = normalize_match_proc(left_proc, input.clone(), env)?;
        let right_result = normalize_match_proc(
            right_proc,
            ProcVisitInputs {
                par: Par::default(),
                free_map: left_result.free_map.clone(),
                ..input.clone()
            },
            env,
        )?;

        let expr: Expr = constructor.from_pars(left_result.par.clone(), right_result.par.clone());

        Ok(ProcVisitOutputs {
            par: prepend_expr(input.par, expr, input.bound_map_chain.depth() as i32),
            free_map: right_result.free_map,
        })
    }

    match proc {
        Proc::Par { left, right, .. } => normalize_p_par(left, right, input, env),

        Proc::SendSync {
            name,
            messages,
            cont,
            line_num,
            col_num,
        } => normalize_p_send_sync(name, messages, cont, *line_num, *col_num, input, env),

        Proc::New { decls, proc, .. } => normalize_p_new(decls, proc, input, env),

        Proc::IfElse {
            condition,
            if_true,
            alternative,
            ..
        } => {
            let mut empty_par_input = input.clone();
            empty_par_input.par = Par::default();

            match alternative {
                Some(alternative_proc) => {
                    normalize_p_if(condition, if_true, alternative_proc, empty_par_input, env).map(
                        |mut new_visits| {
                            let new_par = new_visits.par.append(input.par);
                            new_visits.par = new_par;
                            new_visits
                        },
                    )
                }
                None => normalize_p_if(
                    condition,
                    if_true,
                    &Proc::Nil {
                        line_num: 0,
                        col_num: 0,
                    },
                    empty_par_input,
                    env,
                )
                .map(|mut new_visits| {
                    let new_par = new_visits.par.append(input.par);
                    new_visits.par = new_par;
                    new_visits
                }),
            }
        }

        Proc::Let { decls, body, .. } => normalize_p_let(decls, body, input, env),

        Proc::Bundle {
            bundle_type,
            proc,
            line_num,
            col_num,
        } => normalize_p_bundle(bundle_type, proc, input, *line_num, *col_num, env),

        Proc::Match {
            expression, cases, ..
        } => normalize_p_match(expression, cases, input, env),

        // I don't think the previous scala developers implemented a normalize function for this
        Proc::Choice {
            branches,
            line_num,
            col_num,
        } => todo!(),

        Proc::Contract {
            name,
            formals,
            proc,
            ..
        } => normalize_p_contr(name, formals, proc, input, env),

        Proc::Input {
            formals,
            proc,
            line_num,
            col_num,
        } => normalize_p_input(formals, proc, *line_num, *col_num, input, env),

        Proc::Send {
            name,
            send_type,
            inputs,
            ..
        } => normalize_p_send(name, send_type, inputs, input, env),

        Proc::Matches { left, right, .. } => normalize_p_matches(&left, &right, input, env),

        // binary
        Proc::Mult { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMult::default()), env)
        }

        Proc::PercentPercent { left, right, .. } => binary_exp(
            left,
            right,
            input,
            Box::new(EPercentPercent::default()),
            env,
        ),

        Proc::Minus { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMinus::default()), env)
        }

        // PlusPlus
        Proc::Concat { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EPlusPlus::default()), env)
        }

        Proc::MinusMinus { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMinusMinus::default()), env)
        }

        Proc::Div { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EDiv::default()), env)
        }
        Proc::Mod { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMod::default()), env)
        }
        Proc::Add { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EPlus::default()), env)
        }
        Proc::Or { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EOr::default()), env)
        }
        Proc::And { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EAnd::default()), env)
        }
        Proc::Eq { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EEq::default()), env)
        }
        Proc::Neq { left, right, .. } => {
            binary_exp(left, right, input, Box::new(ENeq::default()), env)
        }
        Proc::Lt { left, right, .. } => {
            binary_exp(left, right, input, Box::new(ELt::default()), env)
        }
        Proc::Lte { left, right, .. } => {
            binary_exp(left, right, input, Box::new(ELte::default()), env)
        }
        Proc::Gt { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EGt::default()), env)
        }
        Proc::Gte { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EGte::default()), env)
        }

        // unary
        Proc::Not { proc: sub_proc, .. } => {
            unary_exp(sub_proc, input, Box::new(ENot::default()), env)
        }
        Proc::Neg { proc: sub_proc, .. } => {
            unary_exp(sub_proc, input, Box::new(ENeg::default()), env)
        }

        Proc::Method {
            receiver,
            name,
            args,
            ..
        } => normalize_p_method(receiver, name, args, input, env),

        Proc::Eval(eval) => normalize_p_eval(eval, input, env),

        Proc::Quote(quote) => normalize_match_proc(&quote.quotable, input, env),

        Proc::Disjunction(disjunction) => normalize_p_disjunction(disjunction, input, env),

        Proc::Conjunction(conjunction) => normalize_p_conjunction(conjunction, input, env),

        Proc::Negation(negation) => normalize_p_negation(negation, input, env),

        Proc::Block(block) => normalize_match_proc(&block.proc, input, env),

        Proc::Collection(collection) => normalize_p_collect(collection, input, env),

        Proc::SimpleType(simple_type) => normalize_simple_type(simple_type, input),

        Proc::BoolLiteral { .. } => normalize_p_ground(proc, input),
        Proc::LongLiteral { .. } => normalize_p_ground(proc, input),
        Proc::StringLiteral { .. } => normalize_p_ground(proc, input),
        Proc::UriLiteral(_) => normalize_p_ground(proc, input),

        Proc::Nil { .. } => Ok(ProcVisitOutputs {
            par: input.par.clone(),
            free_map: input.free_map.clone(),
        }),

        Proc::Var(_) => normalize_p_var(proc, input),
        Proc::Wildcard { .. } => normalize_p_var(proc, input),

        Proc::VarRef(var_ref) => normalize_p_var_ref(var_ref, input),
    }
}
