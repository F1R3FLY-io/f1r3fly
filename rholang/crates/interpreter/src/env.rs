use std::collections::HashMap;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/Env.scala
#[derive(Clone, Debug, Default)]
pub struct Env<V: Default + Clone> {
    pub entities: HashMap<i32, V>,
    pub level: i32,
    pub shift: i32,
}

impl<V: Default + Clone> Env<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(self, a: V) -> Env<V> {
        Env {
            entities: HashMap::from_iter(self.entities.into_iter().chain([(self.level, a)])),
            level: self.level + 1,
            ..self
        }
    }

    pub fn get(&self, k: &i32) -> Option<&V> {
        let key = self.level + self.shift - k - 1;
        self.entities.get(&key)
    }

    pub fn shift(&self, j: i32) -> Env<V> {
        Env {
            shift: self.shift + j,
            ..self.clone()
        }
    }
}
