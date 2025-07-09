// See casper/src/main/scala/coop/rchain/casper/blocks/BlockProcessor.scala

/*
 * ARCHITECTURAL CHOICE: Trait-based Dependency Injection
 *
 * This implementation uses trait-based dependency injection instead of functional closures
 * because Rust's ownership model and async system work better with traits than with complex
 * closure captures. Traits provide zero-cost abstractions, better testability, and seamless
 * async support while maintaining the same flexibility as the original Scala version.
 */

use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use block_storage::rust::{
    block_store::BlockStore,
    casperbuffer::casper_buffer_key_value_storage::CasperBufferKeyValueStorage,
    dag::block_dag_key_value_storage::KeyValueDagRepresentation,
    key_value_block_store::KeyValueBlockStore,
};
use comm::rust::{
    rp::{connect::ConnectionsCell, rp_conf::RPConf},
    transport::transport_layer::TransportLayer,
};
use models::rust::{
    block_hash::{BlockHash, BlockHashSerde},
    casper::pretty_printer::PrettyPrinter,
    casper::protocol::casper_message::BlockMessage,
};
use rspace_plus_plus::rspace::history::Either;

use crate::rust::block_status::BlockError;
use crate::rust::{
    block_status::InvalidBlock,
    casper::{Casper, CasperSnapshot},
    errors::CasperError,
    util::proto_util,
    validate::Validate,
    ValidBlockProcessing,
};
use crate::rust::engine::block_retriever::AdmitHashReason;

fn to_block_hash_serde(hash: &BlockHash) -> BlockHashSerde {
    BlockHashSerde(hash.clone())
}

/// Helper function to convert HashSet<BlockHash> to format suitable for PrettyPrinter
fn format_block_hashes(hashes: &HashSet<BlockHash>) -> String {
    let contents: Vec<String> = hashes
        .iter()
        .map(|hash| PrettyPrinter::build_string_bytes(hash))
        .collect();
    format!("[{}]", contents.join(" "))
}

/// Equivalent to Scala's: storeBlock: BlockMessage => F[Unit]
pub trait BlockStorage: BlockStore {
    async fn store_block(&mut self, block: &BlockMessage) -> Result<(), CasperError>;
}

/// Equivalent to Scala's: getNonValidatedDependencies: (Casper[F], BlockMessage) => F[(Boolean, Set[BlockHash], Set[BlockHash])]
pub trait DependencyAnalyzer {
  async fn get_non_validated_dependencies(
    &self,
    casper: &mut impl Casper,
    block: &BlockMessage,
  ) -> Result<(bool, std::collections::HashSet<BlockHash>, std::collections::HashSet<BlockHash>), CasperError>;
}

/// Equivalent to Scala's: getCasperStateSnapshot: Casper[F] => F[CasperSnapshot[F]]
pub trait CasperSnapshotProvider {
    async fn get_casper_state_snapshot(
        &self,
        casper: &mut impl Casper,
    ) -> Result<CasperSnapshot, CasperError>;
}

/// Equivalent to Scala's: commitToBuffer: (BlockMessage, Option[Set[BlockHash]]) => F[Unit]
/// and removeFromBuffer: BlockMessage => F[Unit]
pub trait BufferManager {
  async fn commit_to_buffer(
    &mut self,
    block: &BlockMessage,
    deps: Option<HashSet<BlockHash>>,
  ) -> Result<(), CasperError>;

  async fn remove_from_buffer(&mut self, block: &BlockMessage) -> Result<(), CasperError>;
}

/// Equivalent to Scala's: requestMissingDependencies: Set[BlockHash] => F[Unit]
pub trait DependencyRequester {
  async fn request_missing_dependencies(
    &mut self,
    deps: &HashSet<BlockHash>,
  ) -> Result<(), CasperError>;
}

/// Equivalent to Scala's: ackProcessed: BlockMessage => F[Unit]
pub trait ProcessAcknowledger {
    async fn ack_processed(&mut self, block: &BlockMessage) -> Result<(), CasperError>;
}

/// Equivalent to Scala's: validateBlock: (Casper[F], CasperSnapshot[F], BlockMessage) => F[ValidBlockProcessing]
pub trait BlockValidator {
    async fn validate_block(
        &self,
        casper: &mut impl Casper,
        snapshot: &CasperSnapshot,
        block: &BlockMessage,
    ) -> Result<ValidBlockProcessing, CasperError>;
}

