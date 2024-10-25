use super::exports::*;
use crate::rust::interpreter::compiler::normalizer::processes::p_var_normalizer::normalize_p_var;
use crate::rust::interpreter::compiler::normalizer::processes::p_var_ref_normalizer::normalize_p_var_ref;
use crate::rust::interpreter::compiler::rholang_ast::{PVar, PVarRef, VarRefKind};
use crate::rust::interpreter::compiler::utils::{BinaryExpr, UnaryExpr};
use crate::rust::interpreter::util::prepend_expr;
use models::rhoapi::{Connective, EAnd, EDiv, EEq, EGt, EGte, ELt, ELte, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr, EPercentPercent, EPlus, EPlusPlus, Expr, Par};
use std::error::Error;
use models::rust::utils::union;
use tree_sitter::Node;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;

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

    let line_num = p_node.start_position().row;
    let col_num = p_node.start_position().column;

    match p_node.kind() {
        "bundle" => {
            println!("Found a bundle node, calling normalize_p_bundle");
            normalize_p_bundle(p_node, input, source_code)
        }
        "long_literal" | "bool_literal" | "string_literal" | "uri_literal" => {
            println!("Found a ground node, calling normalize_p_ground");
            normalize_p_ground(p_node, input, source_code)
        }
        "collection" => normalize_p_collect(p_node, input, source_code),
        "matches" => {
            println!("Found a matches node, calling normalize_p_matches");
            normalize_p_matches(p_node, input, source_code)
        }
        "conjunction" => {
            println!("Found a conjunction node, calling normalize_p_conjunction");
            normalize_p_сonjunction(p_node, input, source_code)
        }
        "disjunction" => {
            println!("Found a disjunction node, calling normalize_p_disjunction");
            normalize_p_disjunction(p_node, input, source_code)
        }
        "contract" => {
            println!("Found a contract node, calling normalize_p_contract");
            normalize_p_contr(p_node, input, source_code)
        }
         "nil" => Ok(ProcVisitOutputs {
            par: input.par.clone(),
            free_map: input.free_map.clone(),
        }),
        "block" => normalize_match(
            p_node.child_by_field_name("body").unwrap(),
            input,
            source_code,
        ),

        "var_ref" => {
            let var_ref_kind_node = p_node
                .child_by_field_name("var_ref_kind")
                .ok_or("Expected a var_ref_kind node but found None")?;
            let var_node = p_node
                .child_by_field_name("var")
                .ok_or("Expected a var node but found None")?;

            let var_ref_kind = match std::str::from_utf8(
                &source_code[var_ref_kind_node.start_byte()..var_ref_kind_node.end_byte()],
            )? {
                "=" => VarRefKind::Proc,
                "= *" => VarRefKind::Name,
                _ => return Err("Unexpected var_ref_kind".into()),
            };

            let var =
                std::str::from_utf8(&source_code[var_node.start_byte()..var_node.end_byte()])?
                    .to_string();

            Ok(normalize_p_var_ref(
                PVarRef {
                    var_ref_kind,
                    var,
                    line_num,
                    col_num,
                },
                input,
            )?)
        }
        // "proc_var" => {
        //   println!("called proc_var");
        //   normalize_match(p_node, input, source_code)
        // }
        "var" => {
            let var_name =
                std::str::from_utf8(&source_code[p_node.start_byte()..p_node.end_byte()])?
                    .to_string();

            Ok(normalize_p_var(
                PVar::ProcVarVar {
                    name: var_name,
                    line_num,
                    col_num,
                },
                input,
            )?)
        }

        "wildcard" => Ok(normalize_p_var(
            PVar::ProcVarWildcard { line_num, col_num },
            input,
        )?),

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
