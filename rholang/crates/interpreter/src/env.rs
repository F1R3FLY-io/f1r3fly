use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct Env<V>
where
    V: Default + Clone,
{
    pub entities: HashMap<u32, V>,
    pub level: u32,
    pub shift: u32,
}

impl<V: Default + Clone> Env<V> {
    pub fn put(self, a: V) -> Env<V> {
        let entities =
            HashMap::from_iter(self.entities.into_iter().chain([(self.level.clone(), a)]));

        Env {
            entities,
            level: self.level + 1,
            ..self
        }
    }

    pub fn get(&self, k: u32) -> Option<&V> {
        let key = self.level.clone() + self.shift.clone() - k - 1;
        self.entities.get(&key)
    }

    pub fn shift(&self, j: u32) -> Env<V> {
        Env {
            shift: self.shift + j,
            ..self.to_owned()
        }
    }
}
