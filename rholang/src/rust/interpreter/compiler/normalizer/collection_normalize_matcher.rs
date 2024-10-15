use models::rhoapi::{EList, ETuple, Expr, Par, Var};
use models::rust::utils::union;
use super::exports::*;
use tree_sitter::Node;
use std::error::Error;
use std::result::Result;
use models::rhoapi::expr::ExprInstance;
use models::rust::par_map::ParMap;
use models::rust::par_map_type_mapper::ParMapTypeMapper;
use models::rust::par_set::ParSet;
use models::rust::par_set_type_mapper::ParSetTypeMapper;
use models::rust::sorted_par_hash_set::SortedParHashSet;
use models::rust::sorted_par_map::SortedParMap;
use crate::rust::interpreter::compiler::exports::FreeMap;
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::compiler::normalizer::remainder_normalizer_matcher::normalize_remainder;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;

pub fn normalize_collection(
  node: Node,
  input: CollectVisitInputs,
  source_code: &[u8],
) -> Result<CollectVisitOutputs, Box<dyn Error>> {
  println!("Normalizing collection node of kind: {}", node.kind());

  pub fn fold_match<F>(
    known_free: FreeMap<VarSort>,
    nodes: Vec<Node>,
    constructor: F,
    source_code: &[u8],
    input: CollectVisitInputs,
  ) -> Result<CollectVisitOutputs, Box<dyn Error>>
  where
    F: Fn(Vec<Par>, Vec<u8>, bool) -> Expr,
  {
    let init = (vec![], known_free.clone(), Vec::new(), false);
    let (mut acc_pars, mut result_known_free, mut locally_free, mut connective_used) = init;

    for node in nodes {
      let result = normalize_match(
        node,
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: result_known_free.clone(),
        },
        source_code,
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
    nodes: Vec<Node>,
    source_code: &[u8],
    input: CollectVisitInputs,
  ) -> Result<CollectVisitOutputs, Box<dyn Error>> {
    let init = (vec![], known_free.clone(), Vec::new(), false);

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
      locally_free = union(locally_free, union(key_result.par.locally_free, value_result.par.locally_free));
      connective_used = connective_used || key_result.par.connective_used || value_result.par.connective_used;
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
      expr_instance: Some(ExprInstance::EMapBody(
        ParMapTypeMapper::par_map_to_emap(ParMap {
          ps: SortedParMap::create_from_vec(acc_pairs.clone().into_iter().rev().collect()),
          connective_used: connective_used || remainder_connective_used,
          locally_free: union(locally_free, remainder_locally_free),
          remainder: remainder.clone(),
        })
      ))
    };

    Ok(CollectVisitOutputs {
      expr,
      free_map: result_known_free,
    })
  }

  // Determine the type of subtype (list, set, tuple, map) for this collection node
  let collection_type = node.child(0).ok_or("Expected a child node for collection type but found None")?;
  match collection_type.kind() {
    "list" => {
      let (optional_remainder, known_free) = normalize_remainder(node, input.free_map.clone(), source_code)?;

      let mut list_children = vec![];
      for child in collection_type.named_children(&mut node.walk()) {
        // Ignore the "cont" node, because it is already processed in normalize_remainder
        if child.kind() == "cont" {
          continue;
        }
        list_children.push(child);
      }

      // Returned Expr, because I can't convert EList to Expr implicitly (implicit toExpr: T => Expr) as in Scala
      let constructor = |ps: Vec<Par>, locally_free: Vec<u8>, connective_used: bool| -> Expr {
        let mut tmp_e_list = EList {
          ps,
          locally_free,
          connective_used,
          remainder: optional_remainder.clone(),
        };

        tmp_e_list.connective_used = tmp_e_list.connective_used || optional_remainder.is_some();
        Expr {
          expr_instance: Some(ExprInstance::EListBody(tmp_e_list)),
        }
      };

      fold_match(known_free, list_children, constructor, source_code, input)
    }

    "tuple" => {
      //We don't have any single and multiple tuple, so just describe simple tuple
      let mut tuple_children = vec![];
      for child in collection_type.named_children(&mut node.walk()) {
        tuple_children.push(child);
      }

      let constructor = |ps: Vec<Par>, locally_free: Vec<u8>, connective_used: bool| -> Expr {
        let tmp_tuple = ETuple {
          ps,
          locally_free,
          connective_used,
        };

        Expr {
          expr_instance: Some(ExprInstance::ETupleBody(tmp_tuple)),
        }
      };

      fold_match(input.free_map.clone(), tuple_children, constructor, source_code, input)
    }

    "set" => {
      let (optional_remainder, known_free) = normalize_remainder(node, input.free_map.clone(), source_code)?;
      let mut set_children = vec![];
      for child in collection_type.named_children(&mut node.walk()) {
        if child.kind() == "cont" {
          continue;
        }
        set_children.push(child);
      }

      let constructor = |pars: Vec<Par>, locally_free: Vec<u8>, connective_used: bool| -> Expr {
        let mut tmp_par_set = ParSet {
          ps: SortedParHashSet::create_from_vec(pars),
          locally_free,
          connective_used,
          remainder: optional_remainder.clone(),
        };

        tmp_par_set.connective_used = tmp_par_set.connective_used || optional_remainder.is_some();

        let eset = ParSetTypeMapper::par_set_to_eset(tmp_par_set);

        Expr {
          expr_instance: Some(ExprInstance::ESetBody(eset)),
        }
      };

      fold_match(known_free, set_children, constructor, source_code, input)
    }

    "map" => {
      let (optional_remainder, known_free) = normalize_remainder(node, input.free_map.clone(), source_code)?;
      let mut map_children = vec![];
      for child in collection_type.named_children(&mut node.walk()) {
        if child.kind() == "cont" {
          continue;
        }
        map_children.push(child);
      }

      fold_match_map(known_free, optional_remainder, map_children, source_code, input)
    }

    _ => Err(InterpreterError::NormalizerError("Unexpected collection node kind".to_string()).into()),
  }
}

