// See models/src/main/scala/coop/rchain/models/SortedParHashSet.scala

use std::collections::HashSet;

use crate::rhoapi::Par;

use super::rholang::sorter::{
    ordering::Ordering, par_sort_matcher::ParSortMatcher, sortable::Sortable,
};

// Enforce ordering and uniqueness.
// - uniqueness is handled by using HashSet.
// - ordering comes from sorting the elements prior to serializing.
#[derive(Clone)]
pub struct SortedParHashSet {
    pub ps: HashSet<Par>,
    pub sorted_pars: Vec<Par>,
    sorted_ps: HashSet<Par>,
}

impl SortedParHashSet {
    pub fn create_from_vec(vec: Vec<Par>) -> Self {
        let sorted_pars = Ordering::sort_pars(&vec);
        let sorted_ps = sorted_pars.clone().into_iter().collect();

        SortedParHashSet {
            ps: vec.into_iter().collect(),
            sorted_pars,
            sorted_ps,
        }
    }

    pub fn create_from_set(set: HashSet<Par>) -> Self {
        let vec = set.into_iter().collect();
        SortedParHashSet::create_from_vec(vec)
    }

    pub fn create_from_empty() -> Self {
        SortedParHashSet::create_from_set(HashSet::new())
    }

    // alias for '+'
    pub fn insert(&mut self, elem: Par) -> SortedParHashSet {
        self.ps.insert(Self::sort(&elem));
        self.clone()
    }

    // alias for '-'
    pub fn remove(&mut self, elem: Par) -> SortedParHashSet {
        self.ps.remove(&Self::sort(&elem));
        self.clone()
    }

    pub fn contains(&self, elem: Par) -> bool {
        self.sorted_ps.contains(&Self::sort(&elem))
    }

    pub fn union(&self, that: HashSet<Par>) -> SortedParHashSet {
        SortedParHashSet::create_from_set(
            self.sorted_ps
                .union(&that.iter().map(Self::sort).collect())
                .cloned()
                .collect(),
        )
    }

    pub fn equals(&self, that: SortedParHashSet) -> bool {
        self.sorted_pars == that.sorted_pars
    }

    pub fn length(&self) -> usize {
        self.sorted_ps.len()
    }

    fn sort(par: &Par) -> Par {
        ParSortMatcher::sort_match(&par).term.clone()
    }
}
