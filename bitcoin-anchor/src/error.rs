use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnchorError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Commitment creation error: {0}")]
    Commitment(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type AnchorResult<T> = Result<T, AnchorError>; 