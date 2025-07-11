//! Bitcoin L1 anchoring for F1r3fly RSpace state
//! 
//! This crate provides Bitcoin Layer 1 anchoring functionality for F1r3fly's RSpace state.

pub mod config;
pub mod commitment;
pub mod error;

// Re-export main types
pub use error::{AnchorError, AnchorResult};
pub use config::AnchorConfig;
pub use commitment::F1r3flyStateCommitment;

/// Main Bitcoin anchoring service for F1r3fly
pub struct F1r3flyBitcoinAnchor {
    config: AnchorConfig,
}

impl F1r3flyBitcoinAnchor {
    /// Create a new Bitcoin anchor service
    pub fn new(config: AnchorConfig) -> AnchorResult<Self> {
        Ok(Self { config })
    }
    
    /// Manually anchor the current state (implementation in later steps)
    pub async fn anchor_current_state(&self) -> AnchorResult<()> {
        todo!("Will implement when we add Bitcoin and RGB dependencies")
    }
} 