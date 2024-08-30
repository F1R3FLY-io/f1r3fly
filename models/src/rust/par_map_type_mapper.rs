// See models/src/main/scala/coop/rchain/models/ParMapTypeMapper.scala

use crate::rhoapi::{EMap, KeyValuePair, Par};

use super::par_map::ParMap;

pub struct ParMapTypeMapper;

impl ParMapTypeMapper {
    pub fn emap_to_par_map(emap: EMap) -> ParMap {
        let ps: Vec<(Par, Par)> = emap.kvs.iter().map(Self::unzip).collect();
        ParMap::new(ps, emap.connective_used, emap.locally_free, emap.remainder)
    }

    pub fn par_map_to_emap(par_map: ParMap) -> EMap {
        let kvs: Vec<KeyValuePair> = par_map
            .ps
            .sorted_list
            .iter()
            .map(|(k, v)| Self::zip(k.clone(), v.clone()))
            .collect();

        EMap {
            kvs,
            locally_free: par_map.locally_free,
            connective_used: par_map.connective_used,
            remainder: par_map.remainder,
        }
    }

    fn unzip(kvp: &KeyValuePair) -> (Par, Par) {
        (kvp.key.clone().unwrap(), kvp.value.clone().unwrap())
    }

    fn zip(k: Par, v: Par) -> KeyValuePair {
        KeyValuePair {
            key: Some(k),
            value: Some(v),
        }
    }
}
