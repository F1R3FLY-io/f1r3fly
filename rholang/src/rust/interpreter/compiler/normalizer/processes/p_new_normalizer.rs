use std::collections::{BTreeMap, HashMap};
use models::rhoapi::{expr, New, Par};
use crate::rust::interpreter::compiler::exports::{BoundMapChain, IdContext, SourcePosition};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, ProcVisitInputs, ProcVisitOutputs, VarSort};
use crate::rust::interpreter::compiler::normalizer::ground_normalize_matcher::normalize_ground;
use crate::rust::interpreter::compiler::rholang_ast::{Decls, NameDecl, Proc};
use crate::rust::interpreter::util::prepend_new;
use crate::rust::interpreter::util::filter_and_adjust_bitset;

pub fn normalize_p_new(
  decls: &Decls,
  proc: &Box<Proc>,
  input: ProcVisitInputs,
  env: &HashMap<String, Par>
) -> Result<ProcVisitOutputs, InterpreterError> {
  //TODO need review for new_tagged_bindings
  let new_tagged_bindings:Vec<(Option<String>, String, VarSort, usize, usize)> = decls
    .decls
    .iter()
    .map(|decl| match decl {
      NameDecl { var, uri: None, line_num, col_num } => {
        Ok((None, var.name.clone(), VarSort::NameSort, *line_num, *col_num))
      }
      NameDecl { var, uri: Some(urn), line_num, col_num } => {
        let uri_expr = normalize_ground(&Proc::UriLiteral(urn.clone()))?;
        //normalize ground return an Expr, not an String, so we need to match it for future usage inside sorted_bindings
        match uri_expr.expr_instance {
          Some(expr::ExprInstance::GUri(uri_string)) => {
            Ok((Some(uri_string), var.name.clone(), VarSort::NameSort, *line_num, *col_num))
          }
          _ => Err(InterpreterError::BugFoundError(format!(
            "Expected a URI literal, found: {:?}",
            uri_expr
          )))
        }
      }
    })
    .collect::<Result<Vec<_>, InterpreterError>>()?;


  // Sort bindings: None's first, then URI's lexicographically
  let mut sorted_bindings: Vec<(Option<String>, String, VarSort, usize, usize)> = new_tagged_bindings;
  sorted_bindings.sort_by(|a, b| a.0.cmp(&b.0));

  let new_bindings: Vec<IdContext<VarSort>> = sorted_bindings
    .iter()
    .map(|row| (
      row.1.clone(),
      row.2.clone(),
      SourcePosition::new(row.3, row.4)
    )).collect();


  let uris: Vec<String> = sorted_bindings
    .iter()
    .filter_map(|row| row.0.clone())
    .collect();

  let new_env: BoundMapChain<VarSort> = input.bound_map_chain.put_all(new_bindings);
  let new_count: usize = new_env.get_count() - input.bound_map_chain.get_count();

  let body_result = normalize_match_proc(
    &proc,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: new_env.clone(),
      free_map: input.free_map.clone(),
    }, env)?;

  //we should build btree_map with real values, not a copied references from env: ref &HashMap
  let btree_map: BTreeMap<String, Par> = env.iter()
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();

  let result_new = New {
    bind_count: new_count as i32,
    p: Some(body_result.par.clone()),
    uri: uris,
    injections: btree_map,
    locally_free: filter_and_adjust_bitset(body_result.par.clone().locally_free, new_count),
  };

  Ok(ProcVisitOutputs {
    par: prepend_new(input.par.clone(), result_new),
    free_map: body_result.free_map.clone(),
  })

}