/// Equivalent to Scala's: effectsForValidBlock and effectsForInvalidBlock
pub trait EffectHandler {
    async fn effects_for_valid_block(
        &mut self,
        casper: &mut impl Casper,
        block: &BlockMessage,
    ) -> Result<KeyValueDagRepresentation, CasperError>;

    async fn effects_for_invalid_block(
        &mut self,
        casper: &mut impl Casper,
        block: &BlockMessage,
        invalid_block: &InvalidBlock,
        snapshot: &CasperSnapshot,
    ) -> Result<KeyValueDagRepresentation, CasperError>;
}

/**
 * Logic for processing incoming blocks
 * Blocks created by node itself are not held here, but in Proposer.
 *
 * This implementation uses 8 traits for dependency injection instead of
 * functional closures (see architectural explanation above).
 *
 * Generic parameters represent different implementations:
 * - S: BlockStorage - handles block persistence
 * - D: DependencyAnalyzer - analyzes block dependencies
 * - C: CasperSnapshotProvider - provides Casper state snapshots
 * - B: BufferManager - manages block buffering
 * - R: DependencyRequester - requests missing dependencies
 * - A: ProcessAcknowledger - acknowledges block processing
 * - V: BlockValidator - validates blocks
 * - E: EffectHandler - handles side effects (valid/invalid blocks)
 */
pub struct BlockProcessor<S, D, C, B, R, A, V, E>
where
    S: BlockStorage,
    D: DependencyAnalyzer,
    C: CasperSnapshotProvider,
    B: BufferManager,
    R: DependencyRequester,
    A: ProcessAcknowledger,
    V: BlockValidator,
    E: EffectHandler,
{
    block_storage: S,
    dependency_analyzer: D,
    casper_snapshot_provider: C,
    buffer_manager: B,
    dependency_requester: R,
    process_acknowledger: A,
    block_validator: V,
    effect_handler: E,
}

