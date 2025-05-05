use std::{collections::HashMap, ops::Add};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/Env.scala
#[derive(Clone, Debug, Default)]
pub struct Env<V, K = u32>
where
    V: Default + Clone,
    K: Add + Default,
{
    pub entities: HashMap<K, V>,
    pub level: K,
    pub shift: K,
}

impl<V: Default + Clone, K: Add + Default> Env<V, K> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(self, a: V) -> Env<_, _> {
        Env {
            entities: HashMap::from_iter(self.entities.into_iter().chain([(self.level, a)])),
            level: self.level + 1,
            ..self
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        let key = self.level + self.shift - k - 1;
        self.entities.get(&key)
    }

    pub fn shift(&self, j: K) -> Env<V> {
        Env {
            shift: self.shift + j,
            ..self.clone()
        }
    }
}
