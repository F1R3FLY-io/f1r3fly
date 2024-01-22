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

    fn clone_box(&self) -> Box<dyn KeyValueTypedStore<K, V>>;
}

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala
pub struct KeyValueTypedStoreInstance<K, V> {
    pub store: Box<dyn KeyValueStore>,
    pub _marker: PhantomData<(K, V)>,
}

impl<K: 'static, V: 'static> KeyValueTypedStore<K, V> for KeyValueTypedStoreInstance<K, V> {
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

    fn clone_box(&self) -> Box<dyn KeyValueTypedStore<K, V>> {
        Box::new(self.clone())
    }
}

impl<K, V> Clone for Box<dyn KeyValueTypedStore<K, V>> {
    fn clone(&self) -> Box<dyn KeyValueTypedStore<K, V>> {
        self.clone_box()
    }
}

impl<K, V> Clone for KeyValueTypedStoreInstance<K, V> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            _marker: PhantomData,
        }
    }
}
