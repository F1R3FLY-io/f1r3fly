use std::fmt;

use rholang::rust::interpreter::errors::InterpreterError;
use shared::rust::store::key_value_store::KvStoreError;

#[derive(Debug)]
pub enum CasperError {
    InterpreterError(InterpreterError),
    KvStoreError(KvStoreError),
}

impl fmt::Display for CasperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CasperError::InterpreterError(error) => write!(f, "Interpreter error: {}", error),
            CasperError::KvStoreError(error) => write!(f, "KvStore error: {}", error),
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
