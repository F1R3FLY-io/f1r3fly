use std::{collections::BTreeMap, ops::Deref};

pub(crate) struct EnvHashMap<K = String, V = crate::normal_forms::Par>(BTreeMap<K, V>);

impl<K, V> EnvHashMap<K, V> {
    pub fn new() -> Self {
        EnvHashMap(BTreeMap::new())
    }
}

impl<K, V> Deref for EnvHashMap<K, V> {
    type Target = BTreeMap<K, V>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<String, models::rhoapi::Par>> for EnvHashMap<String, crate::normal_forms::Par> {
    fn from(value: BTreeMap<String, models::rhoapi::Par>) -> Self {
        EnvHashMap(
            value
                .into_iter()
                .map(|(k, v)| (k, crate::normal_forms::Par::from(v)))
                .collect(),
        )
    }
}
