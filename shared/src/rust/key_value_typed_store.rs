// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala

use std::collections::HashMap;

pub trait KeyValueTypedStore<K, V> {
    fn get(&self, keys: Vec<K>) -> Vec<Option<V>>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> ();

    fn delete(&self, keys: Vec<K>) -> ();

    fn contains(&self, keys: Vec<K>) -> Vec<bool>;

    /**
     * Efficient way to iterate and filter the whole KV store
     *
     * @param pf Partial function to project and filter values
     */
    fn collect<F, T>(&self, f: F) -> Vec<T>
    where
        F: FnMut((&K, Box<dyn FnOnce() -> V>)) -> Option<T>;

    fn to_map(&self) -> HashMap<K, V>;
}
