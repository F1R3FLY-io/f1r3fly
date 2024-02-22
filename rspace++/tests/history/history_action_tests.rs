// See rspace/src/test/scala/coop/rchain/rspace/history/HistoryActionTests.scala
#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};
    use std::error::Error;

    use rand::distributions::{Alphanumeric, DistString};
    use rand::seq::SliceRandom;
    use rand::Rng;
    use rspace_plus_plus::rspace::history::radix_tree::RadixTreeError;
    use rspace_plus_plus::rspace::shared::key_value_store::KeyValueStore;
    use rspace_plus_plus::rspace::{
        hashing::blake3_hash::Blake3Hash,
        history::{
            history::{History, HistoryError, HistoryInstances},
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
        let changes1 = vec![history_insert(hex_key("0011"))];
        let changes2 = vec![history_delete(hex_key("0012")), history_delete(hex_key("0011"))];

        let empty_history = create_empty_history();
        let history_one = empty_history.process(changes1).unwrap();
        let err = history_one.process(changes2);

        assert!(err.is_ok());
    }

    #[test]
    fn history_should_not_allow_to_store_different_length_key_records_in_same_branch() {
        let data = vec![history_insert(hex_key("01")), history_insert(hex_key("0100"))];
        let empty_history = create_empty_history();
        let err = empty_history.process(data);

        assert!(err.is_err());
        match err {
            Err(HistoryError::RadixTreeError(RadixTreeError::PrefixError(message))) => {
                assert_eq!(message, "The length of all prefixes in the subtree must be the same.");
            }
            _ => panic!("Expected PrefixError variant"),
        }

        // TODO: For MergingHistory
        // ex shouldBe a[RuntimeException]
        // ex.getMessage shouldBe s"malformed trie"
    }

    #[test]
    fn history_should_not_allow_to_process_history_actions_with_same_keys() {
        let data1 = vec![history_insert(zeros()), history_insert(zeros())];
        let empty_history = create_empty_history();
        let err1 = empty_history.process(data1);

        assert!(err1.is_err());
        match err1 {
            Err(HistoryError::ActionError(message)) => {
                assert_eq!(message, "Cannot process duplicate actions on one key.");
            }
            _ => panic!("Expected ActionError variant"),
        }

        let data2 = vec![history_insert(zeros()), history_delete(zeros())];
        let empty_history = create_empty_history();
        let err2 = empty_history.process(data2);

        assert!(err2.is_err());
        match err2 {
            Err(HistoryError::ActionError(message)) => {
                assert_eq!(message, "Cannot process duplicate actions on one key.");
            }
            _ => panic!("Expected ActionError variant"),
        }
    }

    #[test]
    fn history_after_deleting_all_records_should_be_empty() {
        let insertions = vec![history_insert(zeros())];
        let deletions = vec![history_delete(zeros())];

        let empty_history = create_empty_history();
        let history_one = empty_history.process(insertions);
        assert!(history_one.is_ok());

        let history_two = history_one.unwrap().process(deletions);
        assert!(history_two.is_ok());
        assert_eq!(history_two.unwrap().root(), empty_history.root());
    }

    #[test]
    fn reading_of_a_non_existing_records_should_return_none() {
        let key = hex_key("0011");
        let empty_history = create_empty_history();
        let read_value = empty_history.read(key);
        assert!(read_value.is_ok());
        assert!(read_value.unwrap().is_none());
    }

    #[test]
    fn update_of_a_record_should_not_change_past_history() {
        let insert_one = vec![history_insert(zeros())];
        let insert_two = vec![history_insert(zeros())];

        let empty_history = create_empty_history();
        let history_one = empty_history.process(insert_one);
        assert!(history_one.is_ok());
        let read_value_one_pre = history_one.as_ref().unwrap().read(zeros());

        let history_two = history_one.as_ref().unwrap().process(insert_two);
        assert!(history_two.is_ok());
        let read_value_one_post = history_one.unwrap().read(zeros());
        let read_value_two = history_two.unwrap().read(zeros());

        assert!(read_value_one_pre.is_ok());
        assert!(read_value_one_post.is_ok());
        assert_eq!(read_value_one_pre.as_ref().unwrap(), &read_value_one_post.unwrap());
        assert_ne!(read_value_one_pre.unwrap(), read_value_two.unwrap());
    }

    #[test]
    fn history_should_correctly_build_the_same_trees_in_different_ways() {
        let insert_one = vec![history_insert(hex_key("010000")), history_insert(hex_key("0200"))];
        let insert_two = vec![history_insert(hex_key("010001")), history_insert(hex_key("0300"))];
        let mut insert_one_two = insert_one.clone();
        insert_one_two.extend(insert_two.clone());

        let delete_one = vec![
            history_delete(insert_one.first().unwrap().key()),
            history_delete(insert_one.get(1).unwrap().key()),
        ];
        let delete_two = vec![
            history_delete(insert_two.first().unwrap().key()),
            history_delete(insert_two.get(1).unwrap().key()),
        ];
        let mut delete_one_two = delete_one.clone();
        delete_one_two.extend(delete_two.clone());

        let empty_history = create_empty_history();
        let history_one = empty_history.process(insert_one);
        assert!(history_one.is_ok());
        let history_two = empty_history.process(insert_two.clone());
        assert!(history_two.is_ok());
        let history_one_two = empty_history.process(insert_one_two);

        let history_one_two_another_way = history_one.as_ref().unwrap().process(insert_two);
        assert!(history_one_two_another_way.is_ok());
        assert_eq!(
            history_one_two.as_ref().unwrap().root(),
            history_one_two_another_way.unwrap().root()
        );

        let history_one_another_way = history_one_two.as_ref().unwrap().process(delete_two);
        assert!(history_one_another_way.is_ok());
        assert_eq!(&history_one.unwrap().root(), &history_one_another_way.unwrap().root());

        let history_two_another_way = history_one_two.as_ref().unwrap().process(delete_one);
        assert!(history_two_another_way.is_ok());
        assert_eq!(&history_two.unwrap().root(), &history_two_another_way.unwrap().root());

        let empty_history_another_way = history_one_two.unwrap().process(delete_one_two);
        assert!(empty_history_another_way.is_ok());
        assert_eq!(&empty_history.root(), &empty_history_another_way.unwrap().root());
    }

    #[test]
    fn adding_already_existing_records_should_not_change_history() {
        let inserts = vec![history_insert(zeros())];

        let (empty_history, in_mem_store) = create_empty_history_and_store();
        let empty_history_size = in_mem_store.size_bytes();

        let history_one = empty_history.process(inserts.clone());
        assert!(history_one.is_ok());
        let history_one_size = in_mem_store.size_bytes();

        let history_two = history_one.as_ref().unwrap().process(inserts);
        assert!(history_two.is_ok());
        let history_two_size = in_mem_store.size_bytes();

        assert_eq!(history_one.unwrap().root(), history_two.unwrap().root());
        assert_eq!(empty_history_size, 0_usize);
        assert_eq!(history_one_size, history_two_size);
    }

    #[test]
    fn collision_detecting_in_kvdb_should_work() {
        let insert_record = vec![history_insert(zeros())];
        let delete_record = vec![history_delete(zeros())];
        let collision_kv_pair = (RadixHistory::empty_root_hash().bytes(), random_blake().bytes());

        let (empty_history, in_mem_store) = create_empty_history_and_store();
        let new_history = empty_history.process(insert_record);
        assert!(new_history.is_ok());
        assert!(in_mem_store.put(vec![collision_kv_pair]).is_ok());

        let err = new_history.unwrap().process(delete_record);
        assert!(err.is_err());
        match err {
            Err(HistoryError::RadixTreeError(RadixTreeError::CollisionError(message))) => {
                assert_eq!(
                    message,
                    format!(
                        "1 collisions in KVDB (first collision with key = {}.",
                        hex::encode(RadixHistory::empty_root_hash().bytes())
                    )
                );
            }
            _ => panic!("Expected CollisionError variant"),
        }
    }

    #[test]
    fn randomly_insert_or_delete_should_return_the_correct_result() {
        let size_inserts = 10000;
        let size_deletes = 3000;
        let size_updates = 1000;
        let state: BTreeMap<KeyPath, Blake3Hash> = BTreeMap::new();
        let inserts = generate_random_insert(0);

        let empty_history = create_empty_history();

        let res: Result<(Box<dyn History>, Vec<HistoryAction>, BTreeMap<Vec<u8>, Blake3Hash>), _> =
            (1..=10).try_fold((empty_history, inserts, state), |(history, inserts, state), _| {
                let new_inserts = generate_random_insert(size_inserts);
                let new_updates = generate_random_insert_from_insert(size_updates, &inserts);

                let update_key: HashSet<_> =
                    new_updates.iter().map(|action| action.key()).collect();

                let last_inserts: Vec<_> = inserts
                    .into_iter()
                    .filter(|i| !update_key.contains(&i.key()))
                    .collect();

                let new_deletes = generate_random_delete_from_insert(size_deletes, last_inserts)
                    .into_iter()
                    .chain(generate_random_delete(size_deletes).into_iter())
                    .collect::<Vec<_>>();

                let actions = new_inserts
                    .clone()
                    .into_iter()
                    .chain(new_deletes.clone().into_iter())
                    .chain(new_updates.into_iter())
                    .collect::<Vec<_>>();

                println!("\nprocess {}", actions.len());
                let new_history = history.process(actions.clone());
                assert!(new_history.is_ok());
                let new_state = update_state(state, &actions);
                println!("\nThe new state size is {}", new_state.len());

                for (k, value) in new_state.iter() {
                    match new_history.as_ref().unwrap().read(k.to_vec()) {
                        Ok(Some(v)) => assert!(v == value.bytes()),
                        _ => assert!(false, "Can not get value"),
                    }
                }

                for d in new_deletes.iter() {
                    match new_history.as_ref().unwrap().read(d.key()) {
                        Ok(None) => assert!(true),
                        _ => assert!(false, "got empty pointer after remove"),
                    }
                }

                Ok::<
                    (Box<dyn History>, Vec<HistoryAction>, BTreeMap<Vec<u8>, Blake3Hash>),
                    Box<dyn Error>,
                >((new_history.unwrap(), new_inserts, new_state))
            });

        assert!(res.is_ok());
    }

    fn create_empty_history() -> Box<dyn History> {
        Box::new(HistoryInstances::create(
            RadixHistory::empty_root_hash(),
            Box::new(InMemoryKeyValueStore::new()),
        ))
    }

    fn create_empty_history_and_store() -> (impl History, InMemoryKeyValueStore) {
        let store = InMemoryKeyValueStore::new();

        (HistoryInstances::create(RadixHistory::empty_root_hash(), Box::new(store.clone())), store)
    }

    fn random_key(size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        (0..size)
            .map(|_| {
                let num = rng.gen_range(0..256) as i16;
                (num - 128) as u8
            })
            .collect()
    }

    fn generate_random_insert(size: usize) -> Vec<HistoryAction> {
        (0..size).map(|_| history_insert(random_key(32))).collect()
    }

    fn generate_random_delete(size: usize) -> Vec<HistoryAction> {
        (0..size).map(|_| history_delete(random_key(32))).collect()
    }

    fn generate_random_delete_from_insert(
        size: usize,
        inserts: Vec<HistoryAction>,
    ) -> Vec<HistoryAction> {
        let mut rng = rand::thread_rng();
        let mut shuffled_inserts = inserts;
        shuffled_inserts.shuffle(&mut rng);
        shuffled_inserts
            .into_iter()
            .take(size)
            .map(|i| history_delete(i.key()))
            .collect()
    }

    fn generate_random_insert_from_insert(
        size: usize,
        inserts: &Vec<HistoryAction>,
    ) -> Vec<HistoryAction> {
        let mut rng = rand::thread_rng();
        let mut shuffled_inserts = inserts.clone();
        shuffled_inserts.shuffle(&mut rng);
        shuffled_inserts
            .into_iter()
            .take(size)
            .map(|i| history_insert(i.key()))
            .collect()
    }

    fn update_state(
        mut state: BTreeMap<KeyPath, Blake3Hash>,
        actions: &Vec<HistoryAction>,
    ) -> BTreeMap<KeyPath, Blake3Hash> {
        for action in actions {
            match action {
                HistoryAction::Insert(InsertAction { key, hash }) => {
                    state.insert(key.clone(), hash.clone());
                }
                HistoryAction::Delete(DeleteAction { key }) => {
                    state.remove(&key.clone());
                }
            }
        }
        state
    }

    fn zeros() -> KeyPath {
        vec![0; 32]
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

    fn history_delete(k: KeyPath) -> HistoryAction {
        HistoryAction::Delete(DeleteAction { key: k })
    }
}
