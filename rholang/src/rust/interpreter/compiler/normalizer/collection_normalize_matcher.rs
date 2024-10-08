use models::rhoapi::{Expr,Par, Var};
use super::exports::*;
use tree_sitter::Node;
use std::error::Error;
use std::result::Result;
use std::collections::HashSet;
use models::rhoapi::expr::ExprInstance;
use models::rust::par_map::ParMap;
use models::rust::par_map_type_mapper::ParMapTypeMapper;
use models::rust::sorted_par_map::SortedParMap;
use crate::rust::interpreter::compiler::exports::FreeMap;
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;

pub fn normalize_collection(
  node: Node,
  input: CollectVisitInputs,
  source_code: &[u8],
) -> Result<CollectVisitOutputs, Box<dyn Error>> {
    println!("Normalizing bundle node of kind: {}", node.kind());

  pub fn fold_match<T, F>(
    known_free: FreeMap<VarSort>,
    nodes: Vec<Node>,
    constructor: F,
    source_code: &[u8],
    input: CollectVisitInputs
  ) -> Result<CollectVisitOutputs, Box<dyn Error>>
  where
    F: Fn(Vec<Par>, HashSet<usize>, bool) -> T,
    T: Into<Expr>,
  {
    let init = (vec![], known_free.clone(), HashSet::new(), false);

    let (mut acc_pars, mut result_known_free, mut locally_free, mut connective_used) = init;

    //repeat the foldM iterative logic for all nodes(proc in Scala) from list_node (listproc in Scala)
    for node in nodes {
      let result = normalize_match(
        node,
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: result_known_free.clone(), //acc._2 is the same to known_free in Rust case
        },
        source_code,
      )?;

      acc_pars.push(result.par.clone());
      result_known_free = result.free_map.clone();

      // union - because in Scala we use BitSet collection which collect only unique values
      let par_locally_free: HashSet<usize> = result.par.locally_free.iter().map(|&x| x as usize).collect();
      locally_free = locally_free.union(&par_locally_free).cloned().collect();

      connective_used = connective_used || result.par.connective_used;
    }

    let constructed_expr: T = constructor(acc_pars, locally_free, connective_used);
    let expr: Expr = constructed_expr.into();
    /* Does order matter? In Scala, we add element to the beginning of Vector, and then do ps.reverse logic,
    but inside Rust implementation logic added element to the end of vec used push logic. So, we don't need reverse logic here.
    */
    Ok(CollectVisitOutputs {
      expr,
      free_map: result_known_free,
    })
  }

  pub fn fold_match_map(
    known_free: FreeMap<VarSort>,
    remainder: Option<Var>,
    nodes: Vec<Node>,
    source_code: &[u8],
    input: CollectVisitInputs,
  ) -> Result<CollectVisitOutputs, Box<dyn Error>> {
    let init = (vec![], known_free.clone(), HashSet::new(), false);

    let (mut acc_pairs, mut result_known_free, mut locally_free, mut connective_used) = init;

    for key_value_pair in nodes {
      let key_result = normalize_match(
        key_value_pair.child_by_field_name("key").unwrap(),
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: result_known_free.clone(),
        },
        source_code,
      )?;

      let value_result = normalize_match(
        key_value_pair.child_by_field_name("value").unwrap(),
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: key_result.free_map.clone(),
        },
        source_code,
      )?;

      acc_pairs.push((key_result.par.clone(), value_result.par.clone()));
      result_known_free = value_result.free_map.clone();

      // union - because in Scala we use BitSet collection which collect only unique values
      let key_locally_free: HashSet<usize> = key_result.par.locally_free.iter().map(|&x| x as usize).collect();
      let value_locally_free: HashSet<usize> = value_result.par.locally_free.iter().map(|&x| x as usize).collect();
      locally_free = locally_free.union(&key_locally_free).cloned().collect();
      locally_free = locally_free.union(&value_locally_free).cloned().collect();
      connective_used = connective_used || key_result.par.connective_used || value_result.par.connective_used;
    }

    //let remainder_connective_used = remainder.unwrap().connective_used(remainder.clone().unwrap());
    let remainder_connective_used = match remainder {
      Some(ref var) => var.connective_used(var.clone()),
      None => return Err(InterpreterError::NormalizerError("Undefined remainder".to_string()).into()),
    };


    //let remainder_locally_free = remainder.clone().unwrap().locally_free(remainder.clone().unwrap(), 0);
    let remainder_locally_free = match remainder {
      Some(ref var) => var.locally_free(var.clone(), 0),
      None => Vec::new(),
    };

    let locally_free_set: HashSet<usize> = locally_free.into_iter().map(|x| x as usize).collect();
    let remainder_locally_free_set: HashSet<usize> = remainder_locally_free.into_iter().map(|x| x as usize).collect();

    let combined_locally_free_set: HashSet<usize> = locally_free_set
      .union(&remainder_locally_free_set)
      .cloned()
      .collect();

    let combined_locally_free: Vec<u8> = combined_locally_free_set.into_iter().map(|x| x as u8).collect();

    let expr = Expr {
      expr_instance: Some(ExprInstance::EMapBody(
        ParMapTypeMapper::par_map_to_emap(ParMap {
          ps: SortedParMap::create_from_vec(acc_pairs.clone().into_iter().rev().collect()),
          connective_used: connective_used || remainder_connective_used,
          locally_free: combined_locally_free,
          remainder: remainder.clone(),
        })
      ))
    };

    Ok(CollectVisitOutputs {
      expr,
      free_map: result_known_free,
    })
  }

  todo!()
}
