use std::fmt;

use rholang::rust::interpreter::errors::InterpreterError;
use shared::rust::store::key_value_store::KvStoreError;

use super::util::rholang::{
    replay_failure::ReplayFailure, system_deploy_user_error::SystemDeployPlatformFailure,
};

#[derive(Debug, Clone)]
pub enum CasperError {
    InterpreterError(InterpreterError),
    KvStoreError(KvStoreError),
    RuntimeError(String),
    SystemRuntimeError(SystemDeployPlatformFailure),
    SigningError(String),
    ReplayFailure(ReplayFailure),
    Other(String),
}

impl fmt::Display for CasperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CasperError::InterpreterError(error) => write!(f, "Interpreter error: {}", error),
            CasperError::KvStoreError(error) => write!(f, "KvStore error: {}", error),
            CasperError::RuntimeError(error) => write!(f, "Runtime error: {}", error),
            CasperError::SystemRuntimeError(error) => write!(f, "System runtime error: {}", error),
            CasperError::SigningError(error) => write!(f, "Signing error: {}", error),
            CasperError::ReplayFailure(error) => write!(f, "Replay failure: {}", error),
            CasperError::Other(error) => write!(f, "Other error: {}", error),
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

impl From<ReplayFailure> for CasperError {
    fn from(error: ReplayFailure) -> Self {
        CasperError::ReplayFailure(error)
    }
}
