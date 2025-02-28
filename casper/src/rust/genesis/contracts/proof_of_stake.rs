// See casper/src/main/scala/coop/rchain/casper/genesis/contracts/ProofOfStake.scala

use super::validator::Validator;

// TODO: Eliminate validators argument if unnecessary. - OLD
// TODO: eliminate the default for epochLength. Now it is used in order to minimise the impact of adding this parameter - OLD
// TODO: Remove hardcoded keys from standard deploys: https://rchain.atlassian.net/browse/RCHAIN-3321?atlOrigin=eyJpIjoiNDc0NjE4YzYxOTRkNDcyYjljZDdlOWMxYjE1NWUxNjIiLCJwIjoiaiJ9 - OLD
pub struct ProofOfStake {
    pub minimum_bond: u64,
    pub maximum_bond: u64,
    pub validators: Vec<Validator>,
    pub epoch_length: i64,
    pub quarantine_length: i64,
    pub number_of_active_validators: i64,
    pub pos_multi_sig_public_keys: Vec<String>,
    pub pos_multi_sig_quorum: i64,
}
