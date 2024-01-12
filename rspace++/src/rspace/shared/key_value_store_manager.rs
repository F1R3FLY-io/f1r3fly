use super::key_value_store::KeyValueStore;
use crate::rspace::rspace::RSpaceStore;

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreManager.scala
pub trait KeyValueStoreManager {
    fn store(&self, name: String) -> Box<dyn KeyValueStore>;

    fn shutdown(&self) -> ();

    fn r_space_stores(&self) -> RSpaceStore {
        self.get_stores("rspace")
    }

    fn eval_stores(&self) -> RSpaceStore {
        self.get_stores("eval")
    }

    // TODO: This function should be private
    fn get_stores(&self, db_prefix: &str) -> RSpaceStore {
        let history = self.store(format!("{}-history", db_prefix));
        let roots = self.store(format!("{}-roots", db_prefix));
        let cold = self.store(format!("{}-cold", db_prefix));

        RSpaceStore {
            history,
            roots,
            cold,
        }
    }
}
