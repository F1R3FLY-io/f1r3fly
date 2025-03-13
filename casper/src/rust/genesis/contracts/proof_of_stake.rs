// See casper/src/main/scala/coop/rchain/casper/genesis/contracts/ProofOfStake.scala

use std::fmt::Write;

use super::validator::Validator;

// TODO: Eliminate validators argument if unnecessary. - OLD
// TODO: eliminate the default for epochLength. Now it is used in order to minimise the impact of adding this parameter - OLD
// TODO: Remove hardcoded keys from standard deploys: https://rchain.atlassian.net/browse/RCHAIN-3321?atlOrigin=eyJpIjoiNDc0NjE4YzYxOTRkNDcyYjljZDdlOWMxYjE1NWUxNjIiLCJwIjoiaiJ9 - OLD
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ProofOfStake {
    pub minimum_bond: i64,
    pub maximum_bond: i64,
    pub validators: Vec<Validator>,
    pub epoch_length: i32,
    pub quarantine_length: i32,
    pub number_of_active_validators: i32,
    pub pos_multi_sig_public_keys: Vec<String>,
    pub pos_multi_sig_quorum: i32,
}

impl ProofOfStake {
    // TODO: Determine how the "initial bonds" map can simulate transferring stake into the PoS contract
    //       when this must be done during genesis, under the authority of the genesisPk, which calls the
    //       linear receive in PoS.rho
    pub fn initial_bonds(validators: &[Validator]) -> String {
        let mut sorted_validators = validators.to_vec();
        sorted_validators.sort_by(|a, b| a.pk.bytes.cmp(&b.pk.bytes));

        let map_entries: Vec<String> = sorted_validators
            .iter()
            .map(|validator| {
                let pk_string = hex::encode(validator.pk.bytes.clone());
                format!(" \"{}\".hexToBytes() : {}", pk_string, validator.stake)
            })
            .collect();

        format!("{{{}}}", map_entries.join(", "))
    }

    pub fn public_keys(pos_multi_sig_public_keys: &[String]) -> String {
        let indent_brackets = 12;
        let indent_keys = indent_brackets + 2;

        let mut result = String::from("[\n");

        for (i, pk) in pos_multi_sig_public_keys.iter().enumerate() {
            write!(result, "{:indent_keys$}\"{}\".hexToBytes()", "", pk).unwrap();

            if i < pos_multi_sig_public_keys.len() - 1 {
                result.push_str(",\n");
            } else {
                result.push('\n');
            }
        }

        write!(result, "{:indent_brackets$}]", "").unwrap();
        result
    }
}
