use crate::rust::interpreter::compiler::exports::{FreeContext, FreeMap, SourcePosition};
use crate::rust::interpreter::compiler::normalize::VarSort;
use crate::rust::interpreter::errors::InterpreterError;
use f1r3fly_models::rhoapi::var::VarInstance::{FreeVar, Wildcard};
use f1r3fly_models::rhoapi::var::WildcardMsg;
use f1r3fly_models::rhoapi::Var as ModelsVar;

use super::processes::exports::Proc;
use crate::rust::interpreter::compiler::rholang_ast::Var;

fn handle_proc_var(
    proc: &Proc,
    known_free: FreeMap<VarSort>,
) -> Result<(Option<ModelsVar>, FreeMap<VarSort>), InterpreterError> {
    // println!("\nhit handle_proc_var");
    // println!("\nknown_free: {:?}", known_free);
    match proc {
        Proc::Wildcard { line_num, col_num } => {
            let wildcard_var = ModelsVar {
                var_instance: Some(Wildcard(WildcardMsg {})),
            };
            let source_position = SourcePosition::new(*line_num, *col_num);
            Ok((Some(wildcard_var), known_free.add_wildcard(source_position)))
        }

        Proc::Var(Var {
            name,
            line_num,
            col_num,
        }) => {
            let source_position = SourcePosition::new(*line_num, *col_num);

            match known_free.get(&name) {
                None => {
                    let binding = (name.clone(), VarSort::ProcSort, source_position);
                    let new_bindings_pair = known_free.put(binding);
                    let free_var = ModelsVar {
                        var_instance: Some(FreeVar(known_free.next_level as i32)),
                    };
                    Ok((Some(free_var), new_bindings_pair))
                }
                Some(FreeContext {
                    source_position: first_source_position,
                    ..
                }) => Err(InterpreterError::UnexpectedReuseOfProcContextFree {
                    var_name: name.clone(),
                    first_use: first_source_position,
                    second_use: source_position,
                }),
            }
        }

        _ => Err(InterpreterError::NormalizerError(format!(
            "Expected Proc::Var or Proc::Wildcard, found {:?}",
            proc,
        ))),
    }
}

// coop.rchain.rholang.interpreter.compiler.normalizer.RemainderNormalizeMatcher.normalizeMatchProc
// This function is to be called in `collection_normalize_matcher`
// This handles the 'cont' field in our grammar.js for 'collection' types. AKA '_proc_remainder'
pub fn normalize_remainder(
    r: &Option<Box<Proc>>,
    known_free: FreeMap<VarSort>,
) -> Result<(Option<ModelsVar>, FreeMap<VarSort>), InterpreterError> {
    match r {
        Some(pr) => handle_proc_var(pr, known_free),
        None => Ok((None, known_free)),
    }
}

// This function handles the 'cont' field in our grammar.js for 'names' types. AKA '_name_remainder'
pub fn normalize_match_name(
    nr: &Option<Box<Proc>>,
    known_free: FreeMap<VarSort>,
) -> Result<(Option<ModelsVar>, FreeMap<VarSort>), InterpreterError> {
    match nr {
        Some(pr) => handle_proc_var(&pr, known_free),
        None => Ok((None, known_free)),
    }
}
