use stacker::maybe_grow;

use super::exports::*;
use super::normalizer::collection_normalize_matcher::{
    normalize_c_list, normalize_c_set, normalize_c_tuple,
};
use super::rholang_ast::{Collection, Proc, SimpleType};
use crate::rust::interpreter::compiler::bound_map::BoundContext;
use crate::rust::interpreter::compiler::normalizer::name_normalize_matcher::normalize_name;
use crate::rust::interpreter::compiler::normalizer::processes::exports::AnnProc;
use crate::rust::interpreter::compiler::normalizer::processes::p_input_normalizer::normalize_p_input;
use crate::rust::interpreter::compiler::normalizer::processes::p_let_normalizer::{
    normalize_p_concurrent_let, normalize_p_let,
};
use crate::rust::interpreter::compiler::normalizer::processes::p_var_ref_normalizer::normalize_p_var_ref;
use crate::rust::interpreter::compiler::rholang_ast::Var;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::normal_forms::{Connective, Expr, Par, Var as NormalizedVar};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BoundVar {
    ProcBound,
    NameBound(Option<String>),
}

const MINIMUM_STACK_SIZE: usize = 512;
const STACK_ALLOC_SIZE: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Mult,
    Interpolation,
    Sub,
    Concat,
    Diff,
    Div,
    Mod,
    Add,
    Or,
    And,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
}

/**
 * Rholang normalizer entry point
 */
#[must_use]
pub(crate) fn normalize_match_proc(
    proc: &Proc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &HashMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    maybe_grow(MINIMUM_STACK_SIZE, STACK_ALLOC_SIZE, || {
        normalize_match_proc_internal(proc, input_par, free_map, bound_map_chain, env, pos)
    })
}

fn binary_op_to_expr_constructor(op: BinaryOp) -> fn(Par, Par) -> Expr {
    match op {
        BinaryOp::Mult => Expr::EMult,
        BinaryOp::Interpolation => Expr::EPercentPercent,
        BinaryOp::Sub => Expr::EMinus,
        BinaryOp::Concat => Expr::EPlusPlus,
        BinaryOp::Diff => Expr::EMinusMinus,
        BinaryOp::Div => Expr::EDiv,
        BinaryOp::Mod => Expr::EMod,
        BinaryOp::Add => Expr::EPlus,
        BinaryOp::Or => Expr::EOr,
        BinaryOp::And => Expr::EAnd,
        BinaryOp::Eq => Expr::EEq,
        BinaryOp::Neq => Expr::ENeq,
        BinaryOp::Lt => Expr::ELt,
        BinaryOp::Lte => Expr::ELte,
        BinaryOp::Gt => Expr::EGt,
        BinaryOp::Gte => Expr::EGte,
    }
}

/**
 * Rholang normalizer entry point
 */
