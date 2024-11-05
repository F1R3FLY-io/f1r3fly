use std::collections::HashMap;
use models::rhoapi::{Match, MatchCase, Par};
use models::rust::utils::union;
use crate::rust::interpreter::compiler::exports::FreeMap;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::rholang_ast::{Case, Proc};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::filter_and_adjust_bitset;

pub fn normalize_p_match(
  expressions: &Box<Proc>,
  cases: &Vec<Case>,
  input: ProcVisitInputs,
  env: &HashMap<String, Par>,
) -> Result<ProcVisitOutputs, InterpreterError> {

  //We don't have any CaseImpl inside Rust AST, so we should work with simple Case struct
  fn lift_case(case: &Case) -> Result<(&Proc, &Proc), InterpreterError> {
    match case {
      Case { pattern, proc, .. } => Ok((pattern, proc)),
    }
  }

  let target_result = normalize_match_proc(
    expressions,
    ProcVisitInputs {
      par: Par::default(),
      ..input.clone()
    },
    env,
  )?;

  let mut init_acc = (vec![], target_result.free_map.clone(), Vec::new(), false);

  for case in cases {
    let (pattern, case_body) = lift_case(case)?;
    let pattern_result = normalize_match_proc(
      pattern,
      ProcVisitInputs {
        par: Par::default(),
        bound_map_chain: input.bound_map_chain.push(),
        free_map: FreeMap::default(),
      },
      env,
    )?;

    let case_env = input.bound_map_chain.absorb_free(pattern_result.free_map.clone());
    let bound_count  = pattern_result.free_map.count_no_wildcards();

    let case_body_result = normalize_match_proc(
      case_body,
      ProcVisitInputs {
        par: Par::default(),
        bound_map_chain: case_env.clone(),
        free_map: init_acc.1.clone(),
      },
      env,
    )?;

    init_acc.0.push(MatchCase {
      pattern: Some(pattern_result.par.clone()),
      source: Some(case_body_result.par.clone()),
      free_count: bound_count as i32,
    });
    init_acc.1 = case_body_result.free_map;
    init_acc.2 = union(
      union(init_acc.2.clone(), pattern_result.par.locally_free.clone()),
      filter_and_adjust_bitset(case_body_result.par.locally_free.clone(), bound_count),
    );
    init_acc.3 = init_acc.3 || case_body_result.par.connective_used;
  }

  let result_match = Match {
    target: Some(target_result.par.clone()),
    cases: init_acc.0.into_iter().rev().collect(),
    locally_free: union(init_acc.2, target_result.par.locally_free.clone()),
    connective_used: init_acc.3 || target_result.par.connective_used.clone(),
  };

  Ok(ProcVisitOutputs {
    par: input.par.clone().prepend_match(result_match.clone()),
    free_map: init_acc.1,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::rust::interpreter::compiler::rholang_ast::{Proc, Case};
  use models::rhoapi::Par;
  use crate::rust::interpreter::compiler::exports::BoundMapChain;

  #[test]
  fn test_normalize_p_match() {
    let expression = Box::new(Proc::LongLiteral {
      value: 42,
      line_num: 1,
      col_num: 1,
    });
    let cases = vec![
      Case {
        pattern: Proc::BoolLiteral {
          value: true,
          line_num: 2,
          col_num: 5,
        },
        proc: Proc::Nil {
          line_num: 3,
          col_num: 1,
        },
        line_num: 2,
        col_num: 5,
      },
    ];
    let input = ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: BoundMapChain::default(),
      free_map: Default::default(),
    };

    let result = normalize_match_proc(&Proc::Match {
      expression: expression.clone(),
      cases: cases.clone(),
      line_num: 1,
      col_num: 1,
    }, input, &HashMap::new());

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.par.matches.len(), 1);
  }
}