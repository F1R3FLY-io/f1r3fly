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

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
// inside this source file we tested unary and binary operations, because we don't have separate normalizers for them.
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::compiler::Compiler;
    use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
    use crate::rust::interpreter::compiler::normalize::VarSort::ProcSort;
    use crate::rust::interpreter::compiler::rholang_ast::{Collection, KeyValuePair, Proc};
    use crate::rust::interpreter::compiler::source_position::SourcePosition;
    use crate::rust::interpreter::test_utils::utils::{
        proc_visit_inputs_and_env, proc_visit_inputs_with_updated_bound_map_chain,
        proc_visit_inputs_with_updated_vec_bound_map_chain,
    };
    use crate::rust::interpreter::util::prepend_expr;
    use models::create_bit_vector;
    use models::rhoapi::expr::ExprInstance;
    use models::rhoapi::var::VarInstance;
    use models::rhoapi::{
        expr, EDiv, EMinus, EMinusMinus, EMult, ENeg, ENot, EPercentPercent, EPlus, EPlusPlus,
        EVar, Expr, Par, Var,
    };
    use models::rust::utils::{
        new_boundvar_expr, new_boundvar_par, new_emap_par, new_eplus_par, new_freevar_par,
        new_gint_par, new_gstring_par, new_key_value_pair,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn p_nil_should_compile_as_no_modification() {
        let (inputs, env) = proc_visit_inputs_and_env();

        let proc = Proc::Nil {
            line_num: 0,
            col_num: 0,
        };
        let result = normalize_match_proc(&proc, inputs.clone(), &env);

        assert_eq!(result.clone().unwrap().par, inputs.par);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    //unary operations:
    #[test]
    fn p_not_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let proc = Proc::Not {
            proc: Box::new(Proc::new_proc_bool(false)),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, inputs.clone(), &env);
        let expected_par = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(expr::ExprInstance::ENotBody(ENot {
                    p: Some(Par {
                        exprs: vec![Expr {
                            expr_instance: Some(expr::ExprInstance::GBool(false)),
                        }],
                        ..Par::default()
                    }),
                })),
            },
            0,
        );

        assert_eq!(result.clone().unwrap().par, expected_par);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_neg_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();

        let bound_inputs =
            proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", ProcSort);
        let proc = Proc::Neg {
            proc: Box::new(Proc::new_proc_var("x")),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);
        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(expr::ExprInstance::ENegBody(ENeg {
                    p: Some(Par {
                        exprs: vec![new_boundvar_expr(0)],
                        locally_free: create_bit_vector(&vec![0]),
                        ..Par::default()
                    }),
                })),
            },
            0,
        );

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    //binary operations:
    #[test]
    fn p_mult_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();

        let bound_inputs =
            proc_visit_inputs_with_updated_bound_map_chain(inputs.clone(), "x", ProcSort);

        let proc = Proc::Mult {
            left: Box::new(Proc::new_proc_var("x")),
            right: Box::new(Proc::new_proc_var("y")),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);

        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(expr::ExprInstance::EMultBody(EMult {
                    p1: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                    p2: Some(new_freevar_par(0, Vec::new())),
                })),
            },
            0,
        );
        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(
            result.unwrap().free_map,
            bound_inputs
                .free_map
                .put(("y".to_string(), ProcSort, SourcePosition::new(0, 0)))
        );
    }

    #[test]
    fn p_div_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();

        let proc = Proc::Div {
            left: Box::new(Proc::new_proc_int(7)),
            right: Box::new(Proc::new_proc_int(2)),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, inputs.clone(), &env);

        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(expr::ExprInstance::EDivBody(EDiv {
                    p1: Some(new_gint_par(7, Vec::new(), false)),
                    p2: Some(new_gint_par(2, Vec::new(), false)),
                })),
            },
            0,
        );

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_percent_percent_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();

        let map_data = Proc::Collection(Collection::Map {
            pairs: vec![KeyValuePair {
                key: Proc::new_proc_string("name".to_string()),
                value: Proc::new_proc_string("Alice".to_string()),
                line_num: 0,
                col_num: 0,
            }],
            cont: None,
            line_num: 0,
            col_num: 0,
        });

        let proc = Proc::PercentPercent {
            left: Box::new(Proc::new_proc_string("Hi ${name}".to_string())),
            right: Box::new(map_data),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, inputs.clone(), &env);
        assert_eq!(
            result.clone().unwrap().par,
            prepend_expr(
                inputs.par,
                Expr {
                    expr_instance: Some(ExprInstance::EPercentPercentBody(EPercentPercent {
                        p1: Some(new_gstring_par("Hi ${name}".to_string(), Vec::new(), false)),
                        p2: Some(new_emap_par(
                            vec![new_key_value_pair(
                                new_gstring_par("name".to_string(), Vec::new(), false),
                                new_gstring_par("Alice".to_string(), Vec::new(), false),
                            )],
                            Vec::new(),
                            false,
                            None,
                            Vec::new(),
                            false,
                        ))
                    }))
                },
                0
            )
        );

        assert_eq!(result.unwrap().free_map, inputs.free_map)
    }

    #[test]
    fn p_add_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let bound_inputs = proc_visit_inputs_with_updated_vec_bound_map_chain(
            inputs.clone(),
            vec![("x".into(), ProcSort), ("y".into(), ProcSort)],
        );
        let proc = Proc::Add {
            left: Box::new(Proc::new_proc_var("x")),
            right: Box::new(Proc::new_proc_var("y")),
            line_num: 0,
            col_num: 0,
        };
        let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);

        // Формуємо очікуваний результат за допомогою хелперів
        let p1 = new_boundvar_par(1, vec![1], false);
        let p2 = new_boundvar_par(0, vec![0], false);

        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(ExprInstance::EPlusBody(EPlus {
                    p1: Some(new_boundvar_par(1, create_bit_vector(&vec![1]), false)),
                    p2: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                })),
            },
            0,
        );
        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, bound_inputs.free_map);
    }

    #[test]
    fn p_minus_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let bound_inputs = proc_visit_inputs_with_updated_vec_bound_map_chain(
            inputs.clone(),
            vec![
                ("x".into(), ProcSort),
                ("y".into(), ProcSort),
                ("z".into(), ProcSort),
            ],
        );

        let proc = Proc::Minus {
            left: Box::new(Proc::new_proc_var("x")),
            right: Box::new(Proc::Mult {
                left: Box::new(Proc::new_proc_var("y")),
                right: Box::new(Proc::new_proc_var("z")),
                line_num: 0,
                col_num: 0,
            }),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, bound_inputs.clone(), &env);

        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(ExprInstance::EMinusBody(EMinus {
                    p1: Some(new_boundvar_par(2, create_bit_vector(&vec![2]), false)),
                    p2: Some(Par {
                        exprs: vec![Expr {
                            expr_instance: Some(ExprInstance::EMultBody(EMult {
                                p1: Some(new_boundvar_par(1, create_bit_vector(&vec![1]), false)),
                                p2: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                            })),
                        }],
                        locally_free: create_bit_vector(&vec![0, 1]),
                        connective_used: false,
                        ..Par::default()
                    }),
                })),
            },
            0,
        );

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_plus_plus_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let proc = Proc::Concat {
            left: Box::new(Proc::new_proc_string("abc".to_string())),
            right: Box::new(Proc::new_proc_string("def".to_string())),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, inputs.clone(), &env);
        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(ExprInstance::EPlusPlusBody(EPlusPlus {
                    p1: Some(new_gstring_par("abc".to_string(), Vec::new(), false)),
                    p2: Some(new_gstring_par("def".to_string(), Vec::new(), false)),
                })),
            },
            0,
        );

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn p_minus_minus_should_delegate() {
        let (inputs, env) = proc_visit_inputs_and_env();

        let proc = Proc::MinusMinus {
            left: Box::new(Proc::new_proc_string("abc".to_string())),
            right: Box::new(Proc::new_proc_string("def".to_string())),
            line_num: 0,
            col_num: 0,
        };

        let result = normalize_match_proc(&proc, inputs.clone(), &env);
        let expected_result = prepend_expr(
            inputs.par.clone(),
            Expr {
                expr_instance: Some(ExprInstance::EMinusMinusBody(EMinusMinus {
                    p1: Some(new_gstring_par("abc".to_string(), vec![], false)),
                    p2: Some(new_gstring_par("def".to_string(), vec![], false)),
                })),
            },
            0,
        );

        assert_eq!(result.clone().unwrap().par, expected_result);
        assert_eq!(result.unwrap().free_map, inputs.free_map);
    }

    #[test]
    fn patterns_should_compile_not_in_top_level() {
        fn check(typ: &str, position: &str, pattern: &str) {
            /*
             We use double curly braces to avoid conflicts with string formatting.
             In Rust, `format!` uses `{}` for inserting values, so to output a literal curly brace,
             we need to use `{{` and `}}`
            */
            let rho = format!(
                r#"
        new x in {{
            for(@y <- x) {{
                match y {{
                    {} => Nil
                }}
            }}
        }}
        "#,
                pattern
            );

            match Compiler::source_to_adt(&rho) {
                Ok(_) => assert!(true),
                Err(e) => panic!(
                    "{} in the {} '{}' should not throw errors: {:?}",
                    typ, position, pattern, e
                ),
            }
        }

        let cases = vec![
            ("wildcard", "send channel", "{_!(1)}"),
            ("wildcard", "send data", "{@=*x!(_)}"),
            ("wildcard", "send data", "{@Nil!(_)}"),
            ("logical AND", "send data", "{@Nil!(1 /\\ 2)}"),
            ("logical OR", "send data", "{@Nil!(1 \\/ 2)}"),
            ("logical NOT", "send data", "{@Nil!(~1)}"),
            ("logical AND", "send channel", "{@{Nil /\\ Nil}!(Nil)}"),
            ("logical OR", "send channel", "{@{Nil \\/ Nil}!(Nil)}"),
            ("logical NOT", "send channel", "{@{~Nil}!(Nil)}"),
            (
                "wildcard",
                "receive pattern of the consume",
                "{for (_ <- x) { 1 }} ",
            ),
            (
                "wildcard",
                "body of the continuation",
                "{for (@1 <- x) { _ }} ",
            ),
            (
                "logical OR",
                "body of the continuation",
                "{for (@1 <- x) { 10 \\/ 20 }} ",
            ),
            (
                "logical AND",
                "body of the continuation",
                "{for(@1 <- x) { 10 /\\ 20 }} ",
            ),
            (
                "logical NOT",
                "body of the continuation",
                "{for(@1 <- x) { ~10 }} ",
            ),
            (
                "logical OR",
                "channel of the consume",
                "{for (@1 <- @{Nil /\\ Nil}) { Nil }} ",
            ),
            (
                "logical AND",
                "channel of the consume",
                "{for(@1 <- @{Nil \\/ Nil}) { Nil }} ",
            ),
            (
                "logical NOT",
                "channel of the consume",
                "{for(@1 <- @{~Nil}) { Nil }} ",
            ),
            (
                "wildcard",
                "channel of the consume",
                "{for(@1 <- _) { Nil }} ",
            ),
        ];

        for (typ, position, pattern) in cases {
            check(typ, position, pattern);
        }
    }
}
