use crate::rhoapi::{EMap, KeyValuePair, Par, Var};
use crate::BitSet;

use super::utils::union;

// See models/src/main/scala/coop/rchain/models/ParMap.scala
// See models/src/main/scala/coop/rchain/models/SortedParMap.scala
// In Rust, because we can't ovveride field types, we are going to add methods to EMap
// to behave similarly to ParMap
impl EMap {
    pub fn new(
        seq: Vec<(Par, Par)>,
        connective_used: bool,
        locally_free: BitSet,
        remainder: Option<Var>,
    ) -> Self {
        let kvs: Vec<KeyValuePair> = seq
            .into_iter()
            .map(|kv| KeyValuePair {
                key: Some(kv.0),
                value: Some(kv.1),
            })
            .collect();

        EMap {
            kvs,
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

    fn connective_used(map: &[(Par, Par)]) -> bool {
        map.iter()
            .any(|(k, v)| k.connective_used || v.connective_used)
    }

    fn update_locally_free(ps: &[(Par, Par)]) -> BitSet {
        ps.iter().fold(Vec::new(), |acc, (key, value)| {
            union(
                acc,
                union(key.locally_free.clone(), value.locally_free.clone()),
            )
        })
    }
}
