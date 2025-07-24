use thiserror::Error;
use std::time::Duration;

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

    /// Network/communication errors (retryable)
    #[error("Network error: {0}")]
    Network(String),

    /// Insufficient funds for transaction
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),

    /// Invalid data format
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Transaction broadcast errors
    #[error("Broadcast error: {0}")]
    Broadcast(String),

    /// Network timeout errors (retryable)
    #[error("Network timeout after {timeout:?}: {message}")]
    NetworkTimeout { timeout: Duration, message: String },

    /// Rate limiting errors (retryable)
    #[error("Rate limited by API provider: {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },

    /// Blockchain API errors with specific error codes
    #[error("Blockchain API error [{code}]: {message}")]
    BlockchainApi { code: u16, message: String },

    /// Fee estimation errors
    #[error("Fee estimation failed: {reason}")]
    FeeEstimation { reason: String },

    /// UTXO availability errors
    #[error("UTXO error: {message}")]
    UtxoError { message: String },

    /// Transaction validation errors
    #[error("Transaction validation failed: {reason}")]
    ValidationError { reason: String },

    /// Address validation errors
    #[error("Invalid address: {address} ({reason})")]
    InvalidAddress { address: String, reason: String },

    /// Amount validation errors
    #[error("Invalid amount: {amount} satoshis ({reason})")]
    InvalidAmount { amount: u64, reason: String },

    /// Dust output errors
    #[error("Output would be dust: {amount} satoshis (minimum: {dust_limit})")]
    DustOutput { amount: u64, dust_limit: u64 },

    /// Transaction too large errors
    #[error("Transaction too large: {size} bytes (maximum: {max_size})")]
    TransactionTooLarge { size: usize, max_size: usize },

    /// Confirmation timeout errors
    #[error("Transaction confirmation timeout: waited {duration:?} for {confirmations} confirmations")]
    ConfirmationTimeout {
        duration: Duration,
        confirmations: u32,
    },

    /// Double spend detection
    #[error("Potential double spend detected for transaction {txid}")]
    DoubleSpend { txid: String },

    /// Mempool congestion
    #[error("Mempool congestion: recommended fee rate {recommended_fee_rate} sat/vB")]
    MempoolCongestion { recommended_fee_rate: f64 },
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation after a delay
    Retry { delay: Duration, max_attempts: u32 },
    /// Increase fee and retry
    IncreaseFee { suggested_fee_rate: f64 },
    /// Use different UTXO selection
    AlternativeUtxos,
    /// Fallback to different API provider
    FallbackProvider,
    /// Manual intervention required
    ManualIntervention { suggestion: String },
    /// Operation cannot be recovered
    NoRecovery,
}

/// Enhanced error context for debugging and recovery
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error code for programmatic handling
    pub error_code: String,
    /// Whether this error can be retried
    pub retryable: bool,
    /// Suggested recovery strategy
    pub recovery_strategy: RecoveryStrategy,
    /// Additional context for debugging
    pub debug_info: std::collections::HashMap<String, String>,
    /// User-friendly error message
    pub user_message: String,
}

