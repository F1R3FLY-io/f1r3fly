// See block-storage/src/main/scala/coop/rchain/blockstorage/dag/BlockMetadataStore.scala

use std::collections::{BTreeMap, HashMap, HashSet};

use models::rust::block_hash::BlockHashWrapper;
use models::rust::block_metadata::BlockMetadata;
use models::rust::casper::pretty_printer::PrettyPrinter;
use shared::rust::store::key_value_store::KvStoreError;
use shared::rust::store::key_value_typed_store::KeyValueTypedStore;
use shared::rust::store::key_value_typed_store_impl::KeyValueTypedStoreImpl;

pub struct BlockMetadataStore {
    store: KeyValueTypedStoreImpl<BlockHashWrapper, BlockMetadata>,
    dag_state: DagState,
}

#[derive(Default)]
struct DagState {
    dag_set: HashSet<BlockHashWrapper>,
    child_map: HashMap<BlockHashWrapper, HashSet<BlockHashWrapper>>,
    height_map: BTreeMap<i64, HashSet<BlockHashWrapper>>,
    // In general - at least genesis should be LFB.
    // But dagstate can be empty, as it is initialized before genesis is inserted.
    // Also lots of tests do not have genesis properly initialised, so fixing all this is pain.
    // So this is Option.
    last_finalized_block: Option<(BlockHashWrapper, i64)>,
    finalized_block_set: HashSet<BlockHashWrapper>,
}

struct BlockInfo {
    hash: BlockHashWrapper,
    parents: HashSet<BlockHashWrapper>,
    block_num: i64,
    is_invalid: bool,
    is_directly_finalized: bool,
    is_finalized: bool,
}

impl BlockMetadataStore {
    pub fn new(
        block_metadata_store: KeyValueTypedStoreImpl<BlockHashWrapper, BlockMetadata>,
    ) -> Self {
        let blocks_info_result = block_metadata_store
            .collect(|(hash, metadata)| {
                Some((hash.clone(), Self::block_metadata_to_info(&hash, metadata)))
            })
            .expect("Failed to collect block metadata");

        let blocks_info_map = blocks_info_result.into_iter().collect::<HashMap<_, _>>();
        let dag_state = Self::recreate_in_memory_state(blocks_info_map);

        Self {
            store: block_metadata_store,
            dag_state,
        }
    }

    fn block_metadata_to_info(
        hash: &BlockHashWrapper,
        block_metadata: &BlockMetadata,
    ) -> BlockInfo {
        let mut parents = HashSet::with_capacity(block_metadata.parents.len());
        parents.extend(block_metadata.parents.iter().cloned());

        BlockInfo {
            hash: hash.clone(),
            parents,
            block_num: block_metadata.block_number,
            is_invalid: block_metadata.invalid,
            is_directly_finalized: block_metadata.directly_finalized,
            is_finalized: block_metadata.finalized,
        }
    }

    pub fn add(&mut self, block_metadata: BlockMetadata) -> Result<(), KvStoreError> {
        let block_hash = block_metadata.block_hash.clone();
        let block_info = Self::block_metadata_to_info(&block_hash, &block_metadata);
        let current_state = std::mem::take(&mut self.dag_state);

        self.dag_state =
            Self::validate_dag_state(Self::add_block_to_dag_state(current_state, block_info));

        // Update persistent block metadata store
        self.store.put_one(block_hash, block_metadata)?;

        Ok(())
    }

    /** Record new last finalized lock. Directly finalized is the output of finalizer,
     * indirectly finalized are new LFB ancestors. */
    pub fn record_finalized(
        &mut self,
        directly: BlockHashWrapper,
        indirectly: HashSet<BlockHashWrapper>,
    ) -> Result<(), KvStoreError> {
        // read current values
        let cur_meta_for_df = self.store.get_unsafe(&directly)?;
        let cur_metas_for_if = self
            .store
            .get_batch(&indirectly.clone().into_iter().collect())?;

        // new values to persist
        let mut new_meta_for_df = cur_meta_for_df.clone();
        new_meta_for_df.finalized = true;
        new_meta_for_df.directly_finalized = true;

        let new_metas_for_if: Vec<(BlockHashWrapper, BlockMetadata)> = cur_metas_for_if
            .into_iter()
            .map(|mut v| {
                v.finalized = true;
                (v.block_hash.clone(), v)
            })
            .collect();

        let mut current_state = std::mem::take(&mut self.dag_state);

        current_state.finalized_block_set.extend(indirectly);
        current_state.finalized_block_set.insert(directly.clone());

        // update lastFinalizedBlock only when current one is lower
        if current_state.last_finalized_block.is_none()
            || current_state.last_finalized_block.as_ref().unwrap().1
                <= cur_meta_for_df.block_number
        {
            current_state.last_finalized_block =
                Some((directly.clone(), cur_meta_for_df.block_number));
        }

        self.dag_state = current_state;

        // persist new values all at once
        let mut new_values = Vec::with_capacity(1 + new_metas_for_if.len());
        new_values.push((directly, new_meta_for_df));
        new_values.extend(new_metas_for_if);
        self.store.put(new_values)?;

        Ok(())
    }

