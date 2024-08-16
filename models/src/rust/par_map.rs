use crate::rhoapi::{Par, Var};
use crate::BitSet;

use super::sorted_par_map::SortedParMap;

// See models/src/main/scala/coop/rchain/models/ParMap.scala
#[derive(Debug, Clone)]
pub struct ParMap {
    ps: SortedParMap,
    connective_used: bool,
    locally_free: BitSet,
    remainder: Option<Var>,
}

impl ParMap {
    pub fn new(
        seq: Vec<(Par, Par)>,
        connective_used: bool,
        locally_free: BitSet,
        remainder: Option<Var>,
    ) -> Self {
        let ps = SortedParMap::from_iter(seq);

        ParMap {
            ps,
            connective_used,
            locally_free,
            remainder,
        }
    }

    pub fn from_seq(seq: Vec<(Par, Par)>) -> Self {
        let connective_used = Self::connective_used(&seq);
        let locally_free = Self::update_locally_free(&seq);

        Self::new(seq, connective_used, locally_free, None)
    }

    pub fn from_sorted_map(map: SortedParMap) -> Self {
        Self::from_seq(
            map.keys()
                .into_iter()
                .map(|k| (k.clone(), map.get(k).unwrap().clone()))
                .collect(),
        )
    }

    fn connective_used(map: &[(Par, Par)]) -> bool {
        map.iter()
            .any(|(k, v)| k.connective_used || v.connective_used)
    }

    fn update_locally_free(ps: &[(Par, Par)]) -> BitSet {
        let mut locally_free_set = Vec::new();

        for (key, value) in ps {
            locally_free_set.extend_from_slice(&key.locally_free);
            locally_free_set.extend_from_slice(&value.locally_free);
        }
        locally_free_set
    }
}
