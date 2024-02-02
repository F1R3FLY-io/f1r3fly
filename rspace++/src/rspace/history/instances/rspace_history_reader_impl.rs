use crate::rspace::{
    hashing::blake3_hash::Blake3Hash,
    history::{
        cold_store::PersistedData,
        history::History,
        history_reader::{HistoryReader, HistoryReaderBase},
        history_repository::{PREFIX_DATUM, PREFIX_JOINS, PREFIX_KONT},
        history_repository_impl::prepend_bytes,
    },
    internal::{Datum, WaitingContinuation},
    serializers::serializers::{decode_continuations, decode_datums, decode_joins},
    shared::key_value_typed_store::KeyValueTypedStore,
};
use serde::Deserialize;
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

    /** Fetch data on a hash pointer */
    fn fetch_data(&self, prefix: u8, key: &Blake3Hash) -> Option<PersistedData> {
        let read_bytes = self
            .target_history
            .read(prepend_bytes(prefix, &key.bytes()))?;

        let read_hash = Blake3Hash::new(&read_bytes);
        let leaf_store_lock = self
            .leaf_store
            .lock()
            .expect("RSpace History Reader Impl: Unable to acquire leaf store lock");
        let get_opt = leaf_store_lock.get_one(&read_hash).unwrap();
        get_opt
    }
}

impl<C, P, A, K> HistoryReader<Blake3Hash, C, P, A, K> for RSpaceHistoryReaderImpl<C, P, A, K>
where
    C: Clone + for<'a> Deserialize<'a>,
    P: Clone + for<'a> Deserialize<'a>,
    A: Clone + for<'a> Deserialize<'a>,
    K: Clone + for<'a> Deserialize<'a>,
{
    fn root(&self) -> Blake3Hash {
        self.target_history.root()
    }

    fn get_data_proj(
        &self,
        key: Blake3Hash,
        proj: fn(Datum<A>, Vec<u8>) -> Datum<A>,
    ) -> Vec<Datum<A>> {
        match self.fetch_data(PREFIX_DATUM, &key) {
            Some(PersistedData::Data(data_leaf)) => decode_datums(&data_leaf.bytes),
            Some(p) => {
                panic!(
                    "Found unexpected leaf while looking for data at key {:?}, data: {:?}",
                    key, p
                );
            }
            None => Vec::new(),
        }
    }

    fn get_continuations_proj(
        &self,
        key: Blake3Hash,
        proj: fn(WaitingContinuation<P, K>, Vec<u8>) -> WaitingContinuation<P, K>,
    ) -> Vec<WaitingContinuation<P, K>> {
        match self.fetch_data(PREFIX_KONT, &key) {
            Some(PersistedData::Continuations(continuation_leaf)) => {
                decode_continuations(&continuation_leaf.bytes)
            }
            Some(p) => {
                panic!(
                    "Found unexpected leaf while looking for continuations at key {:?}, data: {:?}",
                    key, p
                );
            }
            None => Vec::new(),
        }
    }

    fn get_joins_proj(&self, key: Blake3Hash, proj: fn(Vec<C>, Vec<u8>) -> Vec<C>) -> Vec<Vec<C>> {
        match self.fetch_data(PREFIX_JOINS, &key) {
            Some(PersistedData::Joins(joins_leaf)) => decode_joins(&joins_leaf.bytes),
            Some(p) => {
                panic!(
                    "Found unexpected leaf while looking for joins at key {:?}, data: {:?}",
                    key, p
                );
            }
            None => Vec::new(),
        }
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