    pub fn get(&self, hash: &BlockHashWrapper) -> Result<Option<BlockMetadata>, KvStoreError> {
        self.store.get_one(hash)
    }

    pub fn get_unsafe(&self, hash: &BlockHashWrapper) -> Result<BlockMetadata, KvStoreError> {
        self.get(hash)?.ok_or(KvStoreError::KeyNotFound(format!(
            "BlockMetadataStore is missing key {}",
            PrettyPrinter::build_string_bytes(&hash.into_bytes().to_vec())
        )))
    }

    // DAG state operations

    pub fn dag_set(&self) -> HashSet<BlockHashWrapper> {
        self.dag_state.dag_set.clone()
    }

    pub fn contains(&self, hash: &BlockHashWrapper) -> bool {
        self.dag_state.dag_set.contains(hash)
    }

    pub fn child_map(&self) -> HashMap<BlockHashWrapper, HashSet<BlockHashWrapper>> {
        self.dag_state.child_map.clone()
    }

    pub fn height_map(&self) -> BTreeMap<i64, HashSet<BlockHashWrapper>> {
        self.dag_state.height_map.clone()
    }

    pub fn last_finalized_block(&self) -> BlockHashWrapper {
        self.dag_state
            .last_finalized_block
            .as_ref()
            .expect("DagState does not contain lastFinalizedBlock. Are you calling this on empty BlockDagStorage? Otherwise there is a bug.")
            .0
            .clone()
    }

    pub fn finalized_block_set(&self) -> HashSet<BlockHashWrapper> {
        self.dag_state.finalized_block_set.clone()
    }

    fn add_block_to_dag_state(mut state: DagState, block_info: BlockInfo) -> DagState {
        let hash = block_info.hash.clone();

        // Update dag set / all blocks in the DAG
        state.dag_set.insert(hash.clone());

        // Update children relation map
        state
            .child_map
            .entry(hash.clone())
            .or_insert_with(HashSet::new);

        for parent in &block_info.parents {
            state
                .child_map
                .entry(parent.clone())
                .or_insert_with(HashSet::new)
                .insert(hash.clone());
        }

        // Update height map
        if !block_info.is_invalid {
            state
                .height_map
                .entry(block_info.block_num)
                .or_insert_with(HashSet::new)
                .insert(hash.clone());
        }

        if block_info.is_directly_finalized
            && state
                .last_finalized_block
                .as_ref()
                .map_or(true, |&(_, height)| height <= block_info.block_num)
        {
            state.last_finalized_block = Some((hash.clone(), block_info.block_num));
        }

        if block_info.is_finalized {
            state.finalized_block_set.insert(block_info.hash);
        }

        state
    }

    fn validate_dag_state(dag_state: DagState) -> DagState {
        // Validate height map index (block numbers) are in sequence without holes
        let (min, max) = if !dag_state.height_map.is_empty() {
            (
                *dag_state.height_map.first_key_value().unwrap().0,
                *dag_state.height_map.last_key_value().unwrap().0 + 1,
            )
        } else {
            (0, 0)
        };
        assert!(
            max - min == dag_state.height_map.len() as i64,
            "DAG store height map has numbers not in sequence."
        );
        dag_state
    }

    fn recreate_in_memory_state(blocks_info_map: HashMap<BlockHashWrapper, BlockInfo>) -> DagState {
        let empty_state = DagState::default();

        // Add blocks to DAG state
        let dag_state = blocks_info_map
            .into_iter()
            .fold(empty_state, |state, (_, block_info)| {
                Self::add_block_to_dag_state(state, block_info)
            });

        Self::validate_dag_state(dag_state)
    }
}
