// See models/src/main/scala/coop/rchain/models/ParMap.scala

use crate::rhoapi::{Par, Var};

use super::{sorted_par_map::SortedParMap, utils::union};

#[derive(Clone, Debug)]
pub struct ParMap {
    pub ps: SortedParMap,
    pub connective_used: bool,
    pub locally_free: Vec<u8>,
    pub remainder: Option<Var>,
}

impl ParMap {
    pub fn new(
        vec: Vec<(Par, Par)>,
        connective_used: bool,
        locally_free: Vec<u8>,
        remainder: Option<Var>,
    ) -> ParMap {
        ParMap {
            ps: SortedParMap::create_from_vec(vec),
            connective_used,
            locally_free,
            remainder,
        }
    }

    pub fn create_from_vec(vec: Vec<(Par, Par)>) -> Self {
        ParMap::new(
            vec.clone(),
            ParMap::connective_used(&vec),
            ParMap::update_locally_free(&vec),
            None,
        )
    }

    pub fn create_from_sorted_par_map(map: SortedParMap) -> Self {
        ParMap::create_from_vec(map.sorted_list)
    }

    pub fn equals(&self, other: ParMap) -> bool {
        self.ps.equals(other.ps)
            && self.remainder == other.remainder
            && self.connective_used == other.connective_used
    }

    fn connective_used(map: &Vec<(Par, Par)>) -> bool {
        map.iter()
            .any(|(k, v)| k.connective_used || v.connective_used)
    }

    fn update_locally_free(ps: &Vec<(Par, Par)>) -> Vec<u8> {
        ps.into_iter().fold(Vec::new(), |acc, (key, value)| {
            union(
                acc,
                union(key.locally_free.clone(), value.locally_free.clone()),
            )
        })
    }
}
