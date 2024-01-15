use super::key_value_store::KeyValueStore;
use crate::rspace::rspace::RSpaceStore;
use async_trait::async_trait;

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreManager.scala
#[async_trait]
pub trait KeyValueStoreManager {
    async fn store(&self, name: String) -> Box<dyn KeyValueStore>;

    fn shutdown(&self) -> ();

    async fn r_space_stores(&self) -> RSpaceStore {
        self.get_stores("rspace").await
    }

    async fn eval_stores(&self) -> RSpaceStore {
        self.get_stores("eval").await
    }

    // TODO: This function should be private
    async fn get_stores(&self, db_prefix: &str) -> RSpaceStore {
        let history = self.store(format!("{}-history", db_prefix)).await;
        let roots = self.store(format!("{}-roots", db_prefix)).await;
        let cold = self.store(format!("{}-cold", db_prefix)).await;

        RSpaceStore {
            history,
            roots,
            cold,
        }
    }
}
