// See block-storage/src/main/scala/coop/rchain/blockstorage/dag/BlockDagKeyValueStorage.scala

use std::collections::{BTreeMap, HashMap, HashSet};

use models::rust::{block_hash::BlockHash, block_metadata::BlockMetadata, validator::Validator};
use shared::rust::store::{
    key_value_store::KvStoreError, key_value_typed_store_impl::KeyValueTypedStoreImpl,
};

use super::{
    block_metadata_store::BlockMetadataStore, equivocation_tracker_store::EquivocationTrackerStore,
};

type DeployId = shared::rust::ByteString;

struct KeyValueDagRepresentation {
    dag_set: HashSet<BlockHash>,
    latest_messages_map: HashMap<Validator, BlockHash>,
    child_map: HashMap<BlockHash, HashSet<BlockHash>>,
    height_map: BTreeMap<i64, HashSet<BlockHash>>,
    invalid_blocks_set: HashSet<BlockMetadata>,
    last_finalized_block_hash: BlockHash,
    finalized_blocks_set: HashSet<BlockHash>,
    block_metadata_index: BlockMetadataStore,
    deploy_index: KeyValueTypedStoreImpl<DeployId, BlockHash>,
}

impl KeyValueDagRepresentation {
    fn lookup(&self, block_hash: &BlockHash) -> Result<Option<BlockMetadata>, KvStoreError> {
        if self.dag_set.contains(block_hash) {
            self.block_metadata_index.get(block_hash)
        } else {
            Ok(None)
        }
    }
}

pub struct BlockDagKeyValueStorage {
    latest_messages_index: KeyValueTypedStoreImpl<Validator, BlockHash>,
    block_metadata_index: BlockMetadataStore,
    deploy_index: KeyValueTypedStoreImpl<DeployId, BlockHash>,
    invalid_blocks_index: KeyValueTypedStoreImpl<BlockHash, BlockMetadata>,
    equivocation_tracker_index: EquivocationTrackerStore,
}
