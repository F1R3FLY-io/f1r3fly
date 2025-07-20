use crate::error::{AnchorError, AnchorResult, RecoveryStrategy};
use std::time::Duration;

/// Recovery service for handling Bitcoin anchoring failures
pub struct RecoveryService {
    max_retry_attempts: u32,
    retry_base_delay: Duration,
}

impl RecoveryService {
    /// Create a new recovery service with default settings
    pub fn new() -> Self {
        Self {
            max_retry_attempts: 3,
            retry_base_delay: Duration::from_secs(2),
        }
    }

    /// Create a recovery service with custom retry settings
    pub fn with_retry_config(max_attempts: u32, base_delay: Duration) -> Self {
        Self {
            max_retry_attempts: max_attempts,
            retry_base_delay: base_delay,
        }
    }

    /// Execute an operation with automatic recovery
    
    pub async fn execute_with_recovery<F, T, Fut>(&self, operation: F, operation_name: &str) -> AnchorResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = AnchorResult<T>>,
    {
        let mut last_error = None;
        
        for attempt in 1..=self.max_retry_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let recovery_strategy = error.recovery_strategy();
                    
                    match recovery_strategy {
                        RecoveryStrategy::Retry { delay, max_attempts } => {
                            if attempt < max_attempts.min(self.max_retry_attempts) {
                                #[cfg(debug_assertions)]
                                eprintln!(
                                    "Attempt {} failed for {}: {}. Retrying in {:?}",
                                    attempt, operation_name, error, delay
                                );
                                
                                
                                tokio::time::sleep(delay).await;
                                last_error = Some(error);
                                continue;
                            }
                        }
                        RecoveryStrategy::NoRecovery => {
                            return Err(error);
                        }
                        _ => {
                            // For other strategies, return the error with recovery suggestion
                            return Err(error);
                        }
                    }
                    
                    last_error = Some(error);
                }
            }
        }
        
        // If we exhausted all retries, return the last error
        Err(last_error.unwrap_or_else(|| AnchorError::Network("Unknown error after retries".to_string())))
    }

    /// Generate a detailed error report with recovery suggestions
    pub fn generate_error_report(&self, error: &AnchorError) -> ErrorReport {
        let context = error.context();
        let recovery_strategy = error.recovery_strategy();
        
        let detailed_suggestion = match &recovery_strategy {
            RecoveryStrategy::Retry { delay, max_attempts } => {
                format!(
                    "This is a temporary error that can be retried. Wait {:?} and try again. Maximum {} attempts recommended.",
                    delay, max_attempts
                )
            }
            RecoveryStrategy::IncreaseFee { suggested_fee_rate } => {
                format!(
                    "The transaction fee is too low for current network conditions. Increase the fee rate to {:.2} sat/vB or higher.",
                    suggested_fee_rate
                )
            }
            RecoveryStrategy::AlternativeUtxos => {
                "Try using different UTXOs or wait for pending transactions to confirm.".to_string()
            }
            RecoveryStrategy::FallbackProvider => {
                "The primary blockchain provider is unavailable. Trying alternative data sources.".to_string()
            }
            RecoveryStrategy::ManualIntervention { suggestion } => {
                format!("Manual action required: {}", suggestion)
            }
            RecoveryStrategy::NoRecovery => {
                "This error cannot be automatically recovered. Please review the error details and take manual action.".to_string()
            }
        };

        ErrorReport {
            error_code: context.error_code,
            user_message: context.user_message,
            detailed_suggestion,
            retryable: context.retryable,
            debug_info: context.debug_info,
            recovery_actions: self.suggest_recovery_actions(error),
        }
    }

    /// Suggest specific recovery actions based on error type
    fn suggest_recovery_actions(&self, error: &AnchorError) -> Vec<RecoveryAction> {
        match error {
            AnchorError::InsufficientFunds(_) => vec![
                RecoveryAction {
                    action: "Add funds to source address".to_string(),
                    priority: ActionPriority::High,
                    automated: false,
                },
                RecoveryAction {
                    action: "Reduce transaction amount".to_string(),
                    priority: ActionPriority::Medium,
                    automated: false,
                },
                RecoveryAction {
                    action: "Wait for pending transactions to confirm".to_string(),
                    priority: ActionPriority::Low,
                    automated: false,
                },
            ],
            AnchorError::NetworkTimeout { .. } => vec![
                RecoveryAction {
                    action: "Retry the operation".to_string(),
                    priority: ActionPriority::High,
                    automated: true,
                },
                RecoveryAction {
                    action: "Check internet connection".to_string(),
                    priority: ActionPriority::Medium,
                    automated: false,
                },
                RecoveryAction {
                    action: "Try a different blockchain provider".to_string(),
                    priority: ActionPriority::Low,
                    automated: true,
                },
            ],
            AnchorError::FeeEstimation { .. } => vec![
                RecoveryAction {
                    action: "Use fallback fee estimation".to_string(),
                    priority: ActionPriority::High,
                    automated: true,
                },
                RecoveryAction {
                    action: "Set manual fee rate".to_string(),
                    priority: ActionPriority::Medium,
                    automated: false,
                },
            ],
            AnchorError::MempoolCongestion { recommended_fee_rate } => vec![
                RecoveryAction {
                    action: format!("Increase fee rate to {:.2} sat/vB", recommended_fee_rate),
                    priority: ActionPriority::High,
                    automated: false,
                },
                RecoveryAction {
                    action: "Wait for network congestion to decrease".to_string(),
                    priority: ActionPriority::Medium,
                    automated: false,
                },
                RecoveryAction {
                    action: "Use Replace-By-Fee (RBF) if transaction is stuck".to_string(),
                    priority: ActionPriority::Low,
                    automated: false,
                },
            ],
            AnchorError::DustOutput { dust_limit, .. } => vec![
                RecoveryAction {
                    action: format!("Increase output amount to at least {} satoshis", dust_limit),
                    priority: ActionPriority::High,
                    automated: false,
                },
                RecoveryAction {
                    action: "Combine with other outputs to avoid dust".to_string(),
                    priority: ActionPriority::Medium,
                    automated: true,
                },
            ],
            _ => vec![
                RecoveryAction {
                    action: "Review error details and retry".to_string(),
                    priority: ActionPriority::Medium,
                    automated: false,
                },
            ],
        }
    }

    /// Attempt automatic recovery for supported error types
    
    pub async fn attempt_automatic_recovery<F, T, Fut>(
        &self,
        error: &AnchorError,
        retry_operation: F,
    ) -> AnchorResult<Option<T>>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = AnchorResult<T>>,
    {
        let recovery_strategy = error.recovery_strategy();
        
        match recovery_strategy {
            RecoveryStrategy::Retry { delay, max_attempts } => {
                tokio::time::sleep(delay).await;
                
                for attempt in 1..=max_attempts.min(self.max_retry_attempts) {
                    match retry_operation().await {
                        Ok(result) => return Ok(Some(result)),
                        Err(_) if attempt < max_attempts => {
                            let backoff_delay = Duration::from_millis(
                                (delay.as_millis() as f64 * 1.5_f64.powi(attempt as i32)) as u64
                            );
                            tokio::time::sleep(backoff_delay).await;
                        }
                        Err(final_error) => return Err(final_error),
                    }
                }
                
                Ok(None)
            }
            RecoveryStrategy::FallbackProvider => {
                // In a real implementation, this would try alternative providers
                #[cfg(debug_assertions)]
                eprintln!("Attempting fallback provider recovery");
                
                // For now, just retry once
                tokio::time::sleep(Duration::from_secs(1)).await;
                match retry_operation().await {
                    Ok(result) => Ok(Some(result)),
                    Err(_) => Ok(None),
                }
            }
            _ => Ok(None), // No automatic recovery available
        }
    }
}

