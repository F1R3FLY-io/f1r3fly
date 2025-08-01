use thiserror::Error;

/// Errors that can occur during F1r3fly Bitcoin anchoring operations
#[derive(Error, Debug)]
pub enum AnchorError {
    /// Bitcoin transaction construction errors
    #[error("Bitcoin transaction error: {0}")]
    Transaction(String),

    /// State commitment creation errors
    #[error("State commitment error: {0}")]
    Commitment(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// PSBT construction and manipulation errors
    #[error("PSBT error: {0}")]
    Psbt(String),

    /// Network/communication errors
    #[error("Network error: {0}")]
    Network(String),

    /// Insufficient funds for transaction
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    /// Invalid data format
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Address validation errors
    #[error("Invalid address: {address} ({reason})")]
    InvalidAddress { address: String, reason: String },

    /// Amount validation errors
    #[error("Invalid amount: {amount} satoshis ({reason})")]
    InvalidAmount { amount: u64, reason: String },

    /// UTXO availability errors
    #[error("UTXO error: {message}")]
    UtxoError { message: String },

    /// Transaction broadcast errors
    #[error("Broadcast error: {0}")]
    Broadcast(String),
}

impl From<bitcoin::address::ParseError> for AnchorError {
    fn from(e: bitcoin::address::ParseError) -> Self {
        AnchorError::InvalidData(format!("Bitcoin address parse error: {}", e))
    }
}

impl From<bitcoin::amount::ParseAmountError> for AnchorError {
    fn from(e: bitcoin::amount::ParseAmountError) -> Self {
        AnchorError::InvalidData(format!("Amount parse error: {}", e))
    }
}

impl From<serde_json::Error> for AnchorError {
    fn from(e: serde_json::Error) -> Self {
        AnchorError::Serialization(format!("JSON error: {}", e))
    }
}

impl From<reqwest::Error> for AnchorError {
    fn from(e: reqwest::Error) -> Self {
        AnchorError::Network(format!("HTTP request error: {}", e))
    }
}

impl From<hex::FromHexError> for AnchorError {
    fn from(e: hex::FromHexError) -> Self {
        AnchorError::InvalidData(format!("Hex decode error: {}", e))
    }
}

/// Result type for F1r3fly Bitcoin anchoring operations
pub type AnchorResult<T> = Result<T, AnchorError>;

/// Error context information for debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub details: Vec<(String, String)>,
}

impl ErrorContext {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            details: Vec::new(),
        }
    }

    pub fn add_detail(mut self, key: &str, value: &str) -> Self {
        self.details.push((key.to_string(), value.to_string()));
        self
    }
}

/// Recovery strategy for errors
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry,
    /// Skip and continue
    Skip,
    /// Manual intervention required
    Manual,
}
