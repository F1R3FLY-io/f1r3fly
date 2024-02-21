// See rspace/src/test/scala/coop/rchain/rspace/history/HistoryActionTests.scala
#[cfg(test)]
mod tests {
    use rand::distributions::{Alphanumeric, DistString};
    use rspace_plus_plus::rspace::{
        hashing::blake3_hash::Blake3Hash,
        history::{
            history::{History, HistoryInstances},
            history_action::{HistoryAction, InsertAction, KeyPath},
            instances::radix_history::RadixHistory,
        },
        shared::mem_key_value_store::InMemoryKeyValueStore,
    };

    fn create_empty_history() -> impl History {
        HistoryInstances::create(
            RadixHistory::empty_root_hash(),
            Box::new(InMemoryKeyValueStore::new()),
        )
    }

    fn zeros() -> KeyPath {
        vec![0u8; 32]
    }

    fn random_blake() -> Blake3Hash {
        Blake3Hash::new(
            &Alphanumeric
                .sample_string(&mut rand::thread_rng(), 32)
                .into_bytes(),
        )
    }

    fn insert(k: KeyPath) -> InsertAction {
        InsertAction {
            key: k,
            hash: random_blake(),
        }
    }

    #[tokio::test]
    async fn creating_and_read_one_record_should_work() {
        let data = vec![HistoryAction::Insert(insert(zeros()))];

        let empty_history = create_empty_history();
        let new_history = empty_history.process(data);
        let read_value = new_history.read(zeros());
        assert!(read_value.is_ok());

        let expected_data = vec![insert(zeros())].first().unwrap().hash.bytes();
        assert_eq!(read_value.unwrap(), Some(expected_data));
    }
}
