use crate::rspace::{
    hashing::serializable_blake3_hash::SerializableBlake3Hash,
    history::{cold_store::PersistedData, history::History, history_reader::HistoryReader},
    shared::key_value_typed_store::KeyValueTypedStore,
};
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub struct RSpaceHistoryReaderImpl<C, P, A, K> {
    target_history: Box<dyn History>,
    leaf_store: Arc<Mutex<Box<dyn KeyValueTypedStore<SerializableBlake3Hash, PersistedData>>>>,
    _marker: PhantomData<(C, P, A, K)>,
}

impl<C, P, A, K> RSpaceHistoryReaderImpl<C, P, A, K> {
    pub fn new(
        target_history: Box<dyn History>,
        leaf_store: Arc<Mutex<Box<dyn KeyValueTypedStore<SerializableBlake3Hash, PersistedData>>>>,
    ) -> Self {
        RSpaceHistoryReaderImpl {
            target_history,
            leaf_store,
            _marker: PhantomData,
        }
    }
}

impl<Key, C, P, A, K> HistoryReader<Key, C, P, A, K> for RSpaceHistoryReaderImpl<C, P, A, K> {
    fn root(&self) -> Key {
        todo!()
    }

    fn get_data_proj(
        &self,
        key: Key,
        proj: fn(
            crate::rspace::internal::Datum<A>,
            bytes::Bytes,
        ) -> crate::rspace::internal::Datum<A>,
    ) -> Vec<crate::rspace::internal::Datum<A>> {
        todo!()
    }

    fn get_continuations_proj(
        &self,
        key: Key,
        proj: fn(
            crate::rspace::internal::WaitingContinuation<P, K>,
            bytes::Bytes,
        ) -> crate::rspace::internal::WaitingContinuation<P, K>,
    ) -> Vec<crate::rspace::internal::WaitingContinuation<P, K>> {
        todo!()
    }

    fn get_joins_proj(&self, key: Key, proj: fn(Vec<C>, bytes::Bytes) -> Vec<C>) -> Vec<Vec<C>> {
        todo!()
    }

    fn base(
        &self,
    ) -> Box<dyn crate::rspace::history::history_reader::HistoryReaderBase<C, P, A, K>> {
        todo!()
    }
}
