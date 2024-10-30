use crate::rust::interpreter::compiler::exports::{FreeContext, FreeMap, SourcePosition};
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::Var;
use models::rhoapi::var::VarInstance::{FreeVar, Wildcard};
use models::rhoapi::var::WildcardMsg;
use crate::rust::interpreter::compiler::normalize::VarSort;
use tree_sitter::Node;
use std::error::Error;
use crate::rust::interpreter::compiler::rholang_ast::Names;

pub fn handle_proc_var(
  node: Node,
  mut known_free: FreeMap<VarSort>,
  source_code: &[u8],
) -> Result<(Option<Var>, FreeMap<VarSort>), Box<dyn Error>> {
  match node.kind() {
    "wildcard" => {
      let wildcard_var = Var {
        var_instance: Some(Wildcard(WildcardMsg {})),
      };
      let source_position = SourcePosition::new(
        node.start_position().row,
        node.start_position().column,
      );

      known_free.add_wildcard(source_position);
      Ok((Some(wildcard_var), known_free))
    }
    "var" => {
      let var_name = node.utf8_text(source_code)?;
      let source_position = SourcePosition::new(
        node.start_position().row,
        node.start_position().column,
      );

      match known_free.get(var_name) {
        None => {
          let binding = (var_name.to_string(), VarSort::ProcSort, source_position);
          let new_bindings_pair = known_free.put(binding);
          let free_var = Var {
            var_instance: Some(FreeVar(new_bindings_pair.next_level as i32)),
          };
          Ok((Some(free_var), new_bindings_pair))
        }
        Some(FreeContext {
               source_position: first_source_position,
               ..
             }) => Err(InterpreterError::UnexpectedReuseOfProcContextFree {
          var_name: var_name.to_string(),
          first_use: first_source_position,
          second_use: source_position,
        }
          .into()),
      }
    }
    _ => Err(InterpreterError::NormalizerError("Unexpected node kind for Remainder".to_string())
      .into())
  }
}

// coop.rchain.rholang.interpreter.compiler.normalizer.RemainderNormalizeMatcher.normalizeMatchProc
pub fn normalize_remainder(
  list_node: Node,
  known_free: FreeMap<VarSort>,
  source_code: &[u8],
) -> Result<(Option<Var>, FreeMap<VarSort>), Box<dyn Error>> {
  if let Some(remainder) = list_node.child_by_field_name("cont") {
    match remainder.kind() {
      "var" => {
        handle_proc_var(remainder, known_free, source_code)
      }
      _ => Err(InterpreterError::NormalizerError("Unexpected node kind for proc Remainder".to_string())
        .into())
    }
  } else {
    Ok((None, known_free))
  }
}

pub fn normalize_match_name(
  formals: &Names,
  known_free: FreeMap<VarSort>
) -> Result<(Option<Var>, FreeMap<VarSort>), InterpreterError> {
  todo!()
  // if let Some(remainder) = formals.clone().cont {
  //   //I'm not sure here, but based on grammar.js names contains 'cont' field which can provide
  //   //optional remainder '...' or "@" (quote) or _proc_var (wildcard and var)
  //   match remainder.kind() {
  //     "'...'" | "quote" | "wildcard" | "var"=> {
  //       handle_proc_var(remainder, known_free, source_code)
  //     }
  //     _ => Err(InterpreterError::NormalizerError("Unexpected node kind for name Remainder".to_string())
  //       .into())
  //   }
  // } else {
  //   Ok((None, known_free))
  // }
}