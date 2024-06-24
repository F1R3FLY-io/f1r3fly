use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::history::history_repository::HistoryRepository;
use rspace_plus_plus::rspace::hot_store::HotStoreInstances;
use rspace_plus_plus::rspace::rspace::RSpaceInstances;
use rspace_plus_plus::rspace::Byte;
use rspace_plus_plus::rspace::{
    history::history_repository::HistoryRepositoryInstances,
    hot_store::HotStoreState,
    matcher::r#match::Match,
    rspace::RSpace,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
    state::{rspace_exporter::RSpaceExporter, rspace_importer::RSpaceImporter},
};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::sync::{Arc, Mutex};

// See rspace/src/main/scala/coop/rchain/rspace/examples/StringExamples.scala
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
enum Pattern {
    #[default]
    Wildcard,
    StringMatch(String),
}

#[derive(Clone)]
struct StringMatch;

impl Match<Pattern, String> for StringMatch {
    fn get(&self, p: Pattern, a: String) -> Option<String> {
        match p {
            Pattern::Wildcard => Some(a),
            Pattern::StringMatch(value) => {
                if value == a {
                    Some(a)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct StringsCaptor {
    res: LinkedList<Vec<String>>,
}

impl StringsCaptor {
    fn new() -> Self {
        StringsCaptor {
            res: LinkedList::new(),
        }
    }

    fn run_k(&mut self, data: Vec<String>) {
        self.res.push_back(data);
    }

    fn results(&self) -> Vec<Vec<String>> {
        self.res.iter().cloned().collect()
    }
}

// TODO: Don't works for MergingHistory

#[tokio::test]
async fn export_and_import_of_one_page_should_works_correctly() {
    let (mut space1, exporter1, importer1, space2, _, importer2) = test_setup().await;

    let page_size = 1000;
    let data_size = 10;
    let start_skip = 0;
    let range = 0..data_size;
    let pattern = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();

    // Generate init data in space1
    for i in 0..data_size {
        space1.produce(format!("ch{}", i), format!("data{}", i), false);
    }

    let init_point = space1.create_checkpoint().unwrap();

    // Export 1 page from space1
    let init_start_path: Vec<(Blake2b256Hash, Option<Byte>)> = vec![(init_point.root, None)];
}

// See rspace/src/test/scala/coop/rchain/rspace/ExportImportTests.scala
async fn test_setup() -> (
    RSpace<String, Pattern, String, String, StringMatch>,
    Arc<Mutex<Box<dyn RSpaceExporter>>>,
    Arc<Mutex<Box<dyn RSpaceImporter>>>,
    RSpace<String, Pattern, String, String, StringMatch>,
    Arc<Mutex<Box<dyn RSpaceExporter>>>,
    Arc<Mutex<Box<dyn RSpaceImporter>>>,
) {
    let mut kvm = InMemoryStoreManager::new();

    let roots1 = kvm.store("roots1".to_string()).await.unwrap();
    let cold1 = kvm.store("cold1".to_string()).await.unwrap();
    let history1 = kvm.store("history1".to_string()).await.unwrap();

    let history_repository1 =
        HistoryRepositoryInstances::<String, Pattern, String, String>::lmdb_repository(
            Arc::new(Mutex::new(roots1)),
            Arc::new(Mutex::new(cold1)),
            Arc::new(Mutex::new(history1)),
        )
        .unwrap();

    let cache1: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let history_reader = history_repository1
        .get_history_reader(history_repository1.root())
        .unwrap();

    let store1 = {
        let hr = history_reader.base();
        Arc::new(HotStoreInstances::create_from_hs_and_hr(cache1, hr))
    };

    let exporter1 = history_repository1.exporter();
    let importer1 = history_repository1.importer();

    let space1 = RSpaceInstances::apply(Box::new(history_repository1), store1, StringMatch);

    let roots2 = kvm.store("roots2".to_string()).await.unwrap();
    let cold2 = kvm.store("cold2".to_string()).await.unwrap();
    let history2 = kvm.store("history2".to_string()).await.unwrap();

    let history_repository2 =
        HistoryRepositoryInstances::<String, Pattern, String, String>::lmdb_repository(
            Arc::new(Mutex::new(roots2)),
            Arc::new(Mutex::new(cold2)),
            Arc::new(Mutex::new(history2)),
        )
        .unwrap();

    let cache2: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let history_reader = history_repository2
        .get_history_reader(history_repository2.root())
        .unwrap();

    let store2 = {
        let hr = history_reader.base();
        Arc::new(HotStoreInstances::create_from_hs_and_hr(cache2, hr))
    };

    let exporter2 = history_repository2.exporter();
    let importer2 = history_repository2.importer();

    let space2 = RSpaceInstances::apply(Box::new(history_repository2), store2, StringMatch);

    (space1, exporter1, importer1, space2, exporter2, importer2)
}
