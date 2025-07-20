use std::fmt;

use bitcoin::{Address, OutPoint, Txid, Amount};
use serde::{Deserialize, Serialize};
use thiserror::Error;


use reqwest;

/// Errors that can occur when interacting with Esplora API
#[derive(Error, Debug)]
pub enum EsploraError {
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("JSON parsing failed: {0}")]
    Json(String),
    #[error("Address not found")]
    AddressNotFound,
    #[error("Transaction not found: {0}")]
    TransactionNotFound(Txid),
    #[error("Network error: {0}")]
    Network(String),
}

/// UTXO information from Esplora API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsploraUtxo {
    pub txid: Txid,
    pub vout: u32,
    pub value: u64,
    pub status: UtxoStatus,
}

/// Status information for a UTXO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

/// Transaction information from Esplora API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsploraTransaction {
    pub txid: String,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<EsploraInput>,
    pub vout: Vec<EsploraOutput>,
    pub size: u32,
    pub weight: u32,
    pub fee: u64,
    pub status: TransactionStatus,
}

/// Transaction input from Esplora API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsploraInput {
    pub txid: String,
    pub vout: u32,
    pub prevout: Option<EsploraOutput>,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    pub witness: Vec<String>,
    pub is_coinbase: bool,
    pub sequence: u32,
}

/// Transaction output from Esplora API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsploraOutput {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

/// Transaction status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
    pub block_hash: Option<String>,
    pub block_time: Option<u64>,
}

/// Fee rate estimates from Esplora API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEstimates {
    #[serde(rename = "1")]
    pub fast: f64,    // Next block
    #[serde(rename = "3")]
    pub medium: f64,  // 3 blocks
    #[serde(rename = "6")]
    pub slow: f64,    // 6 blocks
}

/// Retry configuration for Esplora API calls
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: std::time::Duration,
    /// Maximum delay between retries
    pub max_delay: std::time::Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Request timeout for each attempt
    pub request_timeout: std::time::Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: std::time::Duration::from_millis(500),
            max_delay: std::time::Duration::from_secs(30),
            backoff_multiplier: 2.0,
            request_timeout: std::time::Duration::from_secs(30),
        }
    }
}

/// Esplora API client for Bitcoin blockchain interactions
pub struct EsploraClient {
    base_url: String,
    retry_config: RetryConfig,
    
    client: reqwest::Client,
}

