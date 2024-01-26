use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    history::{
        cold_store::PersistedData,
        history::History,
        history_reader::{HistoryReader, HistoryReaderBase},
    },
    internal::{Datum, WaitingContinuation},
    shared::key_value_typed_store::KeyValueTypedStore,
};
use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

pub struct RSpaceHistoryReaderImpl<C, P, A, K> {
    target_history: Box<dyn History>,
    leaf_store: Arc<Mutex<Box<dyn KeyValueTypedStore<Blake3Hash, PersistedData>>>>,
    _marker: PhantomData<(C, P, A, K)>,
}

impl<C, P, A, K> RSpaceHistoryReaderImpl<C, P, A, K> {
    pub fn new(
        target_history: Box<dyn History>,
        leaf_store: Arc<Mutex<Box<dyn KeyValueTypedStore<Blake3Hash, PersistedData>>>>,
    ) -> Self {
        RSpaceHistoryReaderImpl {
            target_history,
            leaf_store,
            _marker: PhantomData,
        }
    }
}

impl<Key, C: Clone, P: Clone, A: Clone, K: Clone> HistoryReader<Key, C, P, A, K>
    for RSpaceHistoryReaderImpl<C, P, A, K>
{
    fn root(&self) -> Key {
        todo!()
    }

    fn get_data_proj(&self, key: Key, proj: fn(Datum<A>, Vec<u8>) -> Datum<A>) -> Vec<Datum<A>> {
        todo!()
    }

    fn get_continuations_proj(
        &self,
        key: Key,
        proj: fn(WaitingContinuation<P, K>, Vec<u8>) -> WaitingContinuation<P, K>,
    ) -> Vec<WaitingContinuation<P, K>> {
        todo!()
    }

    fn get_joins_proj(&self, key: Key, proj: fn(Vec<C>, Vec<u8>) -> Vec<C>) -> Vec<Vec<C>> {
        todo!()
    }

    fn base(&self) -> Box<dyn HistoryReaderBase<C, P, A, K>> {
        struct HistoryReader;

        impl<C: Clone, P: Clone, A: Clone, K: Clone> HistoryReaderBase<C, P, A, K> for HistoryReader {
            fn get_data_proj(
                &self,
                key: C,
                proj: fn(Datum<A>, Vec<u8>) -> Datum<A>,
            ) -> Vec<Datum<A>> {
                todo!()
            }

            fn get_continuations_proj(
                &self,
                key: Vec<C>,
                proj: fn(WaitingContinuation<P, K>, Vec<u8>) -> WaitingContinuation<P, K>,
            ) -> Vec<WaitingContinuation<P, K>> {
                todo!()
            }

            fn get_joins_proj(&self, key: C, proj: fn(Vec<C>, Vec<u8>) -> Vec<C>) -> Vec<Vec<C>> {
                todo!()
            }
        }

        Box::new(HistoryReader)
    }
}
