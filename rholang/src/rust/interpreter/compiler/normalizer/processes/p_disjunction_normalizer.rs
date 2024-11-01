use std::collections::HashMap;
use models::rhoapi::{Connective, ConnectiveBody, Par};
use models::rhoapi::connective::ConnectiveInstance;
use crate::rust::interpreter::compiler::exports::SourcePosition;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::rholang_ast::Disjunction;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;

pub fn normalize_p_disjunction(
  proc: &Disjunction,
  input: ProcVisitInputs,
  env: &HashMap<String, Par>
) -> Result<ProcVisitOutputs, InterpreterError> {

  let left_result = normalize_match_proc(
    &proc.left,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: input.free_map.clone(),
    },
    env)?;

  let mut right_result = normalize_match_proc(
    &proc.right,
    ProcVisitInputs {
      par: Par::default(),
      bound_map_chain: input.bound_map_chain.clone(),
      free_map: left_result.free_map.clone(),
    },
    env)?;

  let lp = left_result.par;
  let result_connective = match lp.single_connective() {
    Some(Connective { connective_instance: Some(ConnectiveInstance::ConnOrBody(conn_body)), }) =>
      Connective {
        connective_instance: Some(ConnectiveInstance::ConnOrBody(ConnectiveBody {
          ps: {
            let mut ps = conn_body.ps.clone();
            ps.push(right_result.par);
            ps
          },
        })),
      },
    _ => Connective {
      connective_instance: Some(ConnectiveInstance::ConnOrBody(ConnectiveBody {
        ps: vec![lp, right_result.par],
      })),
    },
  };

  let result_par = prepend_connective(input.par, result_connective.clone(), input.bound_map_chain.depth() as i32);

  let updated_free_map = right_result
    .free_map
    .add_connective(result_connective.connective_instance.unwrap(), SourcePosition {
      row: proc.line_num,
      column: proc.col_num,
    });

  Ok(ProcVisitOutputs {
    par: result_par,
    free_map: updated_free_map,
  })
}

// #[test]
// fn test_normalize_disjunction() {
//   let rholang_code = r#"{ "2" \/ Nil}"#;
//   let tree = parse_rholang_code(rholang_code);
//   let root_node = tree.root_node();
//   println!("Tree S-expression: {}", root_node.to_sexp());
//   println!("Root node kind: {}", root_node.kind());
//
//   let block_node = root_node.child(0).expect("Expected a block node");
//   println!("Found block node: {}", block_node.to_sexp());
//
//   let disjunction_node = block_node.child_by_field_name("body").expect("Expected a conjunction node");
//   println!("Found disjunction node: {}", disjunction_node.to_sexp());
//
//   let input = ProcVisitInputs {
//     par: Par::default(),
//     bound_map_chain: Default::default(),
//     free_map: Default::default(),
//   };
//
//   match normalize_match(disjunction_node, input, rholang_code.as_bytes()) {
//     Ok(result) => {
//       println!("Normalization successful!");
//
//       assert_eq!(result.par.connectives.len(), 1);
//       if let Some(ConnectiveInstance::ConnOrBody(conn_body)) = &result.par.connectives[0].connective_instance {
//         assert_eq!(conn_body.ps.len(), 2);
//         if let Some(ExprInstance::GString(value)) = conn_body.ps[0].exprs.get(0).and_then(|e| e.expr_instance.clone()) {
//           assert_eq!(value, "2");
//         } else {
//           panic!("Left operand is not GInt(1)");
//         }
//         assert!(conn_body.ps[1].is_empty(), "Right operand is not Nil");
//       } else {
//         panic!("Expected ConnOrBody in connectives");
//       }
//     }
//     Err(e) => {
//       println!("Normalization failed: {}", e);
//       panic!("Test failed due to normalization error");
//     }
//   }
// }