impl<S, D, C, B, R, A, V, E> BlockProcessor<S, D, C, B, R, A, V, E>
where
    S: BlockStorage,
    D: DependencyAnalyzer,
    C: CasperSnapshotProvider,
    B: BufferManager,
    R: DependencyRequester,
    A: ProcessAcknowledger,
    V: BlockValidator,
    E: EffectHandler,
{
    pub fn new(
        block_storage: S,
        dependency_analyzer: D,
        casper_snapshot_provider: C,
        buffer_manager: B,
        dependency_requester: R,
        process_acknowledger: A,
        block_validator: V,
        effect_handler: E,
    ) -> Self {
        Self {
            block_storage,
            dependency_analyzer,
            casper_snapshot_provider,
            buffer_manager,
            dependency_requester,
            process_acknowledger,
            block_validator,
            effect_handler,
        }
    }

    /// check if block should be processed
    pub async fn check_if_of_interest(
        &self,
        casper: &mut impl Casper,
        block: &BlockMessage,
    ) -> Result<bool, CasperError> {
        // TODO casper.dag_contains does not take into account equivocation tracker
        let already_processed =
            casper.dag_contains(&block.block_hash) || casper.buffer_contains(&block.block_hash);

        // Port Scala monadic operations using Result::map and Result::and_then
        let shard_of_interest = casper.get_approved_block().map(|approved_block| {
            approved_block
                .shard_id
                .eq_ignore_ascii_case(&block.shard_id)
        })?;

        let version_of_interest = casper
            .get_approved_block()
            .map(|approved_block| Validate::version(block, approved_block.header.version))?;

        let old_block = casper.get_approved_block().map(|approved_block| {
            proto_util::block_number(block) < proto_util::block_number(approved_block)
        })?;

        Ok(!already_processed && shard_of_interest && version_of_interest && !old_block)
    }

    /// check block format and store if check passed
    pub async fn check_if_well_formed_and_store(
        &mut self,
        block: &BlockMessage,
    ) -> Result<bool, CasperError> {
        let valid_format = Validate::format_of_fields(block);
        let valid_sig = Validate::block_signature(block);
        let is_valid = valid_format && valid_sig;

        // Equivalent to Scala's `storeBlock(b).whenA(isValid)`
        // whenA means "when condition is true, execute action"
        if is_valid {
            self.block_storage.store_block(block).await?;
        }

        Ok(is_valid)
    }

    /// check if block has all dependencies available and can be validated
    pub async fn check_dependencies_with_effects(
        &mut self,
        casper: &mut impl Casper,
        block: &BlockMessage,
    ) -> Result<bool, CasperError> {
        let (is_ready, deps_to_fetch, deps_in_buffer) = self
            .dependency_analyzer
            .get_non_validated_dependencies(casper, block)
            .await?;

        if is_ready {
            // store pendant block in buffer, it will be removed once block is validated and added to DAG
            self.buffer_manager.commit_to_buffer(block, None).await?;
        } else {
            // associate parents with new block in casper buffer
            let mut all_deps = deps_to_fetch.clone();
            all_deps.extend(deps_in_buffer.clone());
            self.buffer_manager
                .commit_to_buffer(block, Some(all_deps))
                .await?;
            self.dependency_requester
                .request_missing_dependencies(&deps_to_fetch)
                .await?;
            self.process_acknowledger.ack_processed(block).await?;
        }

        Ok(is_ready)
    }

    /// validate block and invoke all effects required
    pub async fn validate_with_effects(
        &mut self,
        casper: &mut impl Casper,
        block: &BlockMessage,
        // this option is required for tests, as sometimes block without parents available are added, so
        // CasperSnapshot cannot be constructed
        snapshot_opt: Option<CasperSnapshot>,
    ) -> Result<ValidBlockProcessing, CasperError> {
        let snapshot = match snapshot_opt {
            Some(snapshot) => snapshot,
            None => {
                self.casper_snapshot_provider
                    .get_casper_state_snapshot(casper)
                    .await?
            }
        };

        let status = self
            .block_validator
            .validate_block(casper, &snapshot, block)
            .await?;

        let _ = match &status {
            Either::Right(_valid_block) => {
                self.effect_handler
                    .effects_for_valid_block(casper, block)
                    .await
            }
            Either::Left(invalid_block) => {
                // this is to maintain backward compatibility with casper validate method.
                // as it returns not only InvalidBlock or ValidBlock
                match invalid_block {
                    BlockError::Invalid(i) => {
                        self.effect_handler
                            .effects_for_invalid_block(casper, block, i, &snapshot)
                            .await
                    }
                    _ => {
                        // this should never happen
                        Ok(snapshot.dag.clone())
                    }
                }
            }
        }?;

        // once block is validated and effects are invoked, it should be removed from buffer
        self.buffer_manager.remove_from_buffer(block).await?;
        self.process_acknowledger.ack_processed(block).await?;

        Ok(status)
    }
}

// COMPANION OBJECT IMPLEMENTATIONS
// TODO should be double checked
/*
I’m not sure about this part, but I’m stuck on the functional approach because I can’t properly use async functions from a sync context
and then use a trait-based approach like we did before (for example, proposer.rs with the async fn get_snapshot from Casper).
 */

// Equivalent to Scala's: storeBlock = (b: BlockMessage) => BlockStore[F].put(b)
pub struct BlockStoreWrapper(Arc<Mutex<KeyValueBlockStore>>);

impl BlockStoreWrapper {
    pub fn new(store: Arc<Mutex<KeyValueBlockStore>>) -> Self {
        Self(store)
    }
}

impl BlockStorage for BlockStoreWrapper {
    async fn store_block(&mut self, block: &BlockMessage) -> Result<(), CasperError> {
        let mut store = self
            .0
            .lock()
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;
        store
            .put_block_message(block)
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;
        Ok(())
    }
}

impl BlockStore for BlockStoreWrapper {
    async fn get(
        &self,
        block_hash: &BlockHash,
    ) -> Result<Option<BlockMessage>, shared::rust::store::key_value_store::KvStoreError> {
        let store = self.0.lock().map_err(|e| {
            shared::rust::store::key_value_store::KvStoreError::LockError(e.to_string())
        })?;
        store.get(block_hash)
    }

    async fn put(
        &mut self,
        block_hash: BlockHash,
        block_message: BlockMessage,
    ) -> Result<(), shared::rust::store::key_value_store::KvStoreError> {
        let mut store = self.0.lock().map_err(|e| {
            shared::rust::store::key_value_store::KvStoreError::LockError(e.to_string())
        })?;
        store.put(block_hash, &block_message)
    }

