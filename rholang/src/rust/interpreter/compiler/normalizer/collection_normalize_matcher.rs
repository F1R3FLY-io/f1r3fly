use models::rhoapi::{EList, ETuple, Expr, Par, Var};
use models::rust::utils::union;
use super::exports::*;
use std::result::Result;
use models::rhoapi::expr::ExprInstance;
use models::rust::par_map::ParMap;
use models::rust::par_map_type_mapper::ParMapTypeMapper;
use models::rust::par_set::ParSet;
use models::rust::par_set_type_mapper::ParSetTypeMapper;
use models::rust::sorted_par_hash_set::SortedParHashSet;
use models::rust::sorted_par_map::SortedParMap;
use crate::rust::interpreter::compiler::exports::FreeMap;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, VarSort};
use crate::rust::interpreter::compiler::normalizer::remainder_normalizer_matcher::normalize_remainder;
use crate::rust::interpreter::compiler::rholang_ast::{Collection, KeyValuePair, Proc};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::matcher::has_locally_free::HasLocallyFree;

pub fn normalize_collection(
  proc: &Collection,
  input: CollectVisitInputs,
) -> Result<CollectVisitOutputs, InterpreterError> {
  pub fn fold_match<F>(
    known_free: FreeMap<VarSort>,
    elements: &Vec<Proc>,
    constructor: F,
    input: CollectVisitInputs,
  ) -> Result<CollectVisitOutputs, InterpreterError>
  where
    F: Fn(Vec<Par>, Vec<u8>, bool) -> Expr,
  {
    let init = (vec![], known_free.clone(), Vec::new(), false);
    let (mut acc_pars, mut result_known_free, mut locally_free, mut connective_used) = init;

    for element in elements {
      let result = normalize_match_proc(
        element,
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: result_known_free.clone(),
        })?;

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
    pairs: &Vec<KeyValuePair>,
    input: CollectVisitInputs,
  ) -> Result<CollectVisitOutputs, InterpreterError> {
    let init = (vec![], known_free.clone(), Vec::new(), false);

    let (mut acc_pairs, mut result_known_free, mut locally_free, mut connective_used) = init;

    for key_value_pair in pairs {
      let key_result = normalize_match_proc(
        &key_value_pair.key,
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: result_known_free.clone(),
        })?;

      let value_result = normalize_match_proc(
        &key_value_pair.value,
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: key_result.free_map.clone(),
        })?;

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

  match proc {
    Collection::List { elements, cont, .. } => {
      let (optional_remainder, known_free) = normalize_remainder(cont, input.free_map.clone())?;

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

      fold_match(known_free, elements, constructor, input)
    }

    Collection::Tuple { elements, .. } => {
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

      fold_match(input.free_map.clone(), elements, constructor, input)
    }

    Collection::Set { elements, cont, .. } => {
      let (optional_remainder, known_free) =
        normalize_remainder(cont, input.free_map.clone())?;

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

      fold_match(known_free, elements, constructor, input)
    }

    Collection::Map { pairs, cont, .. } => {
      let (optional_remainder, known_free) =
        normalize_remainder(cont, input.free_map.clone())?;

      fold_match_map(known_free, optional_remainder, pairs, input)
    }

    _ => Err(InterpreterError::NormalizerError("Unexpected collection type".to_string()).into()),
  }
}

/*
  I wrote 4 tests to understand if our normalizer understands collection subtypes - list, set, tuple and map.
  These tests are not the same as we have inside CollectMatcherSpec.scala class.
 */
// #[test]
// fn test_normalize_collection_list() {
//   let rholang_code = r#"{[1,2,3]}"#;
//   let tree = parse_rholang_code(rholang_code);
//   let (collection_node, input) = setup_collection_test(&tree);
//
//   match normalize_match(collection_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       if let Some(ExprInstance::EListBody(e_list)) = &result.par.exprs.get(0).unwrap().expr_instance {
//         assert_eq!(e_list.ps.len(), 3, "Expected 3 elements in the normalized list");
//         for (i, par) in e_list.ps.iter().enumerate() {
//           if let Some(ExprInstance::GInt(value)) = &par.exprs.get(0).unwrap().expr_instance {
//             assert_eq!(*value, (i + 1) as i64, "Expected value {} but found {}", i + 1, value);
//           } else {
//             panic!("Expected GInt but found something else");
//           }
//         }
//       } else {
//         panic!("Expected EListBody but found something else");
//       }
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }

