use super::exports::*;
use crate::rust::interpreter::compiler::normalizer::ground_normalize_matcher::Ground;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::{expr, EEq, EGt, ELt, EMinus, ENeg, ENot, Expr, Par, EMult, EDiv, EMod, EPercentPercent, EPlus, EPlusPlus, EMinusMinus, ELte, EGte, ENeq, EAnd, EOr};
use models::rust::utils::union;
use std::error::Error;
use tree_sitter::Node;

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct ProcVisitInputs {
  pub(crate) par: Par,
  pub bound_map_chain: BoundMapChain<VarSort>,
  pub(crate) free_map: FreeMap<VarSort>,
}

// Returns the update Par and an updated map of free variables.
#[derive(Clone, Debug)]
pub struct ProcVisitOutputs {
  pub(crate) par: Par,
  pub(crate) free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct NameVisitInputs {
  bound_map_chain: BoundMapChain<VarSort>,
  free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct NameVisitOutputs {
  par: Par,
  free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct CollectVisitInputs {
  bound_map_chain: BoundMapChain<VarSort>,
  pub(crate) free_map: FreeMap<VarSort>,
}

#[derive(Clone, Debug)]
pub struct CollectVisitOutputs {
  pub(crate) expr: Expr,
  pub(crate) free_map: FreeMap<VarSort>,
}

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

  match p_node.kind() {
    "PBundle" => normalize_p_bundle(p_node, input, source_code),
    "PGround" => normalize_p_ground(p_node, input, source_code),
    "PMatches" => normalize_p_ground(p_node, input, source_code),
    "PNil" => Ok(ProcVisitOutputs {
      par: input.par.clone(),
      free_map: input.free_map.clone(),
    }),

    //unary
    "PNot" => unary_exp(
      p_node.child(0).unwrap(),
      input,
      source_code,
      Box::new(ENot::default()),
    ),
    "PNeg" => unary_exp(
      p_node.child(0).unwrap(),
      input,
      source_code,
      Box::new(ENeg::default()),
    ),

    //binary
    "PMult" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EMult::default()),
    ),
    "PDiv" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EDiv::default()),
    ),
    "PMod" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EMod::default()),
    ),
    "PPercentPercent" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EPercentPercent::default()),
    ),
    "PAdd" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EPlus::default()),
    ),
    "PMinus" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EMinus::default()),
    ),
    "PPlusPlus" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EPlusPlus::default()),
    ),
    "PMinusMinus" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EMinusMinus::default()),
    ),

    "PLt" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(ELt::default()),
    ),
    "PLte" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(ELte::default()),
    ),
    "PGt" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EGt::default()),
    ),
    "PGte" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EGte::default()),
    ),

    "PEq" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EEq::default()),
    ),
    "PNeq" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(ENeq::default()),
    ),

    "PAnd" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EAnd::default()),
    ),
    "POr" => binary_exp(
      p_node.child(0).unwrap(),
      p_node.child(1).unwrap(),
      input,
      source_code,
      Box::new(EOr::default()),
    ),

    _ => Err(format!("Compilation of construct not yet supported.: {}", p_node.kind()).into()),
  }
}

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - prepend
pub fn prepend_expr(mut p: Par, e: Expr, depth: i32) -> Par {
  let mut new_exprs = vec![e.clone()];
  new_exprs.append(&mut p.exprs);

  Par {
    exprs: new_exprs,
    locally_free: union(p.locally_free.clone(), e.locally_free(e.clone(), depth)),
    connective_used: p.connective_used || e.clone().connective_used(e),
    ..p.clone()
  }
}

// I'm not sure about this func, but I need it for p_ground_normalizer.rs
pub fn ground_to_expr(ground: Ground) -> Expr {
  match ground {
    Ground::Bool(value) => Expr {
      expr_instance: Some(expr::ExprInstance::GBool(value)),
    },
    Ground::Int(value) => Expr {
      expr_instance: Some(expr::ExprInstance::GInt(value)),
    },
    Ground::String(value) => Expr {
      expr_instance: Some(expr::ExprInstance::GString(value)),
    },
    Ground::Uri(value) => Expr {
      expr_instance: Some(expr::ExprInstance::GUri(value)),
    },
  }
}

pub trait UnaryExpr {
  fn from_par(&self, p: Par) -> Expr;
}

pub trait BinaryExpr {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr;
}

impl UnaryExpr for ENot {
  fn from_par(&self, p: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::ENotBody(ENot {
        p: Some(p),
      })),
    }
  }
}

impl UnaryExpr for ENeg {
  fn from_par(&self, p: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::ENegBody(ENeg {
        p: Some(p),
      })),
    }
  }
}

impl BinaryExpr for EMult {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EMultBody(EMult {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EDiv {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EDivBody(EDiv {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EMod {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EModBody(EMod {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EPercentPercent {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EPercentPercentBody(EPercentPercent {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EPlus {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EPlusBody(EPlus {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EMinus {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EMinusBody(EMinus {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EPlusPlus {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EPlusPlusBody(EPlusPlus {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EMinusMinus {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EMinusMinusBody(EMinusMinus {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for ELt {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::ELtBody(ELt {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for ELte {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::ELteBody(ELte {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EGt {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EGtBody(EGt {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EGte {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EGteBody(EGte {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EEq {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EEqBody(EEq {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for ENeq {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::ENeqBody(ENeq {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EAnd {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EAndBody(EAnd {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}

impl BinaryExpr for EOr {
  fn from_pars(&self, p1: Par, p2: Par) -> Expr {
    Expr {
      expr_instance: Some(ExprInstance::EOrBody(EOr {
        p1: Some(p1),
        p2: Some(p2),
      })),
    }
  }
}