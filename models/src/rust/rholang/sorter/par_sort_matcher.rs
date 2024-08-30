// See models/src/main/scala/coop/rchain/models/rholang/sorter/ParSortMatcher.scala

use crate::rhoapi::Par;

use super::{score_tree::ScoredTerm, sortable::Sortable};

pub struct ParSortMatcher;

impl Sortable<Par> for ParSortMatcher {
    fn sort_match(term: &Par) -> ScoredTerm<Par> {
        todo!()
    }
}