// #[test]
// fn test_normalize_collection_set() {
//   let rholang_code = r#"{Set("1", "2", "3")}"#;
//   let tree = parse_rholang_code(rholang_code);
//   let (collection_node, input) = setup_collection_test(&tree);
//
//   match normalize_match(collection_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       if let Some(ExprInstance::ESetBody(set)) = &result.par.exprs[0].expr_instance {
//         let values: Vec<_> = set.ps.iter().map(|par| {
//           if let Some(ExprInstance::GString(ref s)) = &par.exprs[0].expr_instance {
//             s.clone()
//           } else {
//             panic!("Expected GString in set but found different type");
//           }
//         }).collect();
//
//         assert_eq!(values.len(), 3);
//         assert!(values.contains(&"1".to_string()));
//         assert!(values.contains(&"2".to_string()));
//         assert!(values.contains(&"3".to_string()));
//       } else {
//         panic!("Expected ESetBody but found different type");
//       }
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
//
// #[test]
// fn test_normalize_collection_map() {
//   let rholang_code = r#"{{"key1":123}}"#;
//   let tree = parse_rholang_code(rholang_code);
//   let (collection_node, input) = setup_collection_test(&tree);
//
//   match normalize_match(collection_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       if let Some(ExprInstance::EMapBody(map)) = &result.par.exprs[0].expr_instance {
//         assert_eq!(map.kvs.len(), 1, "Expected 1 key-value pair in the map");
//         let key_value = &map.kvs[0];
//
//         if let Some(key) = &key_value.key {
//           if let Some(ExprInstance::GString(ref key_str)) = &key.exprs[0].expr_instance {
//             assert_eq!(key_str, "key1", "Expected key 'key1' but found {}", key_str);
//           } else {
//             panic!("Expected GString as key but found different type");
//           }
//         } else {
//           panic!("Expected key but found None");
//         }
//
//         if let Some(value) = &key_value.value {
//           if let Some(ExprInstance::GInt(value_int)) = &value.exprs[0].expr_instance {
//             assert_eq!(*value_int, 123, "Expected value 123 but found {}", value_int);
//           } else {
//             panic!("Expected GInt as value but found different type");
//           }
//         } else {
//           panic!("Expected value but found None");
//         }
//       } else {
//         panic!("Expected EMapBody but found different type");
//       }
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
//
// #[test]
// fn test_normalize_collection_tuple() {
//   let rholang_code = r#"{(1,"2")}"#;
//   let tree = parse_rholang_code(rholang_code);
//   let (collection_node, input) = setup_collection_test(&tree);
//
//   match normalize_match(collection_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       if let Some(ExprInstance::ETupleBody(tuple)) = &result.par.exprs[0].expr_instance {
//         assert_eq!(tuple.ps.len(), 2, "Expected 2 elements in the tuple");
//
//         if let Some(ExprInstance::GInt(value)) = &tuple.ps[0].exprs[0].expr_instance {
//           assert_eq!(*value, 1, "Expected value 1 but found {}", value);
//         } else {
//           panic!("Expected GInt as first element but found different type");
//         }
//
//         if let Some(ExprInstance::GString(ref value)) = &tuple.ps[1].exprs[0].expr_instance {
//           assert_eq!(value, "2", "Expected value '2' but found {}", value);
//         } else {
//           panic!("Expected GString as second element but found different type");
//         }
//       } else {
//         panic!("Expected ETupleBody but found different type");
//       }
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }
//
// fn setup_collection_test<'a>(tree: &'a tree_sitter::Tree) -> (tree_sitter::Node<'a>, ProcVisitInputs) {
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", root_node.to_sexp());
//   println!("Root node kind: {}", root_node.kind());
//
//   let block_node = root_node.child(0).expect("Expected a block node");
//   println!("Found block node: {}", block_node.to_sexp());
//
//   let collection_node = block_node.child_by_field_name("body").expect("Expected a collection node");
//   println!("Found collection node: {}", collection_node.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: Default::default(),
//   };
//
//   (collection_node, input)
// }