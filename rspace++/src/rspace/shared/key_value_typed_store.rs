use crate::rspace::shared::key_value_store::KeyValueStore;
use std::{collections::BTreeMap, marker::PhantomData};

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala
pub trait KeyValueTypedStore<K, V> {
    fn get(&self, keys: Vec<K>) -> Vec<Option<V>>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> ();

    fn delete(&self, keys: Vec<K>) -> i32;

    fn contains(&self, keys: Vec<K>) -> Vec<bool>;

    // def collect[T](pf: PartialFunction[(K, () => V), T]): F[Seq[T]]
    // TODO: Update this to match scala
    fn collect(&self) -> ();

    fn to_map(&self) -> BTreeMap<K, V>;
}

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala
#[derive(Clone)]
pub struct KeyValueTypedStoreInstance<U: KeyValueStore, K, V> {
    pub store: U,
    pub _marker: PhantomData<(K, V)>,
}

impl<U: KeyValueStore, K, V> KeyValueTypedStore<K, V> for KeyValueTypedStoreInstance<U, K, V> {
    fn get(&self, keys: Vec<K>) -> Vec<Option<V>> {
        todo!()
    }

    fn put(&self, kv_pairs: Vec<(K, V)>) -> () {
        todo!()
    }

    fn delete(&self, keys: Vec<K>) -> i32 {
        todo!()
    }

    fn contains(&self, keys: Vec<K>) -> Vec<bool> {
        todo!()
    }

    fn collect(&self) -> () {
        todo!()
    }

    fn to_map(&self) -> BTreeMap<K, V> {
        todo!()
    }
}
