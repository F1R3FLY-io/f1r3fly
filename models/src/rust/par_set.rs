// See models/src/main/scala/coop/rchain/models/ParSet.scala

use crate::rhoapi::{Par, Var};

use super::{sorted_par_hash_set::SortedParHashSet, utils::union};

#[derive(Clone)]
pub struct ParSet {
    pub ps: SortedParHashSet,
    pub connective_used: bool,
    pub locally_free: Vec<u8>,
    pub remainder: Option<Var>,
}

impl ParSet {
    pub fn new(
        vec: Vec<Par>,
        connective_used: bool,
        locally_free: Vec<u8>,
        remainder: Option<Var>,
    ) -> ParSet {
        ParSet {
            ps: SortedParHashSet::create_from_vec(vec),
            connective_used,
            locally_free,
            remainder,
        }
    }

    pub fn create_from_vec_and_remainder(vec: Vec<Par>, remainder: Option<Var>) -> Self {
        let shs = SortedParHashSet::create_from_vec(vec.clone());
        ParSet {
            ps: shs.clone(),
            connective_used: ParSet::connective_used(&vec) || remainder.is_some(),
            locally_free: ParSet::update_locally_free(&shs),
            remainder,
        }
    }

    pub fn create_from_vec(vec: Vec<Par>) -> Self {
        ParSet::create_from_vec_and_remainder(vec.clone(), None)
    }

    pub fn equals(&self, other: ParSet) -> bool {
        self.ps.equals(other.ps)
            && self.remainder == other.remainder
            && self.connective_used == other.connective_used
    }

    fn connective_used(vec: &Vec<Par>) -> bool {
        vec.iter().any(|p| p.connective_used)
    }

    fn update_locally_free(ps: &SortedParHashSet) -> Vec<u8> {
        ps.sorted_pars
            .clone()
            .into_iter()
            .fold(Vec::new(), |acc, p| union(acc, p.locally_free))
    }
}
