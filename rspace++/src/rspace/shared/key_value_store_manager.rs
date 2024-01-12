use super::key_value_store::KeyValueStore;

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreManager.scala
pub trait KeyValueStoreManager {
    fn store(&self, name: String) -> Box<dyn KeyValueStore>;

    fn shutdown(&self) -> ();
}
