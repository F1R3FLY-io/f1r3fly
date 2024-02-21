// See rspace/src/test/scala/coop/rchain/rspace/history/HistoryActionTests.scala
#[cfg(test)]
mod tests {
    use rand::distributions::{Alphanumeric, DistString};
    use rspace_plus_plus::rspace::{
        hashing::blake3_hash::Blake3Hash,
        history::{
            history::{History, HistoryInstances},
            history_action::{
                DeleteAction, HistoryAction, HistoryActionTrait, InsertAction, KeyPath,
            },
            instances::radix_history::RadixHistory,
        },
        shared::mem_key_value_store::InMemoryKeyValueStore,
        Byte, ByteVector,
    };

    #[test]
    fn creating_and_read_one_record_should_work() {
        let data = vec![history_insert(zeros())];

        let empty_history = create_empty_history();
        let new_history = empty_history.process(data).unwrap();
        let read_value = new_history.read(zeros());
        assert!(read_value.is_ok());

        let expected_data = vec![insert(zeros())].first().unwrap().hash.bytes();
        assert_eq!(read_value.unwrap(), Some(expected_data));
    }

    #[test]
    fn reset_method_of_history_should_work() {
        let data = vec![history_insert(zeros())];

        let empty_history = create_empty_history();
        let new_history = empty_history.process(data).unwrap();
        let history_one_reset = empty_history.reset(&new_history.root());
        let read_value = history_one_reset.read(zeros());
        assert!(read_value.is_ok());

        let expected_data = vec![insert(zeros())].first().unwrap().hash.bytes();
        assert_eq!(read_value.unwrap(), Some(expected_data));
    }

    #[test]
    fn creating_ten_records_should_work() {
        let data: Vec<HistoryAction> = (0..10).map(zeros_and).map(|k| history_insert(k)).collect();

        let empty_history = create_empty_history();
        let new_history = empty_history.process(data.clone()).unwrap();

        let read_values: Vec<Option<ByteVector>> = data
            .iter()
            .map(|action| new_history.read(action.key()))
            .map(|read_result| {
                assert!(read_result.is_ok());
                read_result.unwrap()
            })
            .collect();

        let insert_actions: Vec<InsertAction> = (0..10).map(zeros_and).map(|k| insert(k)).collect();
        let expected_data: Vec<Option<ByteVector>> = insert_actions
            .iter()
            .map(|action| Some(action.hash.bytes()))
            .collect();
        assert_eq!(read_values, expected_data);
    }

    #[test]
    fn history_should_allow_to_store_different_length_key_records_in_different_branches() {
        let data = vec![
            history_insert(hex_key("01")),
            history_insert(hex_key("02")),
            history_insert(hex_key("0001")),
            history_insert(hex_key("0002")),
        ];

        let empty_history = create_empty_history();
        let new_history = empty_history.process(data.clone()).unwrap();

        let read_values: Vec<Option<ByteVector>> = data
            .iter()
            .map(|action| new_history.read(action.key()))
            .map(|read_result| {
                assert!(read_result.is_ok());
                read_result.unwrap()
            })
            .collect();

        let insert_actions = vec![
            insert(hex_key("01")),
            insert(hex_key("02")),
            insert(hex_key("0001")),
            insert(hex_key("0002")),
        ];
        let expected_data: Vec<Option<ByteVector>> = insert_actions
            .iter()
            .map(|action| Some(action.hash.bytes()))
            .collect();
        assert_eq!(read_values, expected_data);
    }

    // TODO: Don't works for MergingHistory
    #[test]
    fn deletion_of_a_non_existing_records_should_not_throw_error() {
        let changes_1 = vec![history_insert(hex_key("0011"))];
        let changes_2 = vec![history_delete(hex_key("0012")), history_delete(hex_key("0011"))];

        let empty_history = create_empty_history();
        let history_one = empty_history.process(changes_1).unwrap();
        let err = history_one.process(changes_2);

        assert!(err.is_ok());
    }

    fn create_empty_history() -> impl History {
        HistoryInstances::create(
            RadixHistory::empty_root_hash(),
            Box::new(InMemoryKeyValueStore::new()),
        )
    }

    fn zeros() -> KeyPath {
        vec![0; 32]
    }

    fn zeros_ones() -> KeyPath {
        [vec![0; 16], vec![1; 16]].concat()
    }

    fn thirty_one_zeros() -> KeyPath {
        vec![0; 31]
    }

    fn zeros_and(i: u8) -> KeyPath {
        let mut key_path = thirty_one_zeros();
        key_path.push(i);
        key_path
    }

    fn hex_key(s: &str) -> Vec<Byte> {
        hex::decode(s).unwrap()
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

    fn history_insert(k: KeyPath) -> HistoryAction {
        HistoryAction::Insert(InsertAction {
            key: k,
            hash: random_blake(),
        })
    }

    fn delete(k: KeyPath) -> DeleteAction {
        DeleteAction { key: k }
    }

    fn history_delete(k: KeyPath) -> HistoryAction {
        HistoryAction::Delete(DeleteAction { key: k })
    }
}
