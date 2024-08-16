use rand::prelude::SliceRandom;
use rand::thread_rng;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::history::history_repository::HistoryRepositoryInstances;
use rspace_plus_plus::rspace::hot_store::{HotStoreInstances, HotStoreState};
use rspace_plus_plus::rspace::hot_store_action::{
    HotStoreAction, InsertAction, InsertContinuations,
};
use rspace_plus_plus::rspace::internal::{ContResult, RSpaceResult};
use rspace_plus_plus::rspace::r#match::Match;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceInstances};
use rspace_plus_plus::rspace::shared::in_mem_store_manager::InMemoryStoreManager;
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use rspace_plus_plus::rspace::trace::event::{Consume, IOEvent};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};
use std::hash::Hash;
use std::sync::Arc;

// See rspace/src/test/scala/coop/rchain/rspace/ReplayRSpaceTests.scala

// See rspace/src/main/scala/coop/rchain/rspace/examples/StringExamples.scala
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
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
            continuation: continuation.clone(),
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

#[tokio::test]
async fn creating_a_comm_event_with_peek_consume_first_should_replay_correctly() {
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
        BTreeSet::from([0]),
    );
    let result_produce = space.produce(channels[0].clone(), datum.clone(), false);
    let rig_point = space.create_checkpoint().unwrap();

    assert!(result_consume.is_none());
    assert!(result_produce.is_some());
    assert_eq!(
        result_produce.clone().unwrap().0,
        ContResult {
            continuation: continuation.clone(),
            persistent: false,
            channels: channels.clone(),
            patterns: patterns.clone(),
            peek: true,
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
        BTreeSet::from([0]),
    );

    let replay_result_produce =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert!(replay_result_consume.is_none());
    assert_eq!(replay_result_produce.clone().unwrap().0, result_produce.clone().unwrap().0);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn creating_a_comm_event_with_peek_produce_first_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();
    let datum = "datum1".to_string();

    let empty_point = space.create_checkpoint().unwrap();
    let result_produce = space.produce(channels[0].clone(), datum.clone(), false);
    let result_consume = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );

    let rig_point = space.create_checkpoint().unwrap();

    assert!(result_produce.is_none());
    assert!(result_consume.is_some());

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);
    let replay_result_produce =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);

    let replay_result_consume = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert!(replay_result_produce.is_none());
    assert_eq!(replay_result_consume.clone().unwrap().0, result_consume.clone().unwrap().0);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn creating_comm_events_on_many_channels_with_peek_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string(), "ch2".to_string()];
    let patterns = vec![Pattern::Wildcard, Pattern::Wildcard];
    let continuation = "continuation".to_string();
    let datum = "datum1".to_string();

    let empty_point = space.create_checkpoint().unwrap();

    let result_consume1 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );

    let result_produce1 = space.produce(channels[1].clone(), datum.clone(), false);
    let result_produce2 = space.produce(channels[0].clone(), datum.clone(), false);
    let _result_produce2a = space.produce(channels[0].clone(), datum.clone(), false);

    let result_consume2 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([1]),
    );

    let result_produce3 = space.produce(channels[1].clone(), datum.clone(), false);
    let _result_produce3a = space.produce(channels[1].clone(), datum.clone(), false);

    let result_consume3 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );

    let result_produce4 = space.produce(channels[0].clone(), datum.clone(), false);

    let rig_point = space.create_checkpoint().unwrap();

    assert!(result_consume1.is_none());
    assert!(result_produce1.is_none());
    assert!(result_produce2.is_some());
    assert!(result_consume2.is_none());
    assert!(result_produce3.is_some());
    assert!(result_consume3.is_none());
    assert!(result_produce4.is_some());

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    let replay_result_consume1 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );

    let replay_result_produce1 =
        replay_space.replay_produce(channels[1].clone(), datum.clone(), false);
    let replay_result_produce2 =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let replay_result_produce2a =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);

    let replay_result_consume2 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([1]),
    );

    let replay_result_produce3 =
        replay_space.replay_produce(channels[1].clone(), datum.clone(), false);
    let replay_result_produce3a =
        replay_space.replay_produce(channels[1].clone(), datum.clone(), false);

    let replay_result_consume3 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );

    let replay_result_produce4 =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);

    assert!(replay_result_consume1.is_none());
    assert!(replay_result_produce1.is_none());
    assert!(replay_result_produce2.is_some());
    assert!(replay_result_produce2a.is_none());
    assert!(replay_result_consume2.is_none());
    assert!(replay_result_produce3.is_some());
    assert!(replay_result_produce3a.is_none());
    assert!(replay_result_consume3.is_none());
    assert!(replay_result_produce4.is_some());

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn creating_multiple_comm_events_with_peeking_a_produce_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();
    let datum = "datum1".to_string();

    let empty_point = space.create_checkpoint().unwrap();
    let result_consume1 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let result_produce = space.produce(channels[0].clone(), datum.clone(), false);
    let result_produce2 = space.produce(channels[0].clone(), datum.clone(), false);
    let result_consume2 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let result_produce3 = space.produce(channels[0].clone(), datum.clone(), false);
    let result_consume3 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let result_produce4 = space.produce(channels[0].clone(), datum.clone(), false);
    let result_consume4 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let result_produce5 = space.produce(channels[0].clone(), datum.clone(), false);
    let result_consume5 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let rig_point = space.create_checkpoint().unwrap();

    let expected_result = Some((
        ContResult {
            continuation: continuation.clone(),
            persistent: false,
            channels: channels.clone(),
            patterns: patterns.clone(),
            peek: true,
        },
        vec![RSpaceResult {
            channel: channels[0].clone(),
            matched_datum: datum.clone(),
            removed_datum: datum.clone(),
            persistent: false,
        }],
    ));

    assert!(result_consume1.is_none());
    assert_eq!(result_consume2, expected_result);
    assert_eq!(result_consume3, expected_result);
    assert_eq!(result_consume4, expected_result);
    assert_eq!(result_consume5, expected_result);
    assert_eq!(result_produce, expected_result);
    assert!(result_produce2.is_none());
    assert!(result_produce3.is_none());
    assert!(result_produce4.is_none());
    assert!(result_produce5.is_none());

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    let replay_result_consume1 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let replay_result_produce =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let replay_result_produce2 =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let replay_result_consume2 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let replay_result_produce3 =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let replay_result_consume3 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let replay_result_produce4 =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let replay_result_consume4 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let replay_result_produce5 =
        replay_space.replay_produce(channels[0].clone(), datum.clone(), false);
    let replay_result_consume5 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::from([0]),
    );
    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_result_consume1, result_consume1);
    assert_eq!(replay_result_consume2, result_consume2);
    assert_eq!(replay_result_consume3, result_consume3);
    assert_eq!(replay_result_consume4, result_consume4);
    assert_eq!(replay_result_consume5, result_consume5);
    assert_eq!(replay_result_produce, result_produce);
    assert_eq!(replay_result_produce2, result_produce2);
    assert_eq!(replay_result_produce3, result_produce3);
    assert_eq!(replay_result_produce4, result_produce4);
    assert_eq!(replay_result_produce5, result_produce5);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn picking_n_datums_from_m_waiting_datums_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let n = 5;
    let m = 10;
    let range: Vec<i32> = (n..m).collect();

    fn consume_many<F, G>(
        space: &mut RSpace<String, Pattern, String, String>,
        range: Vec<i32>,
        channels_creator: F,
        patterns: &Vec<Pattern>,
        continuation_creator: G,
        persist: bool,
        peeks: &BTreeSet<i32>,
    ) -> Vec<Option<(ContResult<String, Pattern, String>, Vec<RSpaceResult<String, String>>)>>
    where
        F: Fn(i32) -> Vec<String>,
        G: Fn(i32) -> String,
    {
        let mut rng = thread_rng();
        let mut shuffled_range = range.clone();
        shuffled_range.shuffle(&mut rng);

        shuffled_range
            .into_iter()
            .map(|i| {
                let result = space.consume(
                    channels_creator(i),
                    patterns.clone(),
                    continuation_creator(i),
                    persist,
                    peeks.clone(),
                );
                result
            })
            .collect()
    }

    fn produce_many<F, A>(
        space: &mut RSpace<String, Pattern, String, String>,
        range: Vec<i32>,
        channel_creator: F,
        datum_creator: A,
        persist: bool,
    ) -> Vec<Option<(ContResult<String, Pattern, String>, Vec<RSpaceResult<String, String>>)>>
    where
        F: Fn(i32) -> String,
        A: Fn(i32) -> String,
    {
        let mut rng = thread_rng();
        let mut shuffled_range = range.clone();
        shuffled_range.shuffle(&mut rng);

        shuffled_range
            .into_iter()
            .map(|i| {
                let result = space.produce(channel_creator(i), datum_creator(i), persist);
                result
            })
            .collect()
    }

    fn replay_consume_many<F, G>(
        space: &mut RSpace<String, Pattern, String, String>,
        range: Vec<i32>,
        channels_creator: F,
        patterns: &Vec<Pattern>,
        continuation_creator: G,
        persist: bool,
        peeks: &BTreeSet<i32>,
    ) -> Vec<Option<(ContResult<String, Pattern, String>, Vec<RSpaceResult<String, String>>)>>
    where
        F: Fn(i32) -> Vec<String>,
        G: Fn(i32) -> String,
    {
        let mut rng = thread_rng();
        let mut shuffled_range = range.clone();
        shuffled_range.shuffle(&mut rng);

        shuffled_range
            .into_iter()
            .map(|i| {
                let result = space.replay_consume(
                    channels_creator(i),
                    patterns.clone(),
                    continuation_creator(i),
                    persist,
                    peeks.clone(),
                );
                result
            })
            .collect()
    }

    fn replay_produce_many<F, A>(
        space: &mut RSpace<String, Pattern, String, String>,
        range: Vec<i32>,
        channel_creator: F,
        datum_creator: A,
        persist: bool,
    ) -> Vec<Option<(ContResult<String, Pattern, String>, Vec<RSpaceResult<String, String>>)>>
    where
        F: Fn(i32) -> String,
        A: Fn(i32) -> String,
    {
        let mut rng = thread_rng();
        let mut shuffled_range = range.clone();
        shuffled_range.shuffle(&mut rng);

        shuffled_range
            .into_iter()
            .map(|i| {
                let result = space.replay_produce(channel_creator(i), datum_creator(i), persist);
                result
            })
            .collect()
    }

    // function that takes one argument and always returns the last argument as a result
    fn kp<A, B: Clone>(x: B) -> impl Fn(A) -> B {
        move |_| x.clone()
    }

    fn datum_creator(i: i32) -> String {
        format!("datum{}", i)
    }

    fn continuation_creator(i: i32) -> String {
        format!("continuation{}", i)
    }

    let empty_point = space.create_checkpoint().unwrap();
    let _ = produce_many(&mut space, range.clone(), kp("ch1".to_string()), datum_creator, true);
    let results = consume_many(
        &mut space,
        range.clone(),
        kp(vec!["ch1".to_string()]),
        &vec![Pattern::Wildcard],
        continuation_creator,
        false,
        &BTreeSet::default(),
    );

    let rig_point = space.create_checkpoint().unwrap();
    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    let _ = replay_produce_many(
        &mut replay_space,
        range.clone(),
        kp("ch1".to_string()),
        datum_creator,
        true,
    );

    let replay_results = replay_consume_many(
        &mut replay_space,
        range,
        kp(vec!["ch1".to_string()]),
        &vec![Pattern::Wildcard],
        continuation_creator,
        false,
        &BTreeSet::default(),
    );
    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert!(check_same_elements(replay_results, results));
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn a_matched_continuation_defined_for_multiple_channels_some_peeked_should_replay_correctly()
{
    let (mut space, mut replay_space) = fixture().await;
    let mut rng = thread_rng();

    let amount_of_channels = 10;
    let amount_of_peeked_channels = 5;

    let channels: Vec<String> = (0..amount_of_channels)
        .map(|i| format!("channel{}", i))
        .collect();
    let patterns: Vec<Pattern> = channels.iter().map(|_| Pattern::Wildcard).collect();
    let continuation = "continuation".to_string();
    let peeks: BTreeSet<i32> = (0..amount_of_peeked_channels).collect();
    let mut produces: Vec<String> = channels.clone();
    produces.shuffle(&mut rng);

    fn consume_and_produce(
        space: &mut RSpace<String, Pattern, String, String>,
        channels: &Vec<String>,
        patterns: &Vec<Pattern>,
        continuation: &String,
        peeks: &BTreeSet<i32>,
        produces: &Vec<String>,
    ) -> Vec<Option<(ContResult<String, Pattern, String>, Vec<RSpaceResult<String, String>>)>> {
        let mut results = vec![];
        let _ = space.consume(
            channels.clone(),
            patterns.clone(),
            continuation.clone(),
            false,
            peeks.clone(),
        );

        for ch in produces {
            let result = space.produce(ch.clone(), format!("datum-{}", ch), false);
            results.push(result);
        }
        results
    }

    fn replay_consume_and_produce(
        space: &mut RSpace<String, Pattern, String, String>,
        channels: &Vec<String>,
        patterns: &Vec<Pattern>,
        continuation: &String,
        peeks: &BTreeSet<i32>,
        produces: &Vec<String>,
    ) -> Vec<Option<(ContResult<String, Pattern, String>, Vec<RSpaceResult<String, String>>)>> {
        let mut results = vec![];
        let _ = space.replay_consume(
            channels.clone(),
            patterns.clone(),
            continuation.clone(),
            false,
            peeks.clone(),
        );

        for ch in produces {
            let result = space.replay_produce(ch.clone(), format!("datum-{}", ch), false);
            results.push(result);
        }
        results
    }

    let empty_point = space.create_checkpoint().unwrap();
    let rs =
        consume_and_produce(&mut space, &channels, &patterns, &continuation, &peeks, &produces);
    assert_eq!(rs.iter().flatten().count(), 1);

    for i in 0..amount_of_channels {
        let ch = format!("channel{}", i);
        let data = space.store.get_data(&ch);
        if !peeks.contains(&i) {
            assert_eq!(data.len(), 0);
        }
    }

    let rig_point = space.create_checkpoint().unwrap();
    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    let rrs = replay_consume_and_produce(
        &mut replay_space,
        &channels,
        &patterns,
        &continuation,
        &peeks,
        &produces,
    );
    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(rs, rrs);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn picking_n_datums_from_m_persistent_waiting_datums_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();

    let range = (1..10).collect::<Vec<_>>();
    for i in &range {
        let _ = space.produce("ch1".to_string(), format!("datum{}", i), true);
    }

    let mut results = vec![];
    for i in &range {
        let result = space.consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range {
        let _ = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), true);
    }

    let mut replay_results = vec![];
    for i in &range {
        let result = replay_space.replay_consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn picking_n_continuations_from_m_waiting_continuations_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();
    let range = (1..10).collect::<Vec<_>>();
    for i in &range {
        let _ = space.consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    let mut results = vec![];
    for i in &range {
        let result = space.produce("ch1".to_string(), format!("datum{}", i), false);
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range {
        let _ = replay_space.replay_consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    let mut replay_results = vec![];
    for i in &range {
        let result = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), false);
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn picking_n_continuations_from_m_persistent_waiting_continuations_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();

    let range = (1..10).collect::<Vec<_>>();
    for i in &range {
        let _ = space.consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            true,
            BTreeSet::new(),
        );
    }

    let mut results = vec![];
    for i in &range {
        let result = space.produce("ch1".to_string(), format!("datum{}", i), false);
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range {
        let _ = replay_space.replay_consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            true,
            BTreeSet::new(),
        );
    }

    let mut replay_results = vec![];
    for i in &range {
        let result = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), false);
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn pick_n_continuations_from_m_waiting_continuations_stored_at_two_channels_should_replay_correctly(
) {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();

    let range = (1..10).collect::<Vec<_>>();
    for i in &range {
        let _ = space.consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    for i in &range {
        let _ = space.produce("ch1".to_string(), format!("datum{}", i), false);
    }

    let mut results = vec![];
    for i in &range {
        let result = space.produce("ch2".to_string(), format!("datum{}", i), false);
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range {
        let _ = replay_space.replay_consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    for i in &range {
        let _ = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), false);
    }

    let mut replay_results = vec![];
    for i in &range {
        let result = replay_space.replay_produce("ch2".to_string(), format!("datum{}", i), false);
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn picking_n_datums_from_m_waiting_datums_while_doing_a_bunch_of_other_junk_should_replay_correctly(
) {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();

    let range = (1..10).collect::<Vec<_>>();
    for i in &range {
        let _ = space.produce("ch1".to_string(), format!("datum{}", i), false);
    }

    for i in 11..20 {
        let _ = space.consume(
            vec![format!("ch{}", i)],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    for i in 21..30 {
        let _ = space.produce(format!("ch{}", i), format!("datum{}", i), false);
    }

    let mut results = vec![];
    for i in &range {
        let result = space.consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range {
        let _ = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), false);
    }

    for i in 11..20 {
        let _ = replay_space.replay_consume(
            vec![format!("ch{}", i)],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    for i in 21..30 {
        let _ = replay_space.replay_produce(format!("ch{}", i), format!("datum{}", i), false);
    }

    let mut replay_results = vec![];
    for i in &range {
        let result = replay_space.replay_consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn picking_n_continuations_from_m_persistent_waiting_continuations_while_doing_a_bunch_of_other_junk_should_replay_correctly(
) {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();

    let range = (1..10).collect::<Vec<_>>();
    for i in &range {
        let _ = space.consume(
            vec![format!("ch{}", i)],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            true,
            BTreeSet::new(),
        );
    }

    for i in 11..20 {
        let _ = space.produce("ch1".to_string(), format!("datum{}", i), false);
    }

    for i in 21..30 {
        let _ = space.consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    let mut results = vec![];
    for i in &range {
        let result = space.produce(format!("ch{}", i), format!("datum{}", i), false);
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range {
        let _ = replay_space.replay_consume(
            vec![format!("ch{}", i)],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            true,
            BTreeSet::new(),
        );
    }

    for i in 11..20 {
        let _ = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), false);
    }

    for i in 21..30 {
        let _ = replay_space.replay_consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::new(),
        );
    }

    let mut replay_results = vec![];
    for i in &range {
        let result = replay_space.replay_produce(format!("ch{}", i), format!("datum{}", i), false);
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn peeking_data_stored_at_two_channels_in_100_continuations_should_replay_correctly() {
    let (mut space, mut replay_space) = fixture().await;

    let empty_point = space.create_checkpoint().unwrap();

    let range1 = (0..100).collect::<Vec<_>>();
    let range2 = (0..3).collect::<Vec<_>>();
    let range3 = (0..5).collect::<Vec<_>>();

    for i in &range2 {
        let _ = space.produce("ch1".to_string(), format!("datum{}", i), false);
    }

    for i in &range3 {
        let _ = space.produce("ch2".to_string(), format!("datum{}", i), false);
    }

    let mut results = vec![];
    for i in &range1 {
        let result = space.consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::from([0, 1]),
        );
        results.push(result);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    for i in &range2 {
        let _ = replay_space.replay_produce("ch1".to_string(), format!("datum{}", i), false);
    }

    for i in &range3 {
        let _ = replay_space.replay_produce("ch2".to_string(), format!("datum{}", i), false);
    }

    let mut replay_results = vec![];
    for i in &range1 {
        let result = replay_space.replay_consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            format!("continuation{}", i),
            false,
            BTreeSet::from([0, 1]),
        );
        replay_results.push(result);
    }

    let final_point = replay_space.replay_create_checkpoint().unwrap();

    assert_eq!(replay_results, results);
    assert_eq!(final_point.root, rig_point.root);
    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn replay_rspace_should_correctly_remove_things_from_replay_data() {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();
    let datum = "datum".to_string();

    let empty_point = space.create_checkpoint().unwrap();

    let cr = Consume::create(channels.clone(), patterns.clone(), continuation.clone(), false);

    for _ in 0..2 {
        let _ = space.consume(
            channels.clone(),
            patterns.clone(),
            continuation.clone(),
            false,
            BTreeSet::new(),
        );
    }

    for _ in 0..2 {
        let _ = space.produce(channels[0].clone(), datum.clone(), false);
    }

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);

    assert_eq!(
        replay_space
            .replay_data
            .map
            .get(&IOEvent::Consume(cr.clone()))
            .unwrap()
            .len(),
        2
    );

    for _ in 0..2 {
        let _ = replay_space.replay_consume(
            channels.clone(),
            patterns.clone(),
            continuation.clone(),
            false,
            BTreeSet::new(),
        );
    }

    let _ = replay_space.replay_produce(channels[0].clone(), datum.clone(), false);

    assert_eq!(
        replay_space
            .replay_data
            .map
            .get(&IOEvent::Consume(cr.clone()))
            .unwrap()
            .len(),
        1
    );

    let _ = replay_space.replay_produce(channels[0].clone(), datum.clone(), false);

    assert!(replay_space
        .replay_data
        .map
        .get(&IOEvent::Consume(cr))
        .is_none());
}

#[tokio::test]
async fn producing_should_return_same_stable_checkpoint_root_hashes() {
    async fn process(indices: Vec<i32>) -> Blake2b256Hash {
        let (mut space, _) = fixture().await;

        for i in indices {
            let _ = space.produce("ch1".to_string(), format!("datum{}", i), false);
        }

        space.create_checkpoint().unwrap().root
    }

    let cp1 = process((0..10).collect()).await;
    let cp2 = process((0..10).rev().collect()).await;

    assert_eq!(cp1, cp2);
}

#[tokio::test]
async fn an_install_should_be_available_after_resetting_to_a_checkpoint() {
    let (mut space, mut replay_space) = fixture().await;

    let channel = "ch1".to_string();
    let datum = "datum1".to_string();
    let key = vec![channel.clone()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();

    let _ = space.install(key.clone(), patterns.clone(), continuation.clone());
    let _ = replay_space.install(key.clone(), patterns.clone(), continuation.clone());

    let produce1 = space.produce(channel.clone(), datum.clone(), false);
    assert!(produce1.is_some());

    let after_produce = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(after_produce.root, after_produce.log);

    let produce2 = replay_space.produce(channel, datum, false);
    assert!(produce2.is_some());
}

#[tokio::test]
async fn reset_should_empty_the_replay_store_and_reset_the_replay_trie_updates_log_and_reset_the_replay_data(
) {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();

    let empty_point = space.create_checkpoint().unwrap();

    let consume1 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );
    assert!(consume1.is_none());

    let rig_point = space.create_checkpoint().unwrap();

    let _ = replay_space.rig_and_reset(empty_point.root.clone(), rig_point.log);

    let consume2 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );
    assert!(consume2.is_none());

    assert!(!replay_space.store.is_empty());
    assert_eq!(
        replay_space
            .store
            .changes()
            .into_iter()
            .filter_map(|ht_action| {
                match ht_action {
                    HotStoreAction::Insert(InsertAction::InsertContinuations(cont)) => Some(cont),
                    _ => None,
                }
            })
            .collect::<Vec<InsertContinuations<String, Pattern, String>>>()
            .len(),
        1
    );

    let _ = replay_space.reset(empty_point.root);
    assert!(replay_space.store.is_empty());
    assert!(replay_space.replay_data.is_empty());

    let checkpoint1 = replay_space.replay_create_checkpoint().unwrap();
    assert!(checkpoint1.log.is_empty());
}

#[tokio::test]
async fn clear_should_empty_the_replay_store_reset_the_replay_event_log_reset_the_replay_trie_updates_log_and_reset_the_replay_data(
) {
    let (mut space, mut replay_space) = fixture().await;

    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();

    let empty_point = space.create_checkpoint().unwrap();
    let consume1 = space.consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );
    assert!(consume1.is_none());

    let rig_point = space.create_checkpoint().unwrap();
    let _ = replay_space.rig_and_reset(empty_point.root, rig_point.log);
    let consume2 = replay_space.replay_consume(
        channels.clone(),
        patterns.clone(),
        continuation.clone(),
        false,
        BTreeSet::new(),
    );
    assert!(consume2.is_none());
    assert!(!replay_space.store.is_empty());
    assert_eq!(
        replay_space
            .store
            .changes()
            .into_iter()
            .filter_map(|action| {
                if let HotStoreAction::Insert(insert) = action {
                    if let InsertAction::InsertContinuations(conts) = insert {
                        Some(conts)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .len(),
        1
    );

    let checkpoint0 = replay_space.replay_create_checkpoint().unwrap();
    assert!(checkpoint0.log.is_empty()); // we don't record trace logs in ReplayRspace

    let _ = replay_space.clear();
    assert!(replay_space.store.is_empty());
    assert!(replay_space.replay_data.is_empty());

    let checkpoint1 = replay_space.replay_create_checkpoint().unwrap();
    assert!(checkpoint1.log.is_empty());
}

#[tokio::test]
async fn replay_should_not_allow_for_ambiguous_executions() {
    let (mut space, mut replay_space) = fixture().await;

    let channel1 = "ch1".to_string();
    let channel2 = "ch2".to_string();
    let key1 = vec!["ch1".to_string(), "ch2".to_string()];
    let patterns = vec![Pattern::Wildcard, Pattern::Wildcard];
    let continuation1 = "continuation1".to_string();
    let continuation2 = "continuation2".to_string();
    let data1 = "datum1".to_string();
    let data2 = "datum2".to_string();
    let data3 = "datum2".to_string();

    let empty_point = space.create_checkpoint().unwrap();
    assert_eq!(space.produce(channel1.clone(), data3.clone(), false), None);
    assert_eq!(space.produce(channel1.clone(), data3.clone(), false), None);
    assert_eq!(space.produce(channel2.clone(), data1.clone(), false), None);

    assert!(space
        .consume(key1.clone(), patterns.clone(), continuation1.clone(), false, BTreeSet::new(),)
        .is_some());

    //continuation1 produces data1 on ch2
    assert!(space
        .produce(channel2.clone(), data1.clone(), false)
        .is_none());
    assert!(space
        .consume(key1.clone(), patterns.clone(), continuation2.clone(), false, BTreeSet::default())
        .is_some());
    //continuation2 produces data2 on ch2
    assert!(space
        .produce(channel2.clone(), data2.clone(), false)
        .is_none());
    let after_play = space.create_checkpoint().unwrap();

    //rig
    let _ = replay_space.rig_and_reset(empty_point.root, after_play.log);

    assert!(replay_space
        .replay_produce(channel1.clone(), data3.clone(), false)
        .is_none());
    assert!(replay_space
        .replay_produce(channel1, data3, false)
        .is_none());
    assert!(replay_space
        .replay_produce(channel2.clone(), data1.clone(), false)
        .is_none());
    assert!(replay_space
        .replay_consume(key1.clone(), patterns.clone(), continuation2, false, BTreeSet::default())
        .is_none());

    assert!(replay_space
        .replay_consume(key1, patterns, continuation1, false, BTreeSet::default())
        .is_some());

    //continuation1 produces data1 on ch2
    assert!(replay_space
        .replay_produce(channel2.clone(), data1, false)
        .is_some());
    //continuation2 produces data2 on ch2
    assert!(replay_space
        .replay_produce(channel2, data2, false)
        .is_none());

    assert!(replay_space.replay_data.is_empty());
}

#[tokio::test]
async fn check_replay_data_should_proceed_if_replay_data_is_empty() {
    let (_space, replay_space) = fixture().await;
    replay_space.check_replay_data();
}

#[tokio::test]
#[should_panic(expected = "Unused COMM event: replayData multimap has 2 elements left")]
async fn check_replay_data_should_panic_if_replay_data_contains_elements() {
    let (mut space, mut replay_space) = fixture().await;
    let channels = vec!["ch1".to_string()];
    let patterns = vec![Pattern::Wildcard];
    let continuation = "continuation".to_string();
    let datum = "datum1".to_string();

    let _ = space.consume(channels.clone(), patterns, continuation, false, BTreeSet::new());
    let _ = space.produce(channels[0].clone(), datum, false);
    let c = space.create_checkpoint().unwrap();
    let _ = replay_space.rig_and_reset(c.root, c.log);
    let _res = replay_space.check_replay_data();
}

type StateSetup =
    (RSpace<String, Pattern, String, String>, RSpace<String, Pattern, String, String>);

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

    let rspace =
        RSpaceInstances::apply(history_repo.clone(), hot_store, Arc::new(Box::new(StringMatch)));

    let history_cache: HotStoreState<String, Pattern, String, String> = HotStoreState::default();
    let replay_store = {
        let hr = history_reader.base();
        HotStoreInstances::create_from_hs_and_hr(history_cache, hr)
    };

    let replay_rspace: RSpace<String, Pattern, String, String> =
        RSpaceInstances::apply(history_repo, replay_store, Arc::new(Box::new(StringMatch)));

    (rspace, replay_rspace)
}
