use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Add, Sub},
};

#[derive(Clone, Debug, Default)]
pub struct Env<K, V>
where
    K: Add<Output = K> + Sub<Output = K> + Clone + Eq + Hash + From<u8>,
    V: Default + Clone,
{
    pub entities: HashMap<K, V>,
    pub level: K,
    pub shift: K,
}

impl<K: Add<Output = K> + Sub<Output = K> + Clone + Eq + Hash + From<u8>, V: Default + Clone>
    Env<K, V>
{
    pub fn put(self, a: V) -> Env<K, V> {
        let entities =
            HashMap::<K, V>::from_iter(self.entities.into_iter().chain([(self.level.clone(), a)]));

        Env {
            entities,
            level: self.level + 1.into(),
            ..self
        }
    }

    pub fn get(&self, k: K) -> Option<&V> {
        let key = self.level.clone() + self.shift.clone() - k - 1.into();
        self.entities.get(&key)
    }

    pub fn shift(&self, j: K) -> Env<K, V> {
        Env {
            shift: self.shift.clone() + j,
            ..self.to_owned()
        }
    }
}