impl AnchorError {
    /// Get error context for this error
    pub fn context(&self) -> ErrorContext {
        match self {
            AnchorError::NetworkTimeout { timeout, message } => ErrorContext {
                error_code: "NETWORK_TIMEOUT".to_string(),
                retryable: true,
                recovery_strategy: RecoveryStrategy::Retry {
                    delay: Duration::from_secs(5),
                    max_attempts: 3,
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("timeout_duration".to_string(), format!("{:?}", timeout));
                    info.insert("original_message".to_string(), message.clone());
                    info
                },
                user_message: "Network request timed out. The operation will be retried automatically.".to_string(),
            },
            
            AnchorError::RateLimited { retry_after } => ErrorContext {
                error_code: "RATE_LIMITED".to_string(),
                retryable: true,
                recovery_strategy: RecoveryStrategy::Retry {
                    delay: retry_after.unwrap_or(Duration::from_secs(60)),
                    max_attempts: 2,
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    if let Some(retry) = retry_after {
                        info.insert("retry_after".to_string(), format!("{:?}", retry));
                    }
                    info
                },
                user_message: "API rate limit exceeded. Please wait before retrying.".to_string(),
            },

            AnchorError::InsufficientFunds(msg) => ErrorContext {
                error_code: "INSUFFICIENT_FUNDS".to_string(),
                retryable: false,
                recovery_strategy: RecoveryStrategy::ManualIntervention {
                    suggestion: "Add more Bitcoin to the source address or reduce the transaction amount".to_string(),
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("details".to_string(), msg.clone());
                    info
                },
                user_message: "Insufficient Bitcoin balance to complete the transaction.".to_string(),
            },

            AnchorError::Broadcast(msg) => ErrorContext {
                error_code: "BROADCAST_FAILED".to_string(),
                retryable: true,
                recovery_strategy: RecoveryStrategy::Retry {
                    delay: Duration::from_secs(30),
                    max_attempts: 3,
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("error_message".to_string(), msg.clone());
                    info
                },
                user_message: "Failed to broadcast transaction to Bitcoin network. This may be temporary.".to_string(),
            },

            AnchorError::FeeEstimation { reason } => ErrorContext {
                error_code: "FEE_ESTIMATION_FAILED".to_string(),
                retryable: true,
                recovery_strategy: RecoveryStrategy::FallbackProvider,
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("reason".to_string(), reason.clone());
                    info
                },
                user_message: "Unable to estimate network fees. Using fallback fee estimation.".to_string(),
            },

            AnchorError::DustOutput { amount, dust_limit } => ErrorContext {
                error_code: "DUST_OUTPUT".to_string(),
                retryable: false,
                recovery_strategy: RecoveryStrategy::ManualIntervention {
                    suggestion: format!("Increase output amount to at least {} satoshis", dust_limit),
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("attempted_amount".to_string(), amount.to_string());
                    info.insert("dust_limit".to_string(), dust_limit.to_string());
                    info
                },
                user_message: "Transaction output too small. Bitcoin network rejects very small outputs.".to_string(),
            },

            AnchorError::MempoolCongestion { recommended_fee_rate } => ErrorContext {
                error_code: "MEMPOOL_CONGESTION".to_string(),
                retryable: true,
                recovery_strategy: RecoveryStrategy::IncreaseFee {
                    suggested_fee_rate: *recommended_fee_rate,
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("recommended_fee_rate".to_string(), recommended_fee_rate.to_string());
                    info
                },
                user_message: "Bitcoin network is congested. Consider increasing the fee rate for faster confirmation.".to_string(),
            },

            AnchorError::ConfirmationTimeout { duration, confirmations } => ErrorContext {
                error_code: "CONFIRMATION_TIMEOUT".to_string(),
                retryable: false,
                recovery_strategy: RecoveryStrategy::IncreaseFee {
                    suggested_fee_rate: 20.0, // Default higher fee rate
                },
                debug_info: {
                    let mut info = std::collections::HashMap::new();
                    info.insert("waited_duration".to_string(), format!("{:?}", duration));
                    info.insert("target_confirmations".to_string(), confirmations.to_string());
                    info
                },
                user_message: "Transaction is taking longer than expected to confirm. You may want to increase the fee.".to_string(),
            },

            _ => ErrorContext {
                error_code: "GENERAL_ERROR".to_string(),
                retryable: false,
                recovery_strategy: RecoveryStrategy::NoRecovery,
                debug_info: std::collections::HashMap::new(),
                user_message: "An error occurred during the Bitcoin anchoring operation.".to_string(),
            },
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        self.context().retryable
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        self.context().user_message
    }

    /// Get suggested recovery strategy
    pub fn recovery_strategy(&self) -> RecoveryStrategy {
        self.context().recovery_strategy
    }
}

/// Type alias for Results with AnchorError
pub type AnchorResult<T> = Result<T, AnchorError>; 