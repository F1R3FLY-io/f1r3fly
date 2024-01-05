use crate::rspace::history::history_action::HistoryAction;
use crate::rspace::history::instances::radix_history::RadixHistory;
use crate::rspace::shared::key_value_store::KeyValueStore;

// See rspace/src/main/scala/coop/rchain/rspace/history/History.scala
pub trait History {
    fn read(&self, key: Vec<u8>) -> Option<Vec<u8>>;

    fn process(&self, actions: Vec<HistoryAction>) -> dyn History;

    fn root(&self) -> blake3::Hash;

    fn reset(&self, root: blake3::Hash) -> dyn History;
}

// struct HistoryStruct {}

// impl dyn History {
//     fn create(&self, root: blake3::Hash, store: KeyValueStore) -> RadixHistory {
//         todo!()
//     }
// }