impl EsploraClient {
    /// Create a new Esplora client with the given base URL
    /// 
    /// # Examples
    /// 
    /// ```
    /// use bitcoin_anchor::bitcoin::EsploraClient;
    /// 
    /// // For testnet
    /// let client = EsploraClient::new("https://blockstream.info/testnet/api");
    /// 
    /// // For mainnet  
    /// let client = EsploraClient::new("https://blockstream.info/api");
    /// ```
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_retry_config(base_url, RetryConfig::default())
    }

    /// Create a new Esplora client with custom retry configuration
    pub fn with_retry_config(base_url: impl Into<String>, retry_config: RetryConfig) -> Self {
        let timeout = retry_config.request_timeout;
        Self {
            base_url: base_url.into(),
            retry_config,
            client: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        }
    }

    /// Create a new client for Bitcoin testnet (Blockstream.info)
    pub fn testnet() -> Self {
        Self::new("https://blockstream.info/testnet/api")
    }

    /// Create a new client for Bitcoin mainnet (Blockstream.info)  
    pub fn mainnet() -> Self {
        Self::new("https://blockstream.info/api")
    }

    /// Create a testnet client with custom retry configuration
    pub fn testnet_with_retry(retry_config: RetryConfig) -> Self {
        Self::with_retry_config("https://blockstream.info/testnet/api", retry_config)
    }

    /// Create a mainnet client with custom retry configuration
    pub fn mainnet_with_retry(retry_config: RetryConfig) -> Self {
        Self::with_retry_config("https://blockstream.info/api", retry_config)
    }

    /// Execute a request with retry logic and exponential backoff
    
    async fn execute_with_retry<F, T, Fut>(&self, operation: F, operation_name: &str) -> Result<T, EsploraError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, EsploraError>>,
    {
        let mut attempt = 0;
        let mut delay = self.retry_config.initial_delay;

        loop {
            attempt += 1;
            
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Check if we should retry this error
                    let should_retry = match &error {
                        EsploraError::Network(_) => true,
                        EsploraError::Http(_) => true,
                        _ => false,
                    };

                    // If we've exhausted our retries or the error is not retryable, return the error
                    if attempt >= self.retry_config.max_attempts || !should_retry {
                        return Err(error);
                    }

                    // Log the retry attempt (in a real implementation, use proper logging)
                    #[cfg(debug_assertions)]
                    eprintln!(
                        "Retry attempt {} for {} failed: {}. Retrying in {:?}",
                        attempt, operation_name, error, delay
                    );

                    // Wait before retrying
                    tokio::time::sleep(delay).await;

                    // Exponential backoff
                    delay = std::cmp::min(
                        std::time::Duration::from_millis(
                            (delay.as_millis() as f64 * self.retry_config.backoff_multiplier) as u64
                        ),
                        self.retry_config.max_delay
                    );
                }
            }
        }
    }

    /// Get UTXOs for a given address
    
    pub async fn get_address_utxos(&self, address: &Address) -> Result<Vec<EsploraUtxo>, EsploraError> {
        let address_clone = address.clone();
        self.execute_with_retry(
            move || {
                let url = format!("{}/address/{}/utxo", self.base_url, address_clone);
                let client = self.client.clone();
                async move {
                    let response = client
                        .get(&url)
                        .send()
                        .await
                        .map_err(|e| {
                            if e.is_timeout() {
                                EsploraError::Network(format!("Request timeout: {}", e))
                            } else if e.is_connect() {
                                EsploraError::Network(format!("Connection failed: {}", e))
                            } else {
                                EsploraError::Network(e.to_string())
                            }
                        })?;

                    if response.status() == 404 {
                        return Err(EsploraError::AddressNotFound);
                    }

                    if response.status() == 429 {
                        return Err(EsploraError::Http("Rate limited".to_string()));
                    }

                    if !response.status().is_success() {
                        return Err(EsploraError::Http(format!(
                            "HTTP error {}: {}",
                            response.status(),
                            response.text().await.unwrap_or_else(|_| "Unknown error".to_string())
                        )));
                    }

                    let utxos: Vec<EsploraUtxo> = response
                        .json()
                        .await
                        .map_err(|e| EsploraError::Json(e.to_string()))?;

                    Ok(utxos)
                }
            },
            "get_address_utxos"
        ).await
    }

    /// Get transaction details by transaction ID
    
    pub async fn get_transaction(&self, txid: &Txid) -> Result<EsploraTransaction, EsploraError> {
        let url = format!("{}/tx/{}", self.base_url, txid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        if response.status() == 404 {
            return Err(EsploraError::TransactionNotFound(*txid));
        }

        let tx: EsploraTransaction = response
            .json()
            .await
            .map_err(|e| EsploraError::Json(e.to_string()))?;

        Ok(tx)
    }

    /// Get raw transaction hex by transaction ID
    
    pub async fn get_transaction_hex(&self, txid: &Txid) -> Result<String, EsploraError> {
        let url = format!("{}/tx/{}/hex", self.base_url, txid);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        if response.status() == 404 {
            return Err(EsploraError::TransactionNotFound(*txid));
        }

        let hex = response
            .text()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        Ok(hex)
    }

    /// Broadcast a transaction to the network
    
    pub async fn broadcast_transaction(&self, tx_hex: &str) -> Result<Txid, EsploraError> {
        let url = format!("{}/tx", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Content-Type", "text/plain")
            .body(tx_hex.to_string())
            .send()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EsploraError::Http(format!("Broadcast failed: {}", error_text)));
        }

        let txid_str = response
            .text()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        let txid = txid_str.parse::<Txid>()
            .map_err(|e| EsploraError::Json(format!("Invalid txid returned: {}", e)))?;

        Ok(txid)
    }

    /// Get current fee estimates (sat/vB)
    
    pub async fn get_fee_estimates(&self) -> Result<FeeEstimates, EsploraError> {
        let url = format!("{}/fee-estimates", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        let estimates: FeeEstimates = response
            .json()
            .await
            .map_err(|e| EsploraError::Json(e.to_string()))?;

        Ok(estimates)
    }

    /// Get the current block height
    
    pub async fn get_block_height(&self) -> Result<u32, EsploraError> {
        let url = format!("{}/blocks/tip/height", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?;

        let height: u32 = response
            .text()
            .await
            .map_err(|e| EsploraError::Network(e.to_string()))?
            .parse()
            .map_err(|e| EsploraError::Json(format!("Invalid block height: {}", e)))?;

        Ok(height)
    }
}

impl fmt::Debug for EsploraClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EsploraClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}

impl EsploraUtxo {
    /// Convert Esplora UTXO to Bitcoin OutPoint
    pub fn to_outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid,
            vout: self.vout,
        }
    }

    /// Get the value as a Bitcoin Amount
    pub fn amount(&self) -> Amount {
        Amount::from_sat(self.value)
    }

    /// Check if this UTXO is confirmed
    pub fn is_confirmed(&self) -> bool {
        self.status.confirmed
    }

    /// Get the confirmation count (None if unconfirmed)
    pub async fn confirmation_count(&self, client: &EsploraClient) -> Result<Option<u32>, EsploraError> {
        if !self.is_confirmed() {
            return Ok(None);
        }

        
        {
            let current_height = client.get_block_height().await?;
            if let Some(block_height) = self.status.block_height {
                Ok(Some(current_height.saturating_sub(block_height) + 1))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esplora_client_creation() {
        let client = EsploraClient::new("https://blockstream.info/testnet/api");
        assert_eq!(client.base_url, "https://blockstream.info/testnet/api");

        let testnet_client = EsploraClient::testnet();
        assert_eq!(testnet_client.base_url, "https://blockstream.info/testnet/api");

        let mainnet_client = EsploraClient::mainnet();
        assert_eq!(mainnet_client.base_url, "https://blockstream.info/api");
    }

    #[test]
    fn test_utxo_helpers() {
        use bitcoin::Txid;
        use std::str::FromStr;

        let txid = Txid::from_str("a1b2c3d4e5f67890123456789012345678901234567890123456789012345678").unwrap();
        let utxo = EsploraUtxo {
            txid,
            vout: 0,
            value: 100000,
            status: UtxoStatus {
                confirmed: true,
                block_height: Some(800000),
                block_hash: Some("block_hash".to_string()),
                block_time: Some(1640000000),
            },
        };

        let outpoint = utxo.to_outpoint();
        assert_eq!(outpoint.txid, txid);
        assert_eq!(outpoint.vout, 0);

        let amount = utxo.amount();
        assert_eq!(amount.to_sat(), 100000);

        assert!(utxo.is_confirmed());
    }
} 