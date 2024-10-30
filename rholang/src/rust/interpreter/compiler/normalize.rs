use super::exports::*;
use super::normalizer::processes::p_var_normalizer::normalize_p_var;
use super::rholang_ast::Proc;
use crate::rust::interpreter::compiler::normalizer::processes::p_var_ref_normalizer::normalize_p_var_ref;
use crate::rust::interpreter::compiler::utils::{BinaryExpr, UnaryExpr};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;
use models::rhoapi::{
    EAnd, EDiv, EEq, EGt, EGte, ELt, ELte, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr,
    EPercentPercent, EPlus, EPlusPlus, Expr, Par,
};
use std::error::Error;
use tree_sitter::Node;

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
pub fn normalize_match(
    p_node: Node,
    input: ProcVisitInputs,
    source_code: &[u8],
) -> Result<ProcVisitOutputs, Box<dyn Error>> {
    pub fn unary_exp(
        sub_proc_node: Node,
        input: ProcVisitInputs,
        source_code: &[u8],
        constructor: Box<dyn UnaryExpr>,
    ) -> Result<ProcVisitOutputs, Box<dyn Error>> {
        let sub_result = normalize_match(sub_proc_node, input.clone(), source_code)?;
        let expr = constructor.from_par(sub_result.par.clone());

        Ok(ProcVisitOutputs {
            par: prepend_expr(input.par, expr, input.bound_map_chain.depth() as i32),
            free_map: sub_result.free_map,
        })
    }

    pub fn binary_exp(
        left_proc_node: Node,
        right_proc_node: Node,
        input: ProcVisitInputs,
        source_code: &[u8],
        constructor: Box<dyn BinaryExpr>,
    ) -> Result<ProcVisitOutputs, Box<dyn Error>> {
        let left_result = normalize_match(left_proc_node, input.clone(), source_code)?;
        let right_result = normalize_match(
            right_proc_node,
            ProcVisitInputs {
                par: Par::default(),
                free_map: left_result.free_map.clone(),
                ..input.clone()
            },
            source_code,
        )?;

        let expr: Expr = constructor.from_pars(left_result.par.clone(), right_result.par.clone());

        Ok(ProcVisitOutputs {
            par: prepend_expr(input.par, expr, input.bound_map_chain.depth() as i32),
            free_map: right_result.free_map,
        })
    }

    //help func for unary cases:
    fn handle_unary_expression(
        p_node: Node,
        input: ProcVisitInputs,
        source_code: &[u8],
        constructor: Box<dyn UnaryExpr>,
    ) -> Result<ProcVisitOutputs, Box<dyn Error>> {
        let sub_proc_node = p_node
            .child_by_field_name("proc")
            .ok_or("Expected a child node for unary expression but found None")?;

        unary_exp(sub_proc_node, input, source_code, constructor)
    }

    //help func for binary cases:
    fn handle_binary_expression(
        p_node: Node,
        input: ProcVisitInputs,
        source_code: &[u8],
        constructor: Box<dyn BinaryExpr>,
    ) -> Result<ProcVisitOutputs, Box<dyn Error>> {
        let left_node = p_node
            .child_by_field_name("left")
            .ok_or("Expected a left node but found None")?;
        let right_node = p_node
            .child_by_field_name("right")
            .ok_or("Expected a right node but found None")?;
        binary_exp(left_node, right_node, input, source_code, constructor)
    }

    println!("Normalizing node kind: {}", p_node.kind());

    match p_node.kind() {
        // "bundle" => {
        //     println!("Found a bundle node, calling normalize_p_bundle");
        //     normalize_p_bundle(p_node, input, source_code)
        // }
        // "long_literal" | "bool_literal" | "string_literal" | "uri_literal" => {
        //     println!("Found a ground node, calling normalize_p_ground");
        //     normalize_p_ground(p_node, input, source_code)
        // }
        "collection" => normalize_p_collect(p_node, input, source_code),
        "matches" => {
            println!("Found a matches node, calling normalize_p_matches");
            normalize_p_matches(p_node, input, source_code)
        }
        // "conjunction" => {
        //     println!("Found a conjunction node, calling normalize_p_conjunction");
        //     normalize_p_сonjunction(p_node, input, source_code)
        // }
        // "disjunction" => {
        //     println!("Found a disjunction node, calling normalize_p_disjunction");
        //     normalize_p_disjunction(p_node, input, source_code)
        // }
        // "contract" => {
        //     println!("Found a contract node, calling normalize_p_contract");
        //     normalize_p_contr(p_node, input, source_code)
        // }
        // "eval" => {
        //     println!("Found an eval node, calling normalize_p_eval");
        //     normalize_p_eval(p_node, input, source_code)
        // }
        // "send" => {
        //     println!("Found a send node, calling normalize_p_send");
        //     normalize_p_send(p_node, input, source_code)
        // }
        "method" => {
            println!("Found a method node, calling normalize_p_method");
            normalize_p_method(p_node, input, source_code)
        }
        // "par" => {
        //     println!("Found a par node, calling normalize_p_par");
        //     normalize_p_par(p_node, input, source_code)
        // }
        "negation" => {
            println!("Found a negation node, calling normalize_p_negation");
            normalize_p_negation(p_node, input, source_code)
        }
        //TODO should be tested, this is not possible now as we have deleted p_var_normalizer
        "ifElse" => match p_node.child_by_field_name("alternative") {
            Some(alternative_node) => {
                normalize_p_if(p_node, input, Option::from(alternative_node), source_code)
            }
            None => normalize_p_if(p_node, input, None, source_code),
        },
        "nil" => Ok(ProcVisitOutputs {
            par: input.par.clone(),
            free_map: input.free_map.clone(),
        }),
        "block" => normalize_match(
            p_node.child_by_field_name("body").unwrap(),
            input,
            source_code,
        ),

        //unary
        "not" => handle_unary_expression(p_node, input, source_code, Box::new(ENot::default())),
        "neg" => handle_unary_expression(p_node, input, source_code, Box::new(ENeg::default())),

        //binary
        "mult" => handle_binary_expression(p_node, input, source_code, Box::new(EMult::default())),
        "div" => handle_binary_expression(p_node, input, source_code, Box::new(EDiv::default())),
        "mod" => handle_binary_expression(p_node, input, source_code, Box::new(EMod::default())),
        "percent_percent" => handle_binary_expression(
            p_node,
            input,
            source_code,
            Box::new(EPercentPercent::default()),
        ),
        "add" => handle_binary_expression(p_node, input, source_code, Box::new(EPlus::default())),
        "minus" => {
            handle_binary_expression(p_node, input, source_code, Box::new(EMinus::default()))
        }
        "plus_plus" => {
            handle_binary_expression(p_node, input, source_code, Box::new(EPlusPlus::default()))
        }
        "minus_minus" => {
            handle_binary_expression(p_node, input, source_code, Box::new(EMinusMinus::default()))
        }
        "lt" => handle_binary_expression(p_node, input, source_code, Box::new(ELt::default())),
        "lte" => handle_binary_expression(p_node, input, source_code, Box::new(ELte::default())),
        "gt" => handle_binary_expression(p_node, input, source_code, Box::new(EGt::default())),
        "gte" => handle_binary_expression(p_node, input, source_code, Box::new(EGte::default())),
        "eq" => handle_binary_expression(p_node, input, source_code, Box::new(EEq::default())),
        "neq" => handle_binary_expression(p_node, input, source_code, Box::new(ENeq::default())),
        "and" => handle_binary_expression(p_node, input, source_code, Box::new(EAnd::default())),
        "or" => handle_binary_expression(p_node, input, source_code, Box::new(EOr::default())),
        _ => Err(format!(
            "Compilation of construct not yet supported.: {}",
            p_node.kind()
        )
        .into()),
    }
}