/*
  I wrote 4 tests to understand if our normalizer understands collection subtypes - list, set, tuple and map.
  These tests are not the same as we have inside CollectMatcherSpec.scala class.
 */
#[test]
fn test_normalize_collection_list() {
  let rholang_code = r#"{[1,2,3]}"#;
  let tree = parse_rholang_code(rholang_code);
  let (collection_node, input) = setup_collection_test(&tree);

  match normalize_match(collection_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      if let Some(ExprInstance::EListBody(e_list)) = &result.par.exprs.get(0).unwrap().expr_instance {
        assert_eq!(e_list.ps.len(), 3, "Expected 3 elements in the normalized list");
        for (i, par) in e_list.ps.iter().enumerate() {
          if let Some(ExprInstance::GInt(value)) = &par.exprs.get(0).unwrap().expr_instance {
            assert_eq!(*value, (i + 1) as i64, "Expected value {} but found {}", i + 1, value);
          } else {
            panic!("Expected GInt but found something else");
          }
        }
      } else {
        panic!("Expected EListBody but found something else");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}

#[test]
fn test_normalize_collection_set() {
  let rholang_code = r#"{Set("1", "2", "3")}"#;
  let tree = parse_rholang_code(rholang_code);
  let (collection_node, input) = setup_collection_test(&tree);

  match normalize_match(collection_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      if let Some(ExprInstance::ESetBody(set)) = &result.par.exprs[0].expr_instance {
        let values: Vec<_> = set.ps.iter().map(|par| {
          if let Some(ExprInstance::GString(ref s)) = &par.exprs[0].expr_instance {
            s.clone()
          } else {
            panic!("Expected GString in set but found different type");
          }
        }).collect();

        assert_eq!(values.len(), 3);
        assert!(values.contains(&"1".to_string()));
        assert!(values.contains(&"2".to_string()));
        assert!(values.contains(&"3".to_string()));
      } else {
        panic!("Expected ESetBody but found different type");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}

#[test]
fn test_normalize_collection_map() {
  let rholang_code = r#"{{"key1":123}}"#;
  let tree = parse_rholang_code(rholang_code);
  let (collection_node, input) = setup_collection_test(&tree);

  match normalize_match(collection_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      if let Some(ExprInstance::EMapBody(map)) = &result.par.exprs[0].expr_instance {
        assert_eq!(map.kvs.len(), 1, "Expected 1 key-value pair in the map");
        let key_value = &map.kvs[0];

        if let Some(key) = &key_value.key {
          if let Some(ExprInstance::GString(ref key_str)) = &key.exprs[0].expr_instance {
            assert_eq!(key_str, "key1", "Expected key 'key1' but found {}", key_str);
          } else {
            panic!("Expected GString as key but found different type");
          }
        } else {
          panic!("Expected key but found None");
        }

        if let Some(value) = &key_value.value {
          if let Some(ExprInstance::GInt(value_int)) = &value.exprs[0].expr_instance {
            assert_eq!(*value_int, 123, "Expected value 123 but found {}", value_int);
          } else {
            panic!("Expected GInt as value but found different type");
          }
        } else {
          panic!("Expected value but found None");
        }
      } else {
        panic!("Expected EMapBody but found different type");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}

#[test]
fn test_normalize_collection_tuple() {
  let rholang_code = r#"{(1,"2")}"#;
  let tree = parse_rholang_code(rholang_code);
  let (collection_node, input) = setup_collection_test(&tree);

  match normalize_match(collection_node, input, rholang_code.as_bytes()) {
    Ok(result) => {
      if let Some(ExprInstance::ETupleBody(tuple)) = &result.par.exprs[0].expr_instance {
        assert_eq!(tuple.ps.len(), 2, "Expected 2 elements in the tuple");

        if let Some(ExprInstance::GInt(value)) = &tuple.ps[0].exprs[0].expr_instance {
          assert_eq!(*value, 1, "Expected value 1 but found {}", value);
        } else {
          panic!("Expected GInt as first element but found different type");
        }

        if let Some(ExprInstance::GString(ref value)) = &tuple.ps[1].exprs[0].expr_instance {
          assert_eq!(value, "2", "Expected value '2' but found {}", value);
        } else {
          panic!("Expected GString as second element but found different type");
        }
      } else {
        panic!("Expected ETupleBody but found different type");
      }
    }
    Err(e) => {
      println!("Normalization failed: {}", e);
      panic!("Test failed due to normalization error");
    }
  }
}

fn setup_collection_test<'a>(tree: &'a tree_sitter::Tree) -> (tree_sitter::Node<'a>, ProcVisitInputs) {
  let root_node = tree.root_node();
  println!("Tree S-expression: {}", root_node.to_sexp());
  println!("Root node kind: {}", root_node.kind());

  let block_node = root_node.child(0).expect("Expected a block node");
  println!("Found block node: {}", block_node.to_sexp());

  let collection_node = block_node.child_by_field_name("body").expect("Expected a collection node");
  println!("Found collection node: {}", collection_node.to_sexp());

  let input = ProcVisitInputs {
    par: Par::default(),
    bound_map_chain: Default::default(),
    free_map: Default::default(),
  };

  (collection_node, input)
}