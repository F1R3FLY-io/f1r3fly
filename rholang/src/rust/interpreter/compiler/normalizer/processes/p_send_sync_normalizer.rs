use super::exports::*;
use crate::rust::interpreter::compiler::normalize::{normalize_match_proc, ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::rholang_ast;
use crate::rust::interpreter::compiler::rholang_ast::{Name, Proc, SyncSendCont};
use crate::rust::interpreter::compiler::rholang_ast::{NameDecl, ProcList};
use crate::rust::interpreter::errors::InterpreterError;
use uuid::Uuid;

pub fn normalize_p_send_sync(
  name: &Name,
  messages: &ProcList,
  cont: &SyncSendCont,
  line_num: usize,
  col_num: usize,
  input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {

  let identifier = Uuid::new_v4().to_string();
  let name_var: rholang_ast::Proc = Proc::Eval(rholang_ast::Eval {
    name: rholang_ast::Name::ProcVar(Box::new(rholang_ast::Proc::Var(rholang_ast::Var {
      name: identifier.clone(),
      line_num,
      col_num,
    })),),
    line_num,
    col_num,
  });


  let send: Proc = {
    let mut listproc =
      messages.procs.clone();

    listproc.insert(0, name_var.clone());

    Proc::Send {
      name: name.clone(),
      send_type: rholang_ast::SendType::Single {
        line_num,
        col_num,
      },
      inputs: ProcList {
        procs: listproc,
        line_num,
        col_num,
      },
      line_num: messages.line_num,
      col_num: messages.col_num,
    }
  };

  let receive: Proc = {
    let list_name = rholang_ast::Names {
      names: vec![rholang_ast::Name::ProcVar(Box::new(rholang_ast::Proc::Wildcard {
        line_num,
        col_num,
      }))],
      cont: None,
      line_num,
      col_num,
    };

    let linear_bind_impl: rholang_ast::LinearBind = rholang_ast::LinearBind {
      names: list_name,
      input: rholang_ast::Source::Simple {
        name: rholang_ast::Name::ProcVar(Box::new(name_var)),
        line_num,
        col_num,
      },
      line_num,
      col_num,
    };
    let list_linear_bind = vec![rholang_ast::Receipt::LinearBinds(linear_bind_impl)];

    let list_receipt = rholang_ast::Receipts {
      receipts: list_linear_bind,
      line_num,
      col_num,
    };

    let proc: Box<rholang_ast::Block> =
      match cont {
        rholang_ast::SyncSendCont::Empty {
          line_num,
          col_num
        } => {
          Box::new(rholang_ast::Block {
            proc: Proc::Nil {
              line_num: *line_num,
              col_num: *col_num,
            },
            line_num: *line_num,
            col_num: *col_num,
          })
        }

        rholang_ast::SyncSendCont::NonEmpty {
          proc,
          line_num,
          col_num
        } => {
          Box::new(rholang_ast::Block {
            proc: *(proc).clone(),
            line_num: *line_num,
            col_num: *col_num,
          })
        }
      };

    Proc::Input {
      formals: list_receipt,
      proc,
      line_num,
      col_num,
    }
  };

  let list_name: Vec<NameDecl> = vec![
    NameDecl {
      var: rholang_ast::Var {
        name: identifier,
        line_num,
        col_num,
      },
      uri: None, // TODO: fix?
      line_num,
      col_num,
    }
  ];

  let decls: rholang_ast::Decls = rholang_ast::Decls {
    decls: list_name,
    line_num,
    col_num,
  };

  let p_par = Proc::Par {
    left: Box::new(send),
    right: Box::new(receive),
    line_num,
    col_num,
  };

  let p_new: Proc = Proc::New {
    decls,
    proc: Box::new(p_par),
    line_num,
    col_num,
  };

  normalize_match_proc(&p_new, input).map_err(|e| e.into())

}


#[cfg(test)]
mod tests {

  use super::*;
  use models::rhoapi::Par;
  use crate::rust::interpreter::compiler::bound_map_chain::BoundMapChain;
  use crate::rust::interpreter::compiler::normalize::{ProcVisitInputs, VarSort};
  use crate::rust::interpreter::compiler::rholang_ast;
  use crate::rust::interpreter::compiler::rholang_ast::Proc;

  fn p_send_sync() -> Proc {
    let p_send_sync = Proc::SendSync {
      name: rholang_ast::Name::ProcVar(Box::new(rholang_ast::Proc::Wildcard {
        line_num: 0,
        col_num: 0,
      })),
      messages: rholang_ast::ProcList {
        procs: vec![],
        line_num: 1,
        col_num: 1,
      },
      cont: rholang_ast::SyncSendCont::Empty {
        line_num: 2,
        col_num: 2,
      },
      line_num: 3,
      col_num: 3,
    };

    p_send_sync
  }

  #[test]
  fn test_normalize_p_send_sync() {
    let p = p_send_sync();
    fn inputs() -> ProcVisitInputs {
      ProcVisitInputs {
        par: Par::default(),
        bound_map_chain: BoundMapChain::new(),
        free_map: FreeMap::<VarSort>::new(),
      }
    }

    let result = match p {
      Proc::SendSync {
        name,
        messages,
        cont,
        line_num,
        col_num,
      } => normalize_p_send_sync(&name, &messages, &cont, line_num, col_num, inputs()),
      _ => Result::Err(InterpreterError::NormalizerError("Expected Proc::SendSync".to_string())),
    };

    assert!(result.is_ok());

    // check the result
    let result = result.unwrap();
    let par = result.par;
    // review assertions when p_new is implemented
    assert_eq!(par.sends.len(), 1);
    assert_eq!(par.receives.len(), 1);
  }
}
