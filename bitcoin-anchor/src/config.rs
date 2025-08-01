use bpstd::Network;

/// Configuration for F1r3fly Bitcoin anchoring
#[derive(Clone, Debug)]
pub struct AnchorConfig {
    /// Bitcoin network to use (mainnet, testnet, etc.)
    pub network: Network,

    /// Optional: Fee rate override (sat/vbyte)
    pub fee_rate_override: Option<u64>,
}

impl AnchorConfig {
    /// Create configuration for Bitcoin mainnet
    pub fn mainnet() -> Self {
        Self {
            network: Network::Mainnet,
            fee_rate_override: None,
        }
    }

    /// Create configuration for Bitcoin testnet
    pub fn testnet() -> Self {
        Self {
            network: Network::Testnet3,
            fee_rate_override: None,
        }
    }

    /// Create configuration for Bitcoin regtest
    pub fn regtest() -> Self {
        Self {
            network: Network::Regtest,
            fee_rate_override: None,
        }
    }

    /// Create configuration for Bitcoin signet
    pub fn signet() -> Self {
        Self {
            network: Network::Signet,
            fee_rate_override: None,
        }
    }

    /// Create basic configuration for any network
    pub fn basic(network: Network) -> Self {
        Self {
            network,
            fee_rate_override: None,
        }
    }

    /// Override default fee rate
    pub fn with_fee_rate(mut self, fee_rate_sat_per_vbyte: u64) -> Self {
        self.fee_rate_override = Some(fee_rate_sat_per_vbyte);
        self
    }
}