#[must_use]
fn normalize_match_proc_internal(
    proc: &Proc,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &HashMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    fn binary_exp<C>(
        left: AnnProc,
        right: AnnProc,
        input_par: &mut Par,
        constr: C,
        free_map: &mut FreeMap,
        bound_map_chain: &mut BoundMapChain,
        env: &HashMap<String, Par>,
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

    fn handle_binary_op(
        op: BinaryOp,
        left: AnnProc,
        right: AnnProc,
        input_par: &mut Par,
        free_map: &mut FreeMap,
        bound_map_chain: &mut BoundMapChain,
        env: &HashMap<String, Par>,
    ) -> Result<(), InterpreterError> {
        binary_exp(
            left,
            right,
            input_par,
            binary_op_to_expr_constructor(op),
            free_map,
            bound_map_chain,
            env,
        )
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
            line_num,
            col_num,
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
        Proc::Mult { left, right } => handle_binary_op(
            BinaryOp::Mult,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Interpolation { left, right } => handle_binary_op(
            BinaryOp::Interpolation,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Sub { left, right } => handle_binary_op(
            BinaryOp::Sub,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Concat { left, right } => handle_binary_op(
            BinaryOp::Concat,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Diff { left, right } => handle_binary_op(
            BinaryOp::Diff,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Div { left, right } => handle_binary_op(
            BinaryOp::Div,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Mod { left, right } => handle_binary_op(
            BinaryOp::Mod,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Add { left, right } => handle_binary_op(
            BinaryOp::Add,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Or { left, right } => handle_binary_op(
            BinaryOp::Or,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::And { left, right } => handle_binary_op(
            BinaryOp::And,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Eq { left, right } => handle_binary_op(
            BinaryOp::Eq,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Neq { left, right } => handle_binary_op(
            BinaryOp::Neq,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Lt { left, right } => handle_binary_op(
            BinaryOp::Lt,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Lte { left, right } => handle_binary_op(
            BinaryOp::Lte,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Gt { left, right } => handle_binary_op(
            BinaryOp::Gt,
            *left,
            *right,
            input_par,
            free_map,
            bound_map_chain,
            env,
        ),
        Proc::Gte { left, right } => handle_binary_op(
            BinaryOp::Gte,
            *left,
            *right,
            input_par,
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
                SimpleType::Bool => input_par.push_connective(Connective::ConnBool, depth),
                SimpleType::Int => input_par.push_connective(Connective::ConnInt, depth),
                SimpleType::String => input_par.push_connective(Connective::ConnString, depth),
                SimpleType::Uri => input_par.push_connective(Connective::ConnUri, depth),
                SimpleType::ByteArray => {
                    input_par.push_connective(Connective::ConnByteArray, depth)
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

// See rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
// inside this source file we tested unary and binary operations, because we don't have separate normalizers for them.
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::compiler::Compiler;
    use crate::rust::interpreter::compiler::rholang_ast::{Collection, Id, KeyValuePair, Proc, Name};
    use crate::rust::interpreter::compiler::source_position::SourcePosition;
    use crate::rust::interpreter::errors::InterpreterError;
    use models::create_bit_vector;
    use models::rhoapi::expr::ExprInstance;
    use models::rhoapi::{
        expr, EDiv, EMinus, EMinusMinus, EMult, ENeg, ENot, EPercentPercent, EPlus, EPlusPlus,
        Expr, Par, EVar,
    };
    use models::rust::utils::{
        new_boundvar_expr, new_boundvar_par, new_emap_par, new_freevar_par, new_gint_par,
        new_gstring_par, new_key_value_pair,
    };
    use pretty_assertions::assert_eq;
    use crate::rust::interpreter::test_utils::utils::{
        defaults, test, test_normalize_match_proc, with_bindings,
    };
    use crate::assert_matches;

    #[test]
    fn p_nil_should_compile_as_no_modification() {
        let proc = Proc::Nil;

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to compile with no modification", |actual_par, free_map| {
                assert_eq!(actual_par.exprs.len(), Par::default().exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    //unary operations:
    #[test]
    fn p_not_should_delegate() {
        let proc = Proc::Not(&Proc::BoolLiteral(false));

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to delegate not correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::ENotBody(ENot {
                            p: Some(Par {
                                exprs: vec![Expr {
                                    expr_instance: Some(expr::ExprInstance::GBool(false)),
                                }],
                                ..Par::default()
                            }),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_neg_should_delegate() {
        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let proc_x = x.as_proc();
        let proc = Proc::Neg(&proc_x);

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test("expected to delegate neg correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::ENegBody(ENeg {
                            p: Some(Par {
                                exprs: vec![new_boundvar_expr(0)],
                                locally_free: create_bit_vector(&vec![0]),
                                ..Par::default()
                            }),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    //binary operations:
    #[test]
    fn p_mult_should_delegate() {
        let pos = SourcePosition::default();
        let x = Id {
            name: "x",
            pos,
        };
        let y = Id {
            name: "y",
            pos,
        };
        let proc_x = x.as_proc();
        let proc_y = y.as_proc();
        let proc = Proc::Mult {
            left: proc_x.annotate(pos),
            right: proc_y.annotate(pos),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test("expected to delegate mult correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EMultBody(EMult {
                            p1: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                            p2: Some(new_freevar_par(0, Vec::new())),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par, &expected_par);
                assert_eq!(
                    free_map.len(),
                    1,
                    "Expected free_map to contain the variable y"
                );
            }),
        );
    }

    #[test]
    fn p_div_should_delegate() {
        let proc = Proc::Div {
            left: Proc::LongLiteral(7).annotate_dummy(),
            right: Proc::LongLiteral(2).annotate_dummy(),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to delegate div correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EDivBody(EDiv {
                            p1: Some(new_gint_par(7, Vec::new(), false)),
                            p2: Some(new_gint_par(2, Vec::new(), false)),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_percent_percent_should_delegate() {
        let map_data = Proc::Collection(Collection::Map {
            pairs: vec![KeyValuePair {
                key: Proc::StringLiteral("name").annotate_dummy(),
                value: Proc::StringLiteral("Alice").annotate_dummy(),
            }],
            cont: None,
        });

        let proc = Proc::Interpolation {
            left: Proc::StringLiteral("Hi ${name}").annotate_dummy(),
            right: map_data.annotate_dummy(),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to delegate percent percent correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
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
                            )),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_add_should_delegate() {
        let pos = SourcePosition::default();
        let x = Id {
            name: "x",
            pos,
        };
        let y = Id {
            name: "y",
            pos,
        };
        let proc_x = x.as_proc();
        let proc_y = y.as_proc();
        let proc = Proc::Add {
            left: proc_x.annotate(pos),
            right: proc_y.annotate(pos),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
                bound_map.put_id_as_proc(&y);
            }),
            test("expected to delegate add correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EPlusBody(EPlus {
                            p1: Some(new_boundvar_par(1, create_bit_vector(&vec![1]), false)),
                            p2: Some(new_boundvar_par(0, create_bit_vector(&vec![0]), false)),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_minus_should_delegate() {
        let pos = SourcePosition::default();
        let x = Id {
            name: "x",
            pos,
        };
        let y = Id {
            name: "y",
            pos,
        };
        let z = Id {
            name: "z",
            pos,
        };
        let proc_x = x.as_proc();
        let proc_y = y.as_proc();
        let proc_z = z.as_proc();

        let mult_proc = Proc::Mult {
            left: proc_y.annotate(pos),
            right: proc_z.annotate(pos),
        };
        let proc = Proc::Sub {
            left: proc_x.annotate(pos),
            right: mult_proc.annotate(pos),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
                bound_map.put_id_as_proc(&y);
                bound_map.put_id_as_proc(&z);
            }),
            test("expected to delegate minus correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
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
                                ..Par::default()
                            }),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_plus_plus_should_delegate() {
        let proc = Proc::Concat {
            left: Proc::StringLiteral("abc").annotate_dummy(),
            right:Proc::StringLiteral("def").annotate_dummy(),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to delegate plus plus correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EPlusPlusBody(EPlusPlus {
                            p1: Some(new_gstring_par("abc".to_string(), Vec::new(), false)),
                            p2: Some(new_gstring_par("def".to_string(), Vec::new(), false)),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_minus_minus_should_delegate() {
        let proc = Proc::Diff {
            left: Proc::StringLiteral("abc").annotate_dummy(),
            right: Proc::StringLiteral("def").annotate_dummy(),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to delegate correctly", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EMinusMinusBody(EMinusMinus {
                            p1: Some(new_gstring_par("abc".to_string(), vec![], false)),
                            p2: Some(new_gstring_par("def".to_string(), vec![], false)),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
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

            match Compiler::new(&rho).compile_to_adt() {
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

    #[test]
    fn p_var_should_compile_as_bound_var_if_its_in_env() {
        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let proc_x = x.as_proc();

        test_normalize_match_proc(
            &proc_x,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test("expected to compile as bound var if it's in env", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EVarBody(EVar {
                            v: Some(models::rhoapi::Var {
                                var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(0)),
                            }),
                        })),
                    }],
                    locally_free: create_bit_vector(&vec![0]),
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_var_should_compile_as_free_var_if_its_not_in_env() {
        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let proc_x = x.as_proc();

        test_normalize_match_proc(
            &proc_x,
            defaults(),
            test("expected to compile as free var if it's not in env", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EVarBody(EVar {
                            v: Some(models::rhoapi::Var {
                                var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(0)),
                            }),
                        })),
                    }],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert_eq!(free_map.len(), 1);
            }),
        );
    }

    #[test]
    fn p_var_should_not_compile_if_its_in_env_of_the_wrong_sort() {
        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let proc_x = x.as_proc();

        test_normalize_match_proc(
            &proc_x,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&x);
            }),
            |result| {
                assert_matches!(
                    result,
                    Err(InterpreterError::UnexpectedProcContext {
                        var_name,
                        name_var_source_position: SourcePosition::default(),
                        process_source_position: SourcePosition::default(),
                    }) => {
                        assert_eq!(var_name, "x")
                    }
                );
            },
        );
    }

    #[test]
    fn p_var_should_not_compile_if_its_used_free_somewhere_else() {
        let pos = SourcePosition::default();
        let x1 = Id {
            name: "x",
            pos,
        };
        let proc_x1 = x1.as_proc();

        let x2 = Id {
            name: "x",
            pos,
        };
        let proc_x2 = x2.as_proc();

        let proc = Proc::Par {
            left: proc_x1.annotate(pos),
            right: proc_x2.annotate(pos),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            |result| {
                assert_matches!(
                    result,
                    Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                        var_name,
                        first_use: SourcePosition { row: 0, column: 0 },
                        second_use: SourcePosition { row: 0, column: 0 }
                    }) => {
                        assert_eq!(var_name, "x")
                    }
                );
            },
        );
    }

    #[test]
    fn p_eval_should_handle_a_bound_name_variable() {
        let pos = SourcePosition::default();
        let x = Id {
            name: "x",
            pos,
        };
        let proc = Proc::Eval {
            name: x.as_name().annotated(pos),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_name(&x);
            }),
            test("expected to handle a bound name variable", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EVarBody(EVar {
                            v: Some(models::rhoapi::Var {
                                var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(0)),
                            }),
                        })),
                    }],
                    locally_free: create_bit_vector(&vec![0]),
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_eval_should_collapse_a_quote() {
        let pos = SourcePosition::default();
        let x = Id {
            name: "x",
            pos,
        };
        let proc_x = x.as_proc();
        let proc = Proc::Eval {
            name: Name::Quote(&proc_x).annotated(pos),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test("expected to collapse a quote", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![Expr {
                        expr_instance: Some(ExprInstance::EVarBody(EVar {
                            v: Some(models::rhoapi::Var {
                                var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(0)),
                            }),
                        })),
                    }],
                    locally_free: create_bit_vector(&vec![0]),
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_par_should_compile_both_branches_into_a_par_object() {
        let proc = Proc::Par {
            left: Proc::LongLiteral(7).annotate_dummy(),
            right: Proc::LongLiteral(8).annotate_dummy(),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to compile both branches into a par object", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![
                        Expr {
                            expr_instance: Some(ExprInstance::GInt(8)),
                        },
                        Expr {
                            expr_instance: Some(ExprInstance::GInt(7)),
                        },
                    ],
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_par_should_compile_both_branches_with_the_same_environment() {
        let x = Id {
            name: "x",
            pos: SourcePosition::default(),
        };
        let proc_x1 = x.as_proc();
        let proc_x2 = x.as_proc();
        let proc = Proc::Par {
            left: proc_x1.annotate_dummy(),
            right: proc_x2.annotate_dummy(),
        };

        test_normalize_match_proc(
            &proc,
            with_bindings(|bound_map| {
                bound_map.put_id_as_proc(&x);
            }),
            test("expected to compile both branches with same env", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![
                        Expr {
                            expr_instance: Some(ExprInstance::EVarBody(EVar {
                                v: Some(models::rhoapi::Var {
                                    var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(0)),
                                }),
                            })),
                        },
                        Expr {
                            expr_instance: Some(ExprInstance::EVarBody(EVar {
                                v: Some(models::rhoapi::Var {
                                    var_instance: Some(models::rhoapi::var::VarInstance::BoundVar(0)),
                                }),
                            })),
                        },
                    ],
                    locally_free: create_bit_vector(&vec![0]),
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert!(free_map.is_empty());
            }),
        );
    }

    #[test]
    fn p_par_should_not_compile_if_both_branches_use_the_same_free_variable() {
        let pos = SourcePosition::default();
        let x1 = Id {
            name: "x",
            pos,
        };
        let proc_x1 = x1.as_proc();

        let x2 = Id {
            name: "x",
            pos,
        };
        let proc_x2 = x2.as_proc();

        let proc = Proc::Par {
            left: proc_x1.annotate(pos),
            right: proc_x2.annotate(pos),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            |result| {
                assert_matches!(
                    result,
                    Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                        var_name,
                        first_use: SourcePosition::default(),
                        second_use: SourcePosition::default()
                    }) => {
                        assert_eq!(var_name, "x")
                    }
                );
            },
        );
    }

    #[test]
    fn p_par_should_accumulate_free_counts_from_both_branches() {
        let pos = SourcePosition::default();
        let x = Id {
            name: "x",
            pos,
        };
        let proc_x = x.as_proc();

        let y = Id {
            name: "y",
            pos,
        };
        let proc_y = y.as_proc();

        let proc = Proc::Par {
            left: proc_x.annotate(pos),
            right: proc_y.annotate(pos),
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            test("expected to accumulate free counts from both branches", |actual_par, free_map| {
                let expected_par = Par {
                    exprs: vec![
                        Expr {
                            expr_instance: Some(ExprInstance::EVarBody(EVar {
                                v: Some(models::rhoapi::Var {
                                    var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(1)),
                                }),
                            })),
                        },
                        Expr {
                            expr_instance: Some(ExprInstance::EVarBody(EVar {
                                v: Some(models::rhoapi::Var {
                                    var_instance: Some(models::rhoapi::var::VarInstance::FreeVar(0)),
                                }),
                            })),
                        },
                    ],
                    connective_used: true,
                    ..Par::default()
                };

                assert_eq!(actual_par.exprs.len(), expected_par.exprs.len());
                assert_eq!(free_map.len(), 2);
            }),
        );
    }

    /*
     * In this test case, 'huge_p_par' should iterate up to '50000'
     * Without passing 'RUST_MIN_STACK' env variable, this test case will fail with StackOverflowError
     * To test this correctly, change '50' to '50000' and run test with this command: 'RUST_MIN_STACK=2147483648 cargo test'
     *
     * 'RUST_MIN_STACK=2147483648' sets stack size to 2GB for rust program
     * 'RUST_MIN_STACK=1073741824' sets stack size to 1GB
     * 'RUST_MIN_STACK=536870912' sets stack size to 512MB
     */
    #[test]
    fn p_par_should_normalize_without_stack_overflow_error_even_for_huge_program() {
        let base_proc = Proc::LongLiteral(1);

        let mut procs = Vec::with_capacity(50);
        for i in 1..=50 {
            procs.push(Proc::LongLiteral(i));
        }

        /*
         * This function build a balanced binary tree of Proc::Par nodes from an array of Proc objects.
         * It's transforms a flat array of processes [Proc1, Proc2, ..., ProcN]
         * into a balanced binary tree structure of Proc::Par.
         * This is necessary for a correct stack overflow test.
         */
        fn build_par(procs: &[Proc], start: usize, end: usize) -> Proc {
            if start == end {
                return procs[start].clone();
            }
            let mid = start + (end - start) / 2;
            let left = build_par(procs, start, mid);
            let right = build_par(procs, mid + 1, end);
            Proc::Par {
                left: left.annotate_dummy(),
                right: right.annotate_dummy(),
            }
        }

        let proc = if procs.len() > 0 {
            build_par(&procs, 0, procs.len() - 1)
        } else {
            base_proc
        };

        test_normalize_match_proc(
            &proc,
            defaults(),
            |result| {
                assert!(result.is_ok(), "Expected no stack overflow for large program");
                let (par, _) = result.unwrap();
                assert_eq!(par.exprs.len(), 50);
            },
        );
    }
}
