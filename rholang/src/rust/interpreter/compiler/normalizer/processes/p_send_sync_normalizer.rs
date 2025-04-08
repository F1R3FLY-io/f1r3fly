use crate::rust::interpreter::compiler::exports::BoundMapChain;
use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
use crate::rust::interpreter::compiler::rholang_ast::{Id, NameDecl};
use crate::rust::interpreter::compiler::rholang_ast::{
    LinearBind, Names, Receipt, SendType, Source, NAME_WILD, NIL,
};
use crate::rust::interpreter::compiler::rholang_ast::{Name, Proc, SyncSendCont};
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::normal_forms::Par;
use std::collections::HashMap;
use uuid::Uuid;

use super::exports::{AnnProc, FreeMap, SourcePosition};

pub fn normalize_p_send_sync(
    name: Name,
    messages: &[AnnProc],
    cont: SyncSendCont,
    input_par: &mut Par,
    free_map: &mut FreeMap,
    bound_map_chain: &mut BoundMapChain,
    env: &HashMap<String, Par>,
    pos: SourcePosition,
) -> Result<(), InterpreterError> {
    let random = Uuid::new_v4().to_string();
    let identifier = Id {
        name: &random,
        pos: SourcePosition::default(),
    };
    let name_var = identifier.as_name();
    let eval_name_var = Proc::Eval {
        name: name_var.annotated_dummy(),
    };

    let mut inputs = Vec::with_capacity(messages.len() + 1);
    inputs.push(eval_name_var.annotate_dummy());
    inputs.extend(messages);
    let send = Proc::Send {
        name,
        send_type: SendType::Single,
        inputs,
    };

    let list_receipt = Receipt::Linear(vec![LinearBind {
        lhs: Names::single(NAME_WILD.annotated_dummy()),
        rhs: Source::Simple {
            name: name_var.annotated_dummy(),
        },
    }]);
    let receive = Proc::ForComprehension {
        receipts: vec![list_receipt],
        proc: match cont {
            SyncSendCont::Empty => NIL.annotate_dummy(),
            SyncSendCont::NonEmpty(proc) => proc,
        },
    };

    let p_proc = Proc::Par {
        left: send.annotate(pos),
        right: receive.annotate(pos),
    };
    let p_send_sync = Proc::New {
        decls: vec![NameDecl {
            id: identifier,
            uri: None,
        }],
        proc: &p_proc,
    };
    normalize_match_proc(&p_send_sync, input_par, free_map, bound_map_chain, env, pos)
}

// FIXME does this test even make sense? Any implentation that does not return err would pass it.
// #[cfg(test)]
// mod tests {

//     use super::*;
//     use crate::rust::interpreter::compiler::bound_map_chain::BoundMapChain;
//     use crate::rust::interpreter::compiler::exports::FreeMap;
//     use crate::rust::interpreter::compiler::normalize::{ProcVisitInputs, VarSort};
//     use crate::rust::interpreter::compiler::rholang_ast;
//     use crate::rust::interpreter::compiler::rholang_ast::Proc;
//     use models::rhoapi::Par;

//     fn p_send_sync() -> Proc {
//         let p_send_sync = Proc::SendSync {
//             name: rholang_ast::Name::ProcVar(Box::new(rholang_ast::Proc::Wildcard {
//                 line_num: 0,
//                 col_num: 0,
//             })),
//             messages: rholang_ast::ProcList {
//                 procs: vec![],
//                 line_num: 1,
//                 col_num: 1,
//             },
//             cont: rholang_ast::SyncSendCont::Empty {
//                 line_num: 2,
//                 col_num: 2,
//             },
//             line_num: 3,
//             col_num: 3,
//         };

//         p_send_sync
//     }

//     #[test]
//     fn test_normalize_p_send_sync() {
//         let p = p_send_sync();
//         fn inputs() -> ProcVisitInputs {
//             ProcVisitInputs {
//                 par: Par::default(),
//                 bound_map_chain: BoundMapChain::new(),
//                 free_map: FreeMap::<VarSort>::new(),
//             }
//         }

//         let env = HashMap::<String, Par>::new();

//         let result = match p {
//             Proc::SendSync {
//                 name,
//                 messages,
//                 cont,
//                 line_num,
//                 col_num,
//             } => normalize_p_send_sync(&name, &messages, &cont, line_num, col_num, inputs(), &env),
//             _ => Result::Err(InterpreterError::NormalizerError(
//                 "Expected Proc::SendSync".to_string(),
//             )),
//         };

//         assert!(result.is_ok());

//         // check the result
//         // let result = result.unwrap();
//         // let par = result.par;
//         // assert_eq!(par.sends.len(), 1);
//         // assert_eq!(par.receives.len(), 1);
//     }
// }
