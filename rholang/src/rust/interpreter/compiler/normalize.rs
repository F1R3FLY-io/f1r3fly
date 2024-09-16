use std::error::Error;
use tree_sitter::Node;
use super::exports::*;
use models::rhoapi::{Connective, Expr, Par};
use models::rust::utils::union;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;

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
  match p_node.kind() {
    "PBundle" => normalize_p_bundle(p_node, input, source_code),
    "PNil" => Ok(ProcVisitOutputs {
      par: input.par.clone(),
      free_map: input.free_map.clone(),
    }),
    _ => Err(format!("Unknown process type: {}", p_node.kind()).into()),
  }
}

pub fn binary_exp<T>(
  left_proc_node: Node,
  right_proc_node: Node,
  input: ProcVisitInputs,
  source_code: &[u8],
  constructor: impl FnOnce(Par, Par) -> T,
) -> Result<ProcVisitOutputs, Box<dyn Error>>
where
  T: Into<Expr>,
{
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

  let expr = constructor(left_result.par.clone(), right_result.par.clone());

  Ok(ProcVisitOutputs {
    par: prepend_expr(input.par, expr.into(), input.bound_map_chain.depth() as i32),
    free_map: right_result.free_map,
  })
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







