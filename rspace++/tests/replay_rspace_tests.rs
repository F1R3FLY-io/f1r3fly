use rspace_plus_plus::rspace::history::history_repository::{
    HistoryRepository, HistoryRepositoryInstances,
};
use rspace_plus_plus::rspace::hot_store::{HotStore, HotStoreInstances, HotStoreState};
use rspace_plus_plus::rspace::hot_store_action::{
    HotStoreAction, InsertAction, InsertContinuations, InsertData,
};
use rspace_plus_plus::rspace::internal::{ContResult, RSpaceResult};
use rspace_plus_plus::rspace::internal::{Datum, WaitingContinuation};
use rspace_plus_plus::rspace::matcher::r#match::Match;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceInstances};
use rspace_plus_plus::rspace::shared::in_mem_store_manager::InMemoryStoreManager;
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use rspace_plus_plus::rspace::shared::lmdb_dir_store_manager::GB;
use rspace_plus_plus::rspace::shared::rspace_store_manager::mk_rspace_store_manager;
use rspace_plus_plus::rspace::trace::event::Consume;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

// See rspace/src/test/scala/coop/rchain/rspace/ReplayRSpaceTests.scala

// See rspace/src/main/scala/coop/rchain/rspace/examples/StringExamples.scala
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
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

// We only care that both vectors contain the same elements, not their ordering
fn check_same_elements<T: Hash + Eq>(vec1: Vec<T>, vec2: Vec<T>) -> bool {
    let set1: HashSet<_> = vec1.into_iter().collect();
    let set2: HashSet<_> = vec2.into_iter().collect();
    set1 == set2
}

pub fn filter_enum_variants<C: Clone, P: Clone, A: Clone, K: Clone, V>(
    vec: Vec<HotStoreAction<C, P, A, K>>,
    variant: fn(HotStoreAction<C, P, A, K>) -> Option<V>,
) -> Vec<V> {
    vec.into_iter().filter_map(variant).collect()
}

async fn create_rspace() -> RSpace<String, Pattern, String, String, StringMatch> {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();

    RSpaceInstances::create(store, StringMatch).unwrap()
}

#[tokio::test]
async fn produce_should_persist_data_in_store() {
    let mut rspace = create_rspace().await;

    let root_0 = rspace.replay_create_checkpoint().unwrap().root;
    // assert!()

    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r = rspace.produce(key[0].clone(), "datum".to_string(), false);
    let data = rspace.store.get_data(&channel);
    assert_eq!(data, vec![Datum::create(channel.clone(), "datum".to_string(), false)]);

    let cont = rspace.store.get_continuations(key);
    assert_eq!(cont.len(), 0);
    assert!(r.is_none());

    let insert_data: Vec<InsertData<_, _>> = filter_enum_variants(rspace.store.changes(), |e| {
        if let HotStoreAction::Insert(InsertAction::InsertData(d)) = e {
            Some(d)
        } else {
            None
        }
    });
    assert_eq!(insert_data.len(), 1);
    assert_eq!(
        insert_data
            .into_iter()
            .map(|d| d.channel)
            .collect::<String>(),
        channel
    );
}

type StateSetup = (
    Arc<Box<dyn HotStore<String, Pattern, String, String>>>,
    Arc<Box<dyn HotStore<String, Pattern, String, String>>>,
    RSpace<String, Pattern, String, String, StringMatch>,
    RSpace<String, Pattern, String, String, StringMatch>,
);

async fn fixture() -> StateSetup {
    let mut kvm = mk_rspace_store_manager("./lmdb/".into(), 1 * GB);
    let store = kvm.r_space_stores().await.unwrap();

    let history_repo_rspace =
        HistoryRepositoryInstances::<String, Pattern, String, String>::lmdb_repository(
            store.history.clone(),
            store.roots.clone(),
            store.cold.clone(),
        )
        .unwrap();

    let cache: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let history_reader = history_repo_rspace
        .get_history_reader(history_repo_rspace.root())
        .unwrap();

    let hot_store = {
        let hr = history_reader.base();
        Arc::new(HotStoreInstances::create_from_hs_and_hr(cache, hr))
    };

    let rspace =
        RSpaceInstances::apply(Box::new(history_repo_rspace), hot_store.clone(), StringMatch);

    let history_cache: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let replay_store = {
        let hr = history_reader.base();
        Arc::new(HotStoreInstances::create_from_hs_and_hr(history_cache, hr))
    };

    let history_repo_replay_rspace =
        HistoryRepositoryInstances::<String, Pattern, String, String>::lmdb_repository(
            store.history,
            store.roots,
            store.cold,
        )
        .unwrap();

    let replay_rspace: RSpace<String, Pattern, String, String, StringMatch> =
        RSpaceInstances::apply(
            Box::new(history_repo_replay_rspace),
            replay_store.clone(),
            StringMatch,
        );

    (hot_store, replay_store, rspace, replay_rspace)
}
