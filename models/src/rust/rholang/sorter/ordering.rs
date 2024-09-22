// See models/src/main/scala/coop/rchain/models/rholang/sorter/ordering.scala

use std::collections::HashMap;

use crate::rhoapi::Par;

use super::{par_sort_matcher::ParSortMatcher, score_tree::ScoredTerm, sortable::Sortable};

pub struct Ordering;

impl Ordering {
    pub fn sort_pars(ps: &Vec<Par>) -> Vec<Par> {
        let mut ps_sorted: Vec<ScoredTerm<Par>> =
            ps.iter().map(ParSortMatcher::sort_match).collect();
        // println!("\nps_sorted in sort_pars before sort: {:?}", ps_sorted);
        ScoredTerm::sort_vec(&mut ps_sorted);
        // println!("\nps_sorted in sort_pars after sort: {:?}", ps_sorted);
        ps_sorted.into_iter().map(|st| st.term).collect()
    }

    pub fn sort_key_value_pair(key: &Par, value: &Par) -> ScoredTerm<(Par, Par)> {
        let sorted_key = ParSortMatcher::sort_match(key);
        let sorted_value = ParSortMatcher::sort_match(value);

        ScoredTerm {
            term: (sorted_key.term, sorted_value.term),
            score: sorted_key.score,
        }
    }

    pub fn sort_map(ps: &HashMap<Par, Par>) -> Vec<(Par, Par)> {
        let mut pairs_sorted: Vec<ScoredTerm<(Par, Par)>> = ps
            .iter()
            .map(|kv| Ordering::sort_key_value_pair(kv.0, kv.1))
            .collect();

        ScoredTerm::sort_vec(&mut pairs_sorted);
        pairs_sorted.into_iter().map(|st| st.term).collect()
    }
}
