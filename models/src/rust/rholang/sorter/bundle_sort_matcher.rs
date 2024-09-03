// See models/src/main/scala/coop/rchain/models/rholang/sorter/BundleSortMatcher.scala

use crate::rhoapi::Bundle;

use super::{
    par_sort_matcher::ParSortMatcher,
    score_tree::{Score, ScoreAtom, ScoredTerm, Tree},
    sortable::Sortable,
};

pub struct BundleSortMatcher;

impl Sortable<Bundle> for BundleSortMatcher {
    fn sort_match(b: &Bundle) -> ScoredTerm<Bundle> {
        let score = if b.write_flag && b.read_flag {
            Score::BUNDLE_READ_WRITE
        } else if b.write_flag && !b.read_flag {
            Score::BUNDLE_WRITE
        } else if !b.write_flag && b.read_flag {
            Score::BUNDLE_READ
        } else {
            Score::BUNDLE_EQUIV
        };

        let sorted_par = ParSortMatcher::sort_match(
            &b.body.as_ref().expect("body was None, should be Some(Par)"),
        );

        ScoredTerm {
            term: {
                let mut b_cloned = b.clone();
                b_cloned.body = Some(sorted_par.term);
                b_cloned
            },
            score: Tree::<ScoreAtom>::create_node_from_i32(score, vec![sorted_par.score]),
        }
    }
}
