use crate::rhoapi::Par;
use std::collections::HashMap;
use std::iter::FromIterator;

// See models/src/main/scala/coop/rchain/models/SortedParMap.scala
#[derive(Debug, Clone)]
pub struct SortedParMap {
    sorted_map: HashMap<Par, Par>,
}

impl SortedParMap {
    pub fn new(map: HashMap<Par, Par>) -> Self {
        let sorted_map = map; // Assuming the map is already sorted or you can sort it here

        SortedParMap { sorted_map }
    }

    pub fn from_seq(seq: Vec<(Par, Par)>) -> Self {
        let map: HashMap<Par, Par> = seq.into_iter().collect();

        SortedParMap::new(map)
    }

    pub fn insert(&mut self, key: Par, value: Par) {
        self.sorted_map.insert(key, value);
    }

    pub fn get(&self, key: &Par) -> Option<&Par> {
        self.sorted_map.get(key)
    }

    pub fn remove(&mut self, key: &Par) {
        self.sorted_map.remove(key);
    }

    pub fn keys(&self) -> Vec<&Par> {
        self.sorted_map.keys().collect()
    }

    pub fn values(&self) -> Vec<&Par> {
        self.sorted_map.values().collect()
    }
}

impl FromIterator<(Par, Par)> for SortedParMap {
    fn from_iter<I: IntoIterator<Item = (Par, Par)>>(iter: I) -> Self {
        let map: HashMap<Par, Par> = iter.into_iter().collect();

        SortedParMap::new(map)
    }
}
