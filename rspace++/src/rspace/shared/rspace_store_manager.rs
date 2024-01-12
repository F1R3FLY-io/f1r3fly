use super::{key_value_store::KeyValueStore, key_value_store_manager::KeyValueStoreManager};
use crate::rspace::rspace::RSpaceStore;

// See rspace/src/main/scala/coop/rchain/rspace/store/RSpaceStoreManagerSyntax.scala
struct RSpaceStoreManagerOps;

impl RSpaceStoreManagerOps {
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

impl KeyValueStoreManager for RSpaceStoreManagerOps {
    fn store(&self, name: String) -> Box<dyn KeyValueStore> {
        todo!()
    }

    fn shutdown(&self) -> () {
        todo!()
    }
}