    async fn get_approved_block(
        &self,
    ) -> Result<
        Option<models::rust::casper::protocol::casper_message::ApprovedBlock>,
        shared::rust::store::key_value_store::KvStoreError,
    > {
        let store = self.0.lock().map_err(|e| {
            shared::rust::store::key_value_store::KvStoreError::LockError(e.to_string())
        })?;
        store.get_approved_block()
    }

    async fn put_approved_block(
        &mut self,
        block: models::rust::casper::protocol::casper_message::ApprovedBlock,
    ) -> Result<(), shared::rust::store::key_value_store::KvStoreError> {
        let mut store = self.0.lock().map_err(|e| {
            shared::rust::store::key_value_store::KvStoreError::LockError(e.to_string())
        })?;
        store.put_approved_block(block)
    }
}

// Equivalent to Scala's: getCasperStateSnapshot = (c: Casper[F]) => c.getSnapshot
pub struct GetCasperStateSnapshot;

impl CasperSnapshotProvider for GetCasperStateSnapshot {
    async fn get_casper_state_snapshot(
        &self,
        casper: &mut impl Casper,
    ) -> Result<CasperSnapshot, CasperError> {
        casper.get_snapshot().await
    }
}

// Equivalent to Scala's: getNonValidatedDependencies = (c: Casper[F], b: BlockMessage) => { ... }
// This wrapper provides access to both CasperBuffer and BlockDagStorage like Scala's dependency injection
pub struct CasperDependencyAnalyzer {
    casper_buffer: Arc<Mutex<CasperBufferKeyValueStorage>>,
    block_dag_storage: Arc<Mutex<block_storage::rust::dag::block_dag_key_value_storage::BlockDagKeyValueStorage>>,
}

impl CasperDependencyAnalyzer {
    pub fn new(
        casper_buffer: Arc<Mutex<CasperBufferKeyValueStorage>>,
        block_dag_storage: Arc<Mutex<block_storage::rust::dag::block_dag_key_value_storage::BlockDagKeyValueStorage>>,
    ) -> Self {
        Self {
            casper_buffer,
            block_dag_storage,
        }
    }
}

// Keep CasperBufferWrapper for other traits that only need buffer access
pub struct CasperBufferWrapper {
    buffer: Arc<Mutex<CasperBufferKeyValueStorage>>,
}

impl CasperBufferWrapper {
    pub fn new(buffer: Arc<Mutex<CasperBufferKeyValueStorage>>) -> Self {
        Self { buffer }
    }
}

