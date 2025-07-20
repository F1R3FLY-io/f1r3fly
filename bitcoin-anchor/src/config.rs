use bpstd::Network;

/// Configuration for F1r3fly Bitcoin anchoring
#[derive(Clone, Debug)]
pub struct AnchorConfig {
    /// Bitcoin network to use (mainnet, testnet, etc.)
    pub network: Network,
    /// RGB protocol activation flag
    pub rgb_enabled: bool,
    /// Enable RGB Multi-Protocol Commitments
    pub mpc_enabled: bool,
    /// Optional: Fee rate override (sat/vbyte)
    pub fee_rate_override: Option<u64>,
}

impl AnchorConfig {
    /// Create configuration for Bitcoin mainnet
    pub fn mainnet() -> Self {
        Self {
            network: Network::Mainnet,
            rgb_enabled: true,
            mpc_enabled: true,
            fee_rate_override: None,
        }
    }

    /// Create configuration for Bitcoin testnet
    pub fn testnet() -> Self {
        Self {
            network: Network::Testnet3,
            rgb_enabled: true,
            mpc_enabled: true,
            fee_rate_override: None,
        }
    }

    /// Create configuration for Bitcoin regtest
    pub fn regtest() -> Self {
        Self {
            network: Network::Regtest,
            rgb_enabled: true,
            mpc_enabled: true,
            fee_rate_override: None,
        }
    }

    /// Create configuration for Bitcoin signet
    pub fn signet() -> Self {
        Self {
            network: Network::Signet,
            rgb_enabled: true,
            mpc_enabled: true,
            fee_rate_override: None,
        }
    }

    /// Create basic configuration without RGB features
    pub fn basic(network: Network) -> Self {
        Self {
            network,
            rgb_enabled: false,
            mpc_enabled: false,
            fee_rate_override: None,
        }
    }

    /// Enable RGB protocol features
    pub fn with_rgb(mut self) -> Self {
        self.rgb_enabled = true;
        self
    }

    /// Enable RGB Multi-Protocol Commitments
    pub fn with_mpc(mut self) -> Self {
        self.mpc_enabled = true;
        self
    }

    /// Override default fee rate
    pub fn with_fee_rate(mut self, fee_rate_sat_per_vbyte: u64) -> Self {
        self.fee_rate_override = Some(fee_rate_sat_per_vbyte);
        self
    }
} 