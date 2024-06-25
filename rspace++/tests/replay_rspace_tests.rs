use rspace_plus_plus::rspace::history::history_repository::HistoryRepositoryInstances;
use rspace_plus_plus::rspace::hot_store::{HotStoreInstances, HotStoreState};
use rspace_plus_plus::rspace::hot_store_action::HotStoreAction;
use rspace_plus_plus::rspace::internal::{ContResult, RSpaceResult};
use rspace_plus_plus::rspace::matcher::r#match::Match;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceInstances};
use rspace_plus_plus::rspace::shared::in_mem_store_manager::InMemoryStoreManager;
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::sync::{Arc, Mutex};

// See rspace/src/test/scala/coop/rchain/rspace/ReplayRSpaceTests.scala

// See rspace/src/main/scala/coop/rchain/rspace/examples/StringExamples.scala
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
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

#[tokio::test]
async fn reset_to_a_checkpoint_from_a_different_branch_should_work() {
    let (mut space, mut replay_space) = fixture().await;

    let root0 = replay_space.replay_create_checkpoint().unwrap().root;
    assert!(replay_space.store.is_empty());

    let _ = space.produce("ch1".to_string(), "datum".to_string(), false);
    let root1 = space.create_checkpoint().unwrap().root;

    let _ = replay_space.reset(root1);
    assert!(replay_space.store.is_empty());

    let _ = space.reset(root0);
    assert!(space.store.is_empty());
}

#[tokio::test]
async fn creating_a_comm_event_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();
    let datum = "datum1".to_string();

    let empty_point = space.create_checkpoint().unwrap();
    let result_consume = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );
    let result_produce = space.produce(channels[0].clone(), datum.clone(), false);
    let rig_point = space.create_checkpoint().unwrap();

    assert!(result_consume.is_none());
    assert!(result_produce.is_some());
    assert_eq!(
        result_produce.clone().unwrap().0,
        ContResult {
            continuation: Arc::new(Mutex::new(continuation.clone())),
            persistent: false,
            channels: channels.clone(),
            patterns: patterns.clone(),
            peek: false,
        }
    );
    assert_eq!(
        result_produce.clone().unwrap().1,
        vec![RSpaceResult {
            channel: channels[0].clone(),
            matched_datum: datum.clone(),
            removed_datum: datum.clone(),
            persistent: false
        }]
    );

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);
    let replay_result_consume = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );

    let replay_result_produce =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert!(replay_result_consume.is_none());
    assert_eq!(replay_result_produce.clone().unwrap().0, result_produce.clone().unwrap().0);
    assert_eq!(replay_result_produce.unwrap().1, result_produce.unwrap().1);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

type StateSetup = (
    RSpace<String, Pattern, String, String, StringMatch>,
    RSpace<String, Pattern, String, String, StringMatch>,
);

async fn fixture() -> StateSetup {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();

    let history_repo = Arc::new(
        HistoryRepositoryInstances::<String, Pattern, String, String>::lmdb_repository(
            store.history.clone(),
            store.roots.clone(),
            store.cold.clone(),
        )
        .unwrap(),
    );

    let cache: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let history_reader = history_repo
        .get_history_reader(history_repo.root())
        .unwrap();

    let hot_store = {
        let hr = history_reader.base();
        HotStoreInstances::create_from_hs_and_hr(cache, hr)
    };

    let rspace = RSpaceInstances::apply(history_repo.clone(), hot_store, StringMatch);

    let history_cache: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let replay_store = {
        let hr = history_reader.base();
        HotStoreInstances::create_from_hs_and_hr(history_cache, hr)
    };

    let replay_rspace: RSpace<String, Pattern, String, String, StringMatch> =
        RSpaceInstances::apply(history_repo, replay_store, StringMatch);

    (rspace, replay_rspace)
}
