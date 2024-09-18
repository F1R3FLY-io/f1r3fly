use std::collections::HashMap;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/Env.scala
#[derive(Clone, Debug)]
pub struct Env<A: Clone> {
    pub env_map: HashMap<i32, A>,
    pub level: i32,
    pub shift: i32,
}

impl<A: Clone> Env<A> {
    pub fn new() -> Env<A> {
        Env {
            env_map: HashMap::new(),
            level: 0,
            shift: 0,
        }
    }

    pub fn put(&mut self, a: A) -> Env<A> {
        Env {
            env_map: {
                self.env_map.insert(self.level, a);
                self.env_map.clone()
            },
            level: self.level + 1,
            shift: self.shift,
        }
    }

    pub fn get(&self, k: &i32) -> Option<A> {
        self.env_map
            .get(&((self.level + self.shift) - k - 1))
            .cloned()
    }

    pub fn shift(&self, j: i32) -> Env<A> {
        Env {
            shift: self.shift + j,
            ..(*self).clone()
        }
    }
}
