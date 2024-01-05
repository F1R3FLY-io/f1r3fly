// See shared/src/main/scala/coop/rchain/store/KeyValueTypedStore.scala
pub trait KeyValueTypedStore<K, V> {
    fn get(&self, keys: Vec<K>) -> Vec<Option<V>>;

    fn put(&self, kv_pairs: Vec<(K, V)>) -> ();

    fn delete(&self, keys: Vec<K>) -> i32;

    fn contains(&self, keys: Vec<K>) -> Vec<bool>;

    // def collect[T](pf: PartialFunction[(K, () => V), T]): F[Seq[T]]

    // def toMap: F[Map[K, V]]
}

pub struct KeyValueTypedStoreStruct {}
