use crate::rspace::internal::{Datum, WaitingContinuation};

/**
* Reader for particular history (state verified on blockchain)
*
* @tparam Key type for hash of a channel
* @tparam C type for Channel => this is Par
* @tparam P type for Pattern => this is BindPattern
* @tparam A type for Abstraction => this is ListParWithRandom
* @tparam K type for Continuation => this is TaggedContinuation
*
* See rspace/src/main/scala/coop/rchain/rspace/history/HistoryReader.scala
*/
pub trait HistoryReader<Key, C: Clone, P: Clone, A: Clone, K: Clone> {
    // Get current root which reader reads from
    fn root(&self) -> Key;

    fn get_data_proj(&self, key: &Key) -> Vec<Datum<A>>;

    fn get_continuations_proj(&self, key: &Key) -> Vec<WaitingContinuation<P, K>>;

    fn get_joins_proj(&self, key: &Key) -> Vec<Vec<C>>;

    /**                                                                                                                                                                                                              
     * Defaults                                                                                                                                                                                                       
     */
    fn get_data(&self, key: &Key) -> Vec<Datum<A>> {
        self.get_data_proj(key)
    }

    fn get_continuations(&self, key: &Key) -> Vec<WaitingContinuation<P, K>> {
        self.get_continuations_proj(key)
    }

    fn get_joins(&self, key: &Key) -> Vec<Vec<C>> {
        self.get_joins_proj(key)
    }

    /**
     * Get reader which accepts non-serialized and hashed keys
     */
    fn base(&self) -> Box<dyn HistoryReaderBase<C, P, A, K>>;
}

/**
 * History reader base, version of a reader which accepts non-serialized and hashed keys
 */
pub trait HistoryReaderBase<C: Clone, P: Clone, A: Clone, K: Clone> {
    fn get_data_proj(&self, key: &C) -> Vec<Datum<A>>;

    fn get_continuations_proj(&self, key: &Vec<C>) -> Vec<WaitingContinuation<P, K>>;

    fn get_joins_proj(&self, key: &C) -> Vec<Vec<C>>;

    /**                                                                                                                                                                                                              
     * Defaults                                                                                                                                                                                                       
     */
    fn get_data(&self, key: &C) -> Vec<Datum<A>> {
        self.get_data_proj(key)
    }

    fn get_continuations(&self, key: &Vec<C>) -> Vec<WaitingContinuation<P, K>> {
        self.get_continuations_proj(key)
    }

    fn get_joins(&self, key: &C) -> Vec<Vec<C>> {
        self.get_joins_proj(key)
    }
}
