// See models/src/main/scala/coop/rchain/models/rholang/sorter/UnforgeableSortMatcher.scala

use crate::rhoapi::{g_unforgeable::UnfInstance, GUnforgeable};

use super::{
    score_tree::{Score, ScoreAtom, ScoredTerm, Tree},
    sortable::Sortable,
};

pub struct UnforgeableSortMatcher;

impl Sortable<GUnforgeable> for UnforgeableSortMatcher {
    fn sort_match(unf: &GUnforgeable) -> ScoredTerm<GUnforgeable> {
        match &unf.unf_instance {
            Some(unf) => match unf {
                UnfInstance::GPrivateBody(gpriv) => ScoredTerm {
                    term: GUnforgeable {
                        unf_instance: Some(UnfInstance::GPrivateBody(gpriv.clone())),
                    },
                    score: Tree::<ScoreAtom>::create_node_from_i32(
                        Score::PRIVATE,
                        vec![Tree::<ScoreAtom>::create_leaf_from_bytes(gpriv.id.clone())],
                    ),
                },

                UnfInstance::GDeployerIdBody(id) => ScoredTerm {
                    term: GUnforgeable {
                        unf_instance: Some(UnfInstance::GDeployerIdBody(id.clone())),
                    },
                    score: Tree::<ScoreAtom>::create_node_from_i32(
                        Score::DEPLOYER_AUTH,
                        vec![Tree::<ScoreAtom>::create_leaf_from_bytes(
                            id.public_key.clone(),
                        )],
                    ),
                },

                UnfInstance::GDeployIdBody(deploy_id) => ScoredTerm {
                    term: GUnforgeable {
                        unf_instance: Some(UnfInstance::GDeployIdBody(deploy_id.clone())),
                    },
                    score: Tree::<ScoreAtom>::create_node_from_i32(
                        Score::DEPLOY_ID,
                        vec![Tree::<ScoreAtom>::create_leaf_from_bytes(
                            deploy_id.sig.clone(),
                        )],
                    ),
                },

                UnfInstance::GSysAuthTokenBody(token) => ScoredTerm {
                    term: GUnforgeable {
                        unf_instance: Some(UnfInstance::GSysAuthTokenBody(token.clone())),
                    },
                    score: Tree::<ScoreAtom>::create_node_from_i32(Score::SYS_AUTH_TOKEN, vec![]),
                },
            },
            None => ScoredTerm {
                term: unf.clone(),
                score: Tree::<ScoreAtom>::create_node_from_i32(Score::ABSENT, Vec::new()),
            },
        }
    }
}
