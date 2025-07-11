use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnchorConfig {
    pub method: AnchorMethod,
    pub frequency: AnchorFrequency,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnchorMethod {
    /// Privacy-preserving Taproot anchoring
    Taproot,
    /// Simple OP_RETURN anchoring
    OpReturn,
    /// Try Taproot first, fallback to OP_RETURN
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AnchorFrequency {
    /// Anchor every finalized block
    EveryBlock,
    /// Anchor every N finalized blocks
    EveryNBlocks(u32),
    /// Only manual anchoring
    Manual,
}

impl Default for AnchorConfig {
    fn default() -> Self {
        Self {
            method: AnchorMethod::Auto,
            frequency: AnchorFrequency::Manual,
        }
    }
} 