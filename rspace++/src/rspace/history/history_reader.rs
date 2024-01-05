// use std::marker::PhantomData;

// use crate::rspace::dbs::disk_conc::DiskConcDB;
// use crate::rspace::history::history::HistoryImpl;
use crate::rspace::internal::{Datum, WaitingContinuation};

/**
 * Reader for particular history (state verified on blockchain)
 *
 * @tparam F effect type
 * @tparam Key type for hash of a channel
 * @tparam C type for Channel => this is Par
 * @tparam P type for Pattern => this is BindPattern
 * @tparam A type for Abstraction => this is ListParWithRandom
 * @tparam K type for Continuation => this is TaggedContinuation
 */
// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryReader.scala
pub trait HistoryReader<Key, C, P, A, K> {
    // Get current root which reader reads from
    fn root(&self) -> Key;

    fn get_data_proj<R>(&self, key: Key, proj: fn(Datum<A>, Vec<u8>) -> R) -> Option<Vec<R>>;

    fn get_continuations_proj<R>(
        &self,
        key: Key,
        proj: fn(WaitingContinuation<P, K>, Vec<u8>) -> R,
    ) -> Option<Vec<R>>;

    fn get_joins_proj<R>(&self, key: Key, proj: fn(Vec<C>, Vec<u8>) -> R) -> Option<Vec<R>>;

    /**                                                                                                                                                                                                              
     * Defaults                                                                                                                                                                                                       
     */
    fn get_data(&self, key: Key) -> Option<Vec<Datum<A>>> {
        self.get_data_proj(key, |d, _| d)
    }

    fn get_continuations(&self, key: Key) -> Option<Vec<WaitingContinuation<P, K>>> {
        self.get_continuations_proj(key, |d, _| d)
    }

    fn get_joins(&self, key: Key) -> Option<Vec<Vec<C>>> {
        self.get_joins_proj(key, |d, _| d)
    }

    // /**
    //  * Get reader which accepts non-serialized and hashed keys
    //  */
    // fn base(&self) -> Box<dyn HistoryReaderBase<C, P, A, K>>;
}

/**
 * History reader base, version of a reader which accepts non-serialized and hashed keys
 */
// See rspace/src/main/scala/coop/rchain/rspace/history/HistoryReader.scala
pub trait HistoryReaderBase<C, P, A, K> {
    fn get_data_proj<R>(&self, key: C, proj: fn(Datum<A>, Vec<u8>) -> R) -> Option<Vec<R>>;

    fn get_continuations_proj<R>(
        &self,
        key: Vec<C>,
        proj: fn(WaitingContinuation<P, K>, Vec<u8>) -> R,
    ) -> Option<Vec<R>>;

    fn get_joins_proj<R>(&self, key: C, proj: fn(Vec<C>, Vec<u8>) -> R) -> Option<Vec<R>>;

    /**                                                                                                                                                                                                              
     * Defaults                                                                                                                                                                                                       
     */
    fn get_data(&self, key: C) -> Option<Vec<Datum<A>>> {
        self.get_data_proj(key, |d, _| d)
    }

    fn get_continuations(&self, key: Vec<C>) -> Option<Vec<WaitingContinuation<P, K>>> {
        self.get_continuations_proj(key, |d, _| d)
    }

    fn get_joins(&self, key: C) -> Option<Vec<Vec<C>>> {
        self.get_joins_proj(key, |d, _| d)
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/history/instances/RSpaceHistoryReaderImpl.scala
// pub struct HistoryReaderImpl<C, P, A, K> {
//     target_history: HistoryImpl,
//     leaf_store: DiskConcDB,
//     phantom: PhantomData<(C, P, A, K)>,
// }
