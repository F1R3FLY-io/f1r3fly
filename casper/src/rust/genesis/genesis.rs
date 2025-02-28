// See casper/src/main/scala/coop/rchain/casper/genesis/Genesis.scala

use super::contracts::{proof_of_stake::ProofOfStake, vault::Vault};

pub struct Genesis {
    pub shard_id: String,
    pub timestamp: u64,
    pub block_number: u64,
    pub proof_of_stake: ProofOfStake,
    pub vaults: Vec<Vault>,
    pub supply: u64,
}
