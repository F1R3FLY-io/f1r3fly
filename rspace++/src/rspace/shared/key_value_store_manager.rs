use super::key_value_store::KeyValueStore;
use crate::rspace::rspace::RSpaceStore;
use async_trait::async_trait;

// See shared/src/main/scala/coop/rchain/store/KeyValueStoreManager.scala
#[async_trait]
pub trait KeyValueStoreManager: Send + Sync {
    async fn store(&self, name: String) -> Result<Box<dyn KeyValueStore>, KVSManagerError>;

    async fn shutdown(&self) -> ();

    async fn r_space_stores(&self) -> Result<RSpaceStore, KVSManagerError> {
        self.get_stores("rspace").await
    }

    async fn eval_stores(&self) -> Result<RSpaceStore, KVSManagerError> {
        self.get_stores("eval").await
    }

    // TODO: This function should be private
    async fn get_stores(&self, db_prefix: &str) -> Result<RSpaceStore, KVSManagerError> {
        let history = self.store(format!("{}-history", db_prefix)).await?;
        let roots = self.store(format!("{}-roots", db_prefix)).await?;
        let cold = self.store(format!("{}-cold", db_prefix)).await?;

        Ok(RSpaceStore {
            history,
            roots,
            cold,
        })
    }
}

#[derive(Debug)]
pub struct KVSManagerError {
    pub message: String,
}
