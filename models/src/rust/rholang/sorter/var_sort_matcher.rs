// See models/src/main/scala/coop/rchain/models/rholang/sorter/VarSortMatcher.scala

use crate::rhoapi::{var::VarInstance, Var};

use super::{
    score_tree::{Score, ScoreAtom, ScoredTerm, Tree},
    sortable::Sortable,
};

pub struct VarSortMatcher;

impl Sortable<Var> for VarSortMatcher {
    fn sort_match(v: &Var) -> ScoredTerm<Var> {
        match &v.var_instance {
            Some(var) => match var {
                VarInstance::BoundVar(level) => ScoredTerm {
                    term: v.clone(),
                    score: Tree::<ScoreAtom>::create_node_from_i64s(vec![
                        Score::BOUND_VAR as i64,
                        *level as i64,
                    ]),
                },

                VarInstance::FreeVar(level) => ScoredTerm {
                    term: v.clone(),
                    score: Tree::<ScoreAtom>::create_node_from_i64s(vec![
                        Score::FREE_VAR as i64,
                        *level as i64,
                    ]),
                },

                VarInstance::Wildcard(_) => ScoredTerm {
                    term: v.clone(),
                    score: Tree::<ScoreAtom>::create_node_from_i64s(vec![Score::WILDCARD as i64]),
                },
            },
            None => ScoredTerm {
                term: Var::default(),
                score: Tree::<ScoreAtom>::create_leaf_from_i64(Score::ABSENT as i64),
            },
        }
    }
}
