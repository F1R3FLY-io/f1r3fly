use errors::InterpreterError;

pub mod accounting;
pub mod compiler;
pub mod contract_call;
pub mod dispatch;
pub mod env;
pub mod errors;
pub mod interpreter;
pub mod matcher;
pub mod pretty_printer;
pub mod reduce;
pub mod registry;
pub mod rho_runtime;
pub mod rho_type;
pub mod storage;
pub mod substitute;
pub mod system_processes;
pub mod test_utils;
pub mod util;
pub mod openai_service;
pub mod merging;

pub fn unwrap_option_safe<A: Clone + std::fmt::Debug>(
    opt: Option<A>,
) -> Result<A, InterpreterError> {
    opt.map(|x| x.clone()).ok_or_else(|| {
        InterpreterError::UndefinedRequiredProtobufFieldError(format!(
            "{:?}",
            std::any::type_name::<A>()
        ))
    })
}
