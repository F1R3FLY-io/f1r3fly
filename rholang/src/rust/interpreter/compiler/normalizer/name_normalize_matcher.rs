
use models::rhoapi::{EVar, Expr, expr, Par, Var as model_var, var};
use crate::rust::interpreter::compiler::bound_context::BoundContext;
use crate::rust::interpreter::compiler::exports::{BoundMapChain, FreeContext, FreeMap};
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, VarSort};
use crate::rust::interpreter::compiler::rholang_ast::{Name, Proc, Quote, Var};
use crate::rust::interpreter::compiler::source_position::SourcePosition;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_expr;
use super::exports::*;

pub fn normalize_name(
  proc: &Name,
  mut input: NameVisitInputs,
) -> Result<NameVisitOutputs, InterpreterError> {
  match proc {
    Name::ProcVar(Proc::Wildcard {line_num, col_num}) => {
      let wildcard_bind_result = input.free_map.add_wildcard(SourcePosition {
        row: *line_num,
        column: *col_num,
      });

      let new_expr = Expr {
        expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
          v: Some(model_var {
            var_instance: Some(var::VarInstance::Wildcard(var::WildcardMsg {})),
          }),
        })),
      };


      Ok(NameVisitOutputs {
        par: prepend_expr(Par::default(), new_expr, input.bound_map_chain.depth() as i32),
        free_map: wildcard_bind_result
      })
    }
    Name::ProcVar(Proc::Var(Var{ ref name, line_num, col_num })) => {
      match input.bound_map_chain.get(name) {
        Some(bound_context) =>
          match bound_context {
            BoundContext { index: level, typ: VarSort::NameSort, .. } => {
              let new_expr = Expr {
                expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
                  v: Some(model_var {
                    var_instance: Some(var::VarInstance::BoundVar(level as i32)),
                  }),
                })),
              };

              return Ok(NameVisitOutputs {
                par: prepend_expr(Par::default(), new_expr, input.bound_map_chain.depth() as i32),
                free_map: input.free_map.clone(),
              });
            }
            BoundContext { typ: VarSort::ProcSort, source_position, .. } => {
              return Err(InterpreterError::UnexpectedNameContext {
                var_name: name.to_string(),
                proc_var_source_position: source_position.to_string(),
                name_source_position: SourcePosition {
                  row: *line_num,
                  column: *col_num,
                }.to_string(),
              }.into());
            }
          }
        None =>  {
          match input.free_map.get(name) {
            None => {
              let updated_free_map = input.free_map.put((
                name.to_string(),
                VarSort::NameSort,
                SourcePosition {
                  row: *line_num,
                  column: *col_num,
                },
              ));
              let new_expr = Expr {
                expr_instance: Some(expr::ExprInstance::EVarBody(EVar {
                  v: Some(model_var {
                    var_instance: Some(var::VarInstance::FreeVar(input.free_map.next_level as i32)),
                  }),
                })),
              };

              Ok(NameVisitOutputs {
                par: prepend_expr(Par::default(), new_expr, input.bound_map_chain.depth() as i32),
                free_map: updated_free_map,
              })
            }
            Some(FreeContext { source_position, .. }) => {
              Err(InterpreterError::UnexpectedReuseOfNameContextFree {
                var_name: name.to_string(),
                first_use: source_position.to_string(),
                second_use: SourcePosition {
                  row: *line_num,
                  column: *col_num,
                }.to_string(),
              }.into())
            }

          }
        }
      }
    }
    Name::Quote(Quote {ref quotable, line_num,col_num}) => {
      let proc_visit_result = normalize_match_proc(
        &quotable,
        ProcVisitInputs {
          par: Par::default(),
          bound_map_chain: input.bound_map_chain.clone(),
          free_map: input.free_map.clone(),
        })?;

      Ok(NameVisitOutputs {
        par: proc_visit_result.par,
        free_map: proc_visit_result.free_map,
      })
    }
    _ => {
      Err(InterpreterError::SyntaxError("Failed to normalize name value".to_string()).into())
    }
  }
}

// #[test]
// fn test_normalize_name_wildcard() {
//   let rholang_code = r#"_"#;
//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
// }
//
// #[test]
// fn test_normalize_name_quote() {
//   let rholang_code = r#"@Nil"#;
//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//
//   let quote_node = tree.root_node().child(0).unwrap();
//   println!("Quote node: {}", quote_node.to_sexp());
//
//   let input = NameVisitInputs {
//     bound_map_chain: BoundMapChain::default(),
//     free_map: FreeMap::default(),
//   };
//
//   let output = normalize_name(quote_node, input, rholang_code.as_bytes()).unwrap();
//
//   assert_eq!(output.par.exprs.len(), 0);
//   assert!(output.par.bundles.is_empty());
//   println!("Normalized Par: {:?}", output.par);
// }
