// See casper/src/main/scala/coop/rchain/casper/merging/DagMerger.scala

use super::deploy_chain_index::DeployChainIndex;

pub fn cost_optimal_rejection_alg() -> impl Fn(&DeployChainIndex) -> u64 {
    |deploy_chain_index: &DeployChainIndex| {
        deploy_chain_index
            .deploys_with_cost
            .0
            .iter()
            .map(|deploy| deploy.cost)
            .sum()
    }
}
