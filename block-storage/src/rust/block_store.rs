// See block-storage/src/main/scala/coop/rchain/blockstorage/BlockStore.scala

use std::sync::{Arc, Mutex};

use models::rust::{
  block_hash::BlockHash,
  casper::protocol::casper_message::{ApprovedBlock, BlockMessage}
};

use crate::rust::key_value_block_store::KeyValueBlockStore;

// Re-export from shared for convenience
pub use shared::rust::store::key_value_store::KvStoreError;

/// Equivalent to Scala's BlockStore[F[_]] trait
pub trait BlockStore {
  async fn get(&self, block_hash: &BlockHash) -> Result<Option<BlockMessage>, KvStoreError>;

  async fn put(&mut self, block_hash: BlockHash, block_message: BlockMessage) -> Result<(), KvStoreError>;

  async fn contains(&self, block_hash: &BlockHash) -> Result<bool, KvStoreError> {
    // Equivalent to Scala's: get(blockHash).map(_.isDefined)
    Ok(self.get(block_hash).await?.is_some())
  }

  async fn get_approved_block(&self) -> Result<Option<ApprovedBlock>, KvStoreError>;

  async fn put_approved_block(&mut self, block: ApprovedBlock) -> Result<(), KvStoreError>;

  // Default implementations - equivalent to Scala's default methods
  async fn put_block_message(&mut self, block_message: &BlockMessage) -> Result<(), KvStoreError> {
    // Equivalent to Scala's: put((blockMessage.blockHash, blockMessage))
    self.put(block_message.block_hash.clone(), block_message.clone()).await
  }
}