impl Default for RecoveryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority level for recovery actions
#[derive(Debug, Clone, PartialEq)]
pub enum ActionPriority {
    High,
    Medium,
    Low,
}

/// Specific recovery action recommendation
#[derive(Debug, Clone)]
pub struct RecoveryAction {
    pub action: String,
    pub priority: ActionPriority,
    pub automated: bool, // Whether this action can be performed automatically
}

/// Comprehensive error report with recovery suggestions
#[derive(Debug)]
pub struct ErrorReport {
    pub error_code: String,
    pub user_message: String,
    pub detailed_suggestion: String,
    pub retryable: bool,
    pub debug_info: std::collections::HashMap<String, String>,
    pub recovery_actions: Vec<RecoveryAction>,
}

impl ErrorReport {
    /// Get high-priority manual actions that require user intervention
    pub fn critical_actions(&self) -> Vec<&RecoveryAction> {
        self.recovery_actions
            .iter()
            .filter(|action| action.priority == ActionPriority::High && !action.automated)
            .collect()
    }

    /// Get actions that can be performed automatically
    pub fn automated_actions(&self) -> Vec<&RecoveryAction> {
        self.recovery_actions
            .iter()
            .filter(|action| action.automated)
            .collect()
    }

    /// Format as user-friendly error message
    pub fn format_user_message(&self) -> String {
        let mut message = format!("Error: {}\n\n", self.user_message);
        message.push_str(&format!("Details: {}\n\n", self.detailed_suggestion));
        
        let critical = self.critical_actions();
        if !critical.is_empty() {
            message.push_str("Required Actions:\n");
            for (i, action) in critical.iter().enumerate() {
                message.push_str(&format!("{}. {}\n", i + 1, action.action));
            }
        }
        
        let automated = self.automated_actions();
        if !automated.is_empty() {
            message.push_str("\nAutomatic Recovery Available:\n");
            for action in automated {
                message.push_str(&format!("• {}\n", action.action));
            }
        }
        
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_recovery_service_creation() {
        let service = RecoveryService::new();
        assert_eq!(service.max_retry_attempts, 3);
        
        let custom_service = RecoveryService::with_retry_config(5, Duration::from_secs(1));
        assert_eq!(custom_service.max_retry_attempts, 5);
    }

    #[test]
    fn test_error_report_generation() {
        let service = RecoveryService::new();
        let error = AnchorError::InsufficientFunds("Not enough Bitcoin".to_string());
        
        let report = service.generate_error_report(&error);
        assert_eq!(report.error_code, "INSUFFICIENT_FUNDS");
        assert!(!report.retryable);
        assert!(!report.critical_actions().is_empty());
    }

    #[test]
    fn test_recovery_actions() {
        let service = RecoveryService::new();
        
        // Test network timeout error
        let timeout_error = AnchorError::NetworkTimeout {
            timeout: Duration::from_secs(30),
            message: "Request timed out".to_string(),
        };
        
        let report = service.generate_error_report(&timeout_error);
        assert!(report.retryable);
        assert!(report.automated_actions().len() > 0);
        
        // Test insufficient funds error
        let funds_error = AnchorError::InsufficientFunds("Not enough BTC".to_string());
        let funds_report = service.generate_error_report(&funds_error);
        assert!(!funds_report.retryable);
        assert!(funds_report.critical_actions().len() > 0);
    }

    #[test]
    fn test_error_message_formatting() {
        let service = RecoveryService::new();
        let error = AnchorError::MempoolCongestion { recommended_fee_rate: 25.0 };
        
        let report = service.generate_error_report(&error);
        let formatted = report.format_user_message();
        
        assert!(formatted.contains("Error:"));
        assert!(formatted.contains("Details:"));
        assert!(formatted.contains("25.00 sat/vB")); // Format is {:.2}, so it's 25.00
    }
} 