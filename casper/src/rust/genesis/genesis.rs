// See casper/src/main/scala/coop/rchain/casper/genesis/Genesis.scala

use models::rhoapi::{g_unforgeable::UnfInstance, GPrivate, GUnforgeable, Par};

use crate::rust::util::rholang::tools::Tools;

use super::contracts::{proof_of_stake::ProofOfStake, standard_deploys, vault::Vault};

pub struct Genesis {
    pub shard_id: String,
    pub timestamp: u64,
    pub block_number: u64,
    pub proof_of_stake: ProofOfStake,
    pub vaults: Vec<Vault>,
    pub supply: u64,
}

impl Genesis {
    pub fn non_negative_mergeable_tag_name() -> Par {
        let mut rng = Tools::unforgeable_name_rng(
            &standard_deploys::NON_NEGATIVE_NUMBER_PUB_KEY,
            standard_deploys::NON_NEGATIVE_NUMBER_TIMESTAMP,
        );

        rng.next();
        let unforgeable_byte = rng.next();

        Par::default().with_unforgeables(vec![GUnforgeable {
            unf_instance: Some(UnfInstance::GPrivateBody(GPrivate {
                id: unforgeable_byte.into_iter().map(|b| b as u8).collect(),
            })),
        }])
    }
}