impl DependencyAnalyzer for CasperDependencyAnalyzer {
    async fn get_non_validated_dependencies(
        &self,
        casper: &mut impl Casper,
        block: &BlockMessage,
    ) -> Result<(bool, HashSet<BlockHash>, HashSet<BlockHash>), CasperError> {
        let all_deps = proto_util::dependencies_hashes_of(block);

        let equivocation_hashes: Vec<BlockHash> = {
            let block_dag_storage = self
                .block_dag_storage
                .lock()
                .map_err(|e| CasperError::RuntimeError(e.to_string()))?;
            
            block_dag_storage.access_equivocations_tracker(|tracker| {
                let equivocation_records = tracker.data()?;
                // Equivalent to Scala's: equivocations.flatMap(_.equivocationDetectedBlockHashes)
                let hashes: Vec<BlockHash> = equivocation_records
                    .iter()
                    .flat_map(|record| record.equivocation_detected_block_hashes.iter())
                    .cloned()
                    .collect();
                Ok(hashes)
            }).map_err(|e| CasperError::RuntimeError(e.to_string()))?
        };

        let deps_in_buffer: Vec<BlockHash> = {
            let casper_buffer = self
                .casper_buffer
                .lock()
                .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

            all_deps
                .iter()
                .filter_map(|dep| {
                    let block_hash_serde = BlockHashSerde(dep.clone());
                    if casper_buffer.contains(&block_hash_serde)
                        || casper_buffer.is_pendant(&block_hash_serde)
                    {
                        Some(dep.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        let deps_in_dag: Vec<BlockHash> = all_deps
            .iter()
            .filter_map(|dep| {
                if casper.dag_contains(dep) {
                    Some(dep.clone())
                } else {
                    None
                }
            })
            .collect();

        let deps_in_eq_tracker: Vec<BlockHash> = all_deps
            .iter()
            .filter(|&dep| equivocation_hashes.contains(dep))
            .cloned()
            .collect();

        let mut deps_validated: Vec<BlockHash> = deps_in_dag.clone();
        deps_validated.extend(deps_in_eq_tracker.iter().cloned());

        let deps_to_fetch: Vec<BlockHash> = all_deps
            .iter()
            .filter(|&dep| !deps_in_buffer.contains(dep))
            .filter(|&dep| !deps_validated.contains(dep))
            .cloned()
            .collect();

        let ready = deps_to_fetch.is_empty() && deps_in_buffer.is_empty();

        if !ready {
            log::info!(
                "Block {} missing dependencies. To fetch: {}. In buffer: {}. Validated: {}.",
                PrettyPrinter::build_string_block_message(block, true),
                format_block_hashes(&deps_to_fetch.iter().cloned().collect()),
                format_block_hashes(&deps_in_buffer.iter().cloned().collect()),
                format_block_hashes(&deps_validated.iter().cloned().collect())
            );
        }

        Ok((
            ready,
            deps_to_fetch.into_iter().collect::<HashSet<BlockHash>>(),
            deps_in_buffer.into_iter().collect::<HashSet<BlockHash>>(),
        ))
    }
}

// Equivalent to Scala's: commitToBuffer and removeFromBuffer
// Use same wrapper for BufferManager functionality
impl BufferManager for CasperBufferWrapper {
    async fn commit_to_buffer(
        &mut self,
        block: &BlockMessage,
        deps: Option<HashSet<BlockHash>>,
    ) -> Result<(), CasperError> {
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        match deps {
            None => {
                let block_hash_serde = BlockHashSerde(block.block_hash.clone());
                buffer
                    .put_pendant(block_hash_serde)
                    .map_err(|e| CasperError::RuntimeError(e.to_string()))?;
            }
            Some(dependencies) => {
                // Equivalent to Scala's: d.toList.traverse_(h => casperBuffer.addRelation(h, b.blockHash))
                for dep in dependencies {
                    let dep_serde = BlockHashSerde(dep.clone());
                    let block_hash_serde = BlockHashSerde(block.block_hash.clone());
                    buffer
                        .add_relation(dep_serde, block_hash_serde)
                        .map_err(|e| CasperError::RuntimeError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    async fn remove_from_buffer(&mut self, block: &BlockMessage) -> Result<(), CasperError> {
        // Equivalent to Scala's: casperBuffer.remove(b.blockHash)
        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        let block_hash_serde = BlockHashSerde(block.block_hash.clone());
        buffer
            .remove(block_hash_serde)
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        Ok(())
    }
}

// Equivalent to Scala's: requestMissingDependencies = (deps: Set[BlockHash]) => { ... }
pub struct RequestMissingDependencies<T: TransportLayer + Send + Sync> {
    block_retriever: Arc<Mutex<crate::rust::engine::block_retriever::BlockRetriever<T>>>,
}

impl<T: TransportLayer + Send + Sync> RequestMissingDependencies<T> {
    pub fn new(
        block_retriever: Arc<Mutex<crate::rust::engine::block_retriever::BlockRetriever<T>>>,
    ) -> Self {
        Self { block_retriever }
    }
}

impl<T: TransportLayer + Send + Sync> DependencyRequester for RequestMissingDependencies<T> {
    async fn request_missing_dependencies(
        &mut self,
        deps: &HashSet<BlockHash>,
    ) -> Result<(), CasperError> {
        let retriever = self
            .block_retriever
            .lock()
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        for dep in deps {
            retriever.admit_hash(dep.clone(), None, AdmitHashReason::MissingDependencyRequested)
        .await
        .map_err(|e| CasperError::RuntimeError(e.to_string()))?;
        }

        Ok(())
    }
}

// Equivalent to Scala's: validateBlock = (c: Casper[F], s: CasperSnapshot[F], b: BlockMessage) => c.validate(b, s)
// Simplified: unit struct for stateless validation
pub struct ValidateBlock;

impl BlockValidator for ValidateBlock {
    async fn validate_block(
        &self,
        casper: &mut impl Casper,
        snapshot: &CasperSnapshot,
        block: &BlockMessage,
    ) -> Result<ValidBlockProcessing, CasperError> {
        casper.validate(block, snapshot)
    }
}

// Equivalent to Scala's: ackProcessed = (b: BlockMessage) => BlockRetriever[F].ackInCasper(b.blockHash)
pub struct AckProcessed<T: TransportLayer + Send + Sync> {
    block_retriever: Arc<Mutex<crate::rust::engine::block_retriever::BlockRetriever<T>>>,
}

impl<T: TransportLayer + Send + Sync> AckProcessed<T> {
    pub fn new(
        block_retriever: Arc<Mutex<crate::rust::engine::block_retriever::BlockRetriever<T>>>,
    ) -> Self {
        Self { block_retriever }
    }
}

impl<T: TransportLayer + Send + Sync> ProcessAcknowledger for AckProcessed<T> {
    async fn ack_processed(&mut self, block: &BlockMessage) -> Result<(), CasperError> {
        let retriever = self
            .block_retriever
            .lock()
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        retriever
            .ack_in_casper(block.block_hash.clone())
            .await
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        Ok(())
    }
}

// Equivalent to Scala's: effectsForInvalidBlock and effectsForValidBlock
pub struct EffectsForBlocks<T: TransportLayer + Send + Sync> {
    transport: Arc<T>,
    connections_cell: ConnectionsCell,
    conf: RPConf,
}

impl<T: TransportLayer + Send + Sync> EffectsForBlocks<T> {
    pub fn new(transport: Arc<T>, connections_cell: ConnectionsCell, conf: RPConf) -> Self {
        Self {
            transport,
            connections_cell,
            conf,
        }
    }
}

impl<T: TransportLayer + Send + Sync> EffectHandler for EffectsForBlocks<T> {
    async fn effects_for_valid_block(
        &mut self,
        casper: &mut impl Casper,
        block: &BlockMessage,
    ) -> Result<KeyValueDagRepresentation, CasperError> {
        let dag = casper.handle_valid_block(block).await?;

        // Equivalent to Scala's: CommUtil[F].sendBlockHash(b.blockHash, b.sender)
        self.transport
            .send_block_hash(
                &self.connections_cell,
                &self.conf,
                &block.block_hash,
                &block.sender,
            )
            .await
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        Ok(dag)
    }

    async fn effects_for_invalid_block(
        &mut self,
        casper: &mut impl Casper,
        block: &BlockMessage,
        invalid_block: &InvalidBlock,
        snapshot: &CasperSnapshot,
    ) -> Result<KeyValueDagRepresentation, CasperError> {
        let dag = casper.handle_invalid_block(block, invalid_block, &snapshot.dag)?;

        // Equivalent to Scala's: CommUtil[F].sendBlockHash(b.blockHash, b.sender)
        self.transport
            .send_block_hash(
                &self.connections_cell,
                &self.conf,
                &block.block_hash,
                &block.sender,
            )
            .await
            .map_err(|e| CasperError::RuntimeError(e.to_string()))?;

        Ok(dag)
    }
}

// Type alias for convenience - using simplified wrapper implementations
pub type StandardBlockProcessor<T> = BlockProcessor<
    BlockStoreWrapper,
    CasperDependencyAnalyzer,
    GetCasperStateSnapshot,
    CasperBufferWrapper,
    RequestMissingDependencies<T>,
    AckProcessed<T>,
    ValidateBlock,
    EffectsForBlocks<T>,
>;

// Constructor function equivalent to Scala's companion object apply method
// Simplified: use lightweight wrapper structs
pub fn new_block_processor<T: TransportLayer + Send + Sync>(
    block_store: Arc<Mutex<KeyValueBlockStore>>,
    casper_buffer: Arc<Mutex<CasperBufferKeyValueStorage>>,
    block_dag_storage: Arc<Mutex<block_storage::rust::dag::block_dag_key_value_storage::BlockDagKeyValueStorage>>,
    block_retriever: Arc<Mutex<crate::rust::engine::block_retriever::BlockRetriever<T>>>,
    transport: Arc<T>,
    connections_cell: ConnectionsCell,
    conf: RPConf,
) -> StandardBlockProcessor<T> {
    BlockProcessor::new(
        BlockStoreWrapper::new(block_store),
        CasperDependencyAnalyzer::new(casper_buffer.clone(), block_dag_storage),
        GetCasperStateSnapshot,
        CasperBufferWrapper::new(casper_buffer),
        RequestMissingDependencies::new(block_retriever.clone()),
        AckProcessed::new(block_retriever),
        ValidateBlock,
        EffectsForBlocks::new(transport, connections_cell, conf),
    )
}