pub fn normalize_match_proc(
    proc: &Proc,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    fn unary_exp(
        sub_proc: &Proc,
        input: ProcVisitInputs,
        constructor: Box<dyn UnaryExpr>,
    ) -> Result<ProcVisitOutputs, InterpreterError> {
        let sub_result = normalize_match_proc(sub_proc, input.clone())?;
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
    ) -> Result<ProcVisitOutputs, InterpreterError> {
        let left_result = normalize_match_proc(left_proc, input.clone())?;
        let right_result = normalize_match_proc(
            right_proc,
            ProcVisitInputs {
                par: Par::default(),
                free_map: left_result.free_map.clone(),
                ..input.clone()
            },
        )?;

        let expr: Expr = constructor.from_pars(left_result.par.clone(), right_result.par.clone());

        Ok(ProcVisitOutputs {
            par: prepend_expr(input.par, expr, input.bound_map_chain.depth() as i32),
            free_map: right_result.free_map,
        })
    }

    match proc {
        Proc::Par { left, right, .. } => normalize_p_par(left, right, input),

        Proc::SendSync {
            name,
            messages,
            cont,
            line_num,
            col_num,
        } => todo!(),

        Proc::New {
            decls,
            proc,
            line_num,
            col_num,
        } => todo!(),

        Proc::IfElse {
            condition,
            if_true,
            alternative,
            line_num,
            col_num,
        } => {
            todo!()
        }

        Proc::Let {
            decls,
            body,
            line_num,
            col_num,
        } => todo!(),

        Proc::Bundle {
            bundle_type,
            proc,
            line_num,
            col_num,
        } => normalize_p_bundle(bundle_type, proc, input, *line_num, *col_num),

        Proc::Match {
            expression,
            cases,
            line_num,
            col_num,
        } => todo!(),

        Proc::Choice {
            branches,
            line_num,
            col_num,
        } => todo!(),

        Proc::Contract {
            name,
            formals,
            proc,
            line_num,
            col_num,
        } => normalize_p_contr(name, formals, proc, input),

        Proc::Input {
            formals,
            proc,
            line_num,
            col_num,
        } => todo!(),

        Proc::Send {
            name,
            send_type,
            inputs,
            line_num,
            col_num,
        } => normalize_p_send(name, send_type, inputs, input),

        Proc::Matches {
            left,
            right,
            line_num,
            col_num,
        } => todo!(),

        // binary
        Proc::Mult { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMult::default()))
        }

        Proc::PercentPercent { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EPercentPercent::default()))
        }

        Proc::Minus { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMinus::default()))
        }

        // PlusPlus
        Proc::Concat { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EPlusPlus::default()))
        }

        Proc::MinusMinus { left, right, .. } => {
            binary_exp(left, right, input, Box::new(EMinusMinus::default()))
        }

        Proc::Div { left, right, .. } => binary_exp(left, right, input, Box::new(EDiv::default())),
        Proc::Mod { left, right, .. } => binary_exp(left, right, input, Box::new(EMod::default())),
        Proc::Add { left, right, .. } => binary_exp(left, right, input, Box::new(EPlus::default())),
        Proc::Or { left, right, .. } => binary_exp(left, right, input, Box::new(EOr::default())),
        Proc::And { left, right, .. } => binary_exp(left, right, input, Box::new(EAnd::default())),
        Proc::Eq { left, right, .. } => binary_exp(left, right, input, Box::new(EEq::default())),
        Proc::Neq { left, right, .. } => binary_exp(left, right, input, Box::new(ENeq::default())),
        Proc::Lt { left, right, .. } => binary_exp(left, right, input, Box::new(ELt::default())),
        Proc::Lte { left, right, .. } => binary_exp(left, right, input, Box::new(ELte::default())),
        Proc::Gt { left, right, .. } => binary_exp(left, right, input, Box::new(EGt::default())),
        Proc::Gte { left, right, .. } => binary_exp(left, right, input, Box::new(EGte::default())),

        // unary
        Proc::Not { proc: sub_proc, .. } => unary_exp(sub_proc, input, Box::new(ENot::default())),
        Proc::Neg { proc: sub_proc, .. } => unary_exp(sub_proc, input, Box::new(ENeg::default())),

        Proc::Method {
            receiver,
            name,
            args,
            line_num,
            col_num,
        } => todo!(),

        Proc::Parenthesized {
            proc_expression,
            line_num,
            col_num,
        } => todo!(),

        Proc::Eval(eval) => normalize_p_eval(eval, input),

        Proc::Quote(quote) => todo!(),

        Proc::Disjunction(disjunction) => normalize_p_disjunction(disjunction, input),

        Proc::Conjunction(conjunction) => normalize_p_conjunction(conjunction, input),

        Proc::Negation(negation) => todo!(),

        Proc::Block(block) => todo!(),

        Proc::Collection(collection) => todo!(),

        Proc::SimpleType(simple_type) => todo!(),

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
