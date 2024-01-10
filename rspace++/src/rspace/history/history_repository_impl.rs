use crate::rspace::history::cold_store::PersistedData;
use crate::rspace::history::history::History;
use crate::rspace::history::history_repository::HistoryRepository;
use crate::rspace::history::root_repository::RootRepository;
use crate::rspace::history::roots_store::RootsStore;
use crate::rspace::shared::key_value_typed_store::KeyValueTypedStore;
use crate::rspace::state::rspace_exporter::RSpaceExporter;
use crate::rspace::state::rspace_importer::RSpaceImporter;
use std::marker::PhantomData;

// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryRepositoryImpl.scala
pub struct HistoryRepositoryImpl<
    C,
    P,
    A,
    K,
    H: History,
    R: RootsStore,
    T: KeyValueTypedStore<blake3::Hash, PersistedData>,
    E: RSpaceExporter,
    I: RSpaceImporter,
> {
    pub current_history: H,
    pub roots_repository: RootRepository<R>,
    pub leaf_store: T,
    pub rspace_exporter: E,
    pub rspace_importer: I,
    pub _marker: PhantomData<(C, P, A, K)>,
}

impl<
        C,
        P,
        A,
        K,
        H: History,
        R: RootsStore,
        T: KeyValueTypedStore<blake3::Hash, PersistedData>,
        E: RSpaceExporter,
        I: RSpaceImporter,
    > HistoryRepository<C, P, A, K> for HistoryRepositoryImpl<C, P, A, K, H, R, T, E, I>
{
    fn checkpoint(
        &self,
        actions: Vec<crate::rspace::hot_store_action::HotStoreAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn do_checkpoint(
        &self,
        actions: Vec<crate::rspace::hot_store_trie_action::HotStoreTrieAction<C, P, A, K>>,
    ) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn reset(&self, root: blake3::Hash) -> Box<dyn HistoryRepository<C, P, A, K>> {
        todo!()
    }

    fn history(&self) -> Box<dyn History> {
        todo!()
    }

    fn exporter(
        &self,
    ) -> Box<
        dyn RSpaceExporter<
            KeyHash = blake3::Hash,
            NodePath = Vec<(blake3::Hash, Option<u8>)>,
            Value = bytes::Bytes,
        >,
    > {
        todo!()
    }

    fn importer(&self) -> Box<dyn RSpaceImporter<KeyHash = blake3::Hash, Value = bytes::Bytes>> {
        todo!()
    }

    fn get_history_reader(
        &self,
        state_hash: blake3::Hash,
    ) -> Box<dyn super::history_reader::HistoryReader<blake3::Hash, C, P, A, K>> {
        todo!()
    }

    fn root(&self) -> blake3::Hash {
        todo!()
    }
}
