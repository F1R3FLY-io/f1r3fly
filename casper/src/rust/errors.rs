use std::fmt;

use rholang::rust::interpreter::errors::InterpreterError;
use shared::rust::store::key_value_store::KvStoreError;

use super::util::rholang::system_deploy_user_error::SystemDeployPlatformFailure;

#[derive(Debug)]
pub enum CasperError {
    InterpreterError(InterpreterError),
    KvStoreError(KvStoreError),
    RuntimeError(String),
    SystemRuntimeError(SystemDeployPlatformFailure),
    SigningError(String),
}

impl fmt::Display for CasperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CasperError::InterpreterError(error) => write!(f, "Interpreter error: {}", error),
            CasperError::KvStoreError(error) => write!(f, "KvStore error: {}", error),
            CasperError::RuntimeError(error) => write!(f, "Runtime error: {}", error),
            CasperError::SystemRuntimeError(error) => write!(f, "System runtime error: {}", error),
            CasperError::SigningError(error) => write!(f, "Signing error: {}", error),
        }
    }
}

impl From<InterpreterError> for CasperError {
    fn from(error: InterpreterError) -> Self {
        CasperError::InterpreterError(error)
    }
}

impl From<KvStoreError> for CasperError {
    fn from(error: KvStoreError) -> Self {
        CasperError::KvStoreError(error)
    }
}
