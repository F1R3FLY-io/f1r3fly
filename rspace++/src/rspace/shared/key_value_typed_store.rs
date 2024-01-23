use crate::rspace::shared::key_value_store::KeyValueStore;
use async_trait::async_trait;
use serde::Serialize;
use std::fmt::Debug;
use std::io::Cursor;
use std::{collections::BTreeMap, marker::PhantomData};

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala
#[async_trait]
pub trait KeyValueTypedStore<K: Debug + Clone + Send + Sync, V> {
    async fn get(&self, keys: Vec<K>) -> Vec<Option<V>>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> ();

    fn delete(&self, keys: Vec<K>) -> i32;

    fn contains(&self, keys: Vec<K>) -> Vec<bool>;

    // def collect[T](pf: PartialFunction[(K, () => V), T]): F[Seq[T]]
    // TODO: Update this to match scala
    fn collect(&self) -> ();

    fn to_map(&self) -> BTreeMap<K, V>;

    // See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreSyntax.scala
    async fn get_one(&self, key: &K) -> Option<V> {
        let mut values = self.get(vec![key.clone()]).await;
        let first_value = values.remove(0);

        match first_value {
            Some(value) => Some(value),
            None => {
                panic!("Key_Value_Store: key not found: {:?}", key);
            }
        }
    }

    // fn clone_box(&self) -> Box<dyn KeyValueTypedStore<K, V>>;
}

// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStoreCodec.scala
#[derive(Clone)]
pub struct KeyValueTypedStoreInstance<K, V> {
    pub store: Box<dyn KeyValueStore>,
    pub _marker: PhantomData<(K, V)>,
}

#[async_trait]
impl<K: Debug + Clone + Serialize + Send + 'static + Sync, V: Send + 'static + Sync>
    KeyValueTypedStore<K, V> for KeyValueTypedStoreInstance<K, V>
{
    async fn get(&self, keys: Vec<K>) -> Vec<Option<V>> {
        let keys_bytes_res = keys
            .into_iter()
            .map(|key| bincode::serialize(&key))
            .collect::<Result<Vec<_>, _>>();

        let keys_buf: Vec<Vec<u8>> = match keys_bytes_res {
            Ok(keys_bytes) => keys_bytes.into_iter().map(|bytes| bytes).collect(),
            Err(err) => {
                panic!("Key Value Typed Store: {:?}", err)
            }
        };

        let values_bytes = self.store.get(keys_buf).await;

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

    // fn clone_box(&self) -> Box<dyn KeyValueTypedStore<K, V>> {
    //     Box::new(self.clone())
    // }
}

// impl<K: Debug + Clone + Send + Sync, V> Clone for Box<dyn KeyValueTypedStore<K, V>> {
//     fn clone(&self) -> Box<dyn KeyValueTypedStore<K, V>> {
//         self.clone_box()
//     }
// }

// impl<K, V> Clone for KeyValueTypedStoreInstance<K, V> {
//     fn clone(&self) -> Self {
//         Self {
//             store: self.store.clone(),
//             _marker: PhantomData,
//         }
//     }
// }
