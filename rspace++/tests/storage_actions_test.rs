// See rspace/src/test/scala/coop/rchain/rspace/StorageActionsTests.scala
use rspace_plus_plus::rspace::internal::Datum;
use rspace_plus_plus::rspace::internal::{ContResult, RSpaceResult};
use rspace_plus_plus::rspace::matcher::r#match::Match;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceInstances};
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use rspace_plus_plus::rspace::shared::lmdb_dir_store_manager::GB;
use rspace_plus_plus::rspace::shared::rspace_store_manager::mk_rspace_store_manager;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet, LinkedList};
use std::hash::Hash;

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

// We only care that both vectors contain the same elements, not their ordering
fn check_same_elements<T: Hash + Eq>(vec1: Vec<T>, vec2: Vec<T>) -> bool {
    let set1: HashSet<_> = vec1.into_iter().collect();
    let set2: HashSet<_> = vec2.into_iter().collect();
    set1 == set2
}

// See rspace/src/main/scala/coop/rchain/rspace/util/package.scala
fn unpack_tuple<C, P, K: Clone, R: Clone>(
    tuple: &(ContResult<C, P, K>, Vec<RSpaceResult<C, R>>),
) -> (K, Vec<R>) {
    match tuple {
        (ContResult { continuation, .. }, data) => (
            continuation.clone(),
            data.into_iter()
                .map(|result| result.matched_datum.clone())
                .collect(),
        ),
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/util/package.scala
fn run_k<C, P>(
    cont: Option<(ContResult<C, P, StringsCaptor>, Vec<RSpaceResult<C, String>>)>,
) -> Vec<Vec<String>> {
    let mut cont_unwrapped = cont.unwrap();
    let unpacked_tuple = unpack_tuple(&cont_unwrapped);
    cont_unwrapped.0.continuation.run_k(unpacked_tuple.1);
    let cont_results = cont_unwrapped.0.continuation.results();
    let cloned_results: Vec<Vec<String>> = cont_results
        .iter()
        .map(|res| res.iter().map(|s| s.to_string()).collect())
        .collect();
    cloned_results
}

async fn create_rspace() -> RSpace<String, Pattern, String, StringsCaptor, StringMatch> {
    let mut kvm = mk_rspace_store_manager("./tests/lmdb".into(), 1 * GB);
    let store = kvm.r_space_stores().await.unwrap();

    RSpaceInstances::create(store, StringMatch).await.unwrap()
}

// NOTE: Not implementing test checks for Scala's side 'insertData' and 'insertContinuations'
#[tokio::test]
async fn produce_should_persist_data_in_store() {
    let rspace = create_rspace().await;

    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r = rspace
        .produce(key[0].clone(), "datum".to_string(), false)
        .await;
    let data = rspace.store.get_data(&channel).await;
    assert_eq!(data, vec![Datum::create(channel, "datum".to_string(), false)]);

    let cont = rspace.store.get_continuations(key).await;
    assert_eq!(cont.len(), 0);
    assert!(r.is_none());
}

#[tokio::test]
async fn producing_twice_on_same_channel_should_persist_two_pieces_of_data_in_store() {
    let rspace = create_rspace().await;
    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r1 = rspace
        .produce(key[0].clone(), "datum1".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&channel).await;
    assert_eq!(d1, vec![Datum::create(channel.clone(), "datum1".to_string(), false)]);

    let wc1 = rspace.store.get_continuations(key.clone()).await;
    assert_eq!(wc1.len(), 0);
    assert!(r1.is_none());

    let r2 = rspace
        .produce(key[0].clone(), "datum2".to_string(), false)
        .await;
    let d2 = rspace.store.get_data(&channel).await;
    assert!(check_same_elements(
        d2,
        vec![
            Datum::create(channel.clone(), "datum1".to_string(), false),
            Datum::create(channel, "datum2".to_string(), false)
        ]
    ));

    let wc2 = rspace.store.get_continuations(key.clone()).await;
    assert_eq!(wc2.len(), 0);
    assert!(r2.is_none());
}

#[tokio::test]
async fn consuming_on_one_channel_should_persist_continuation_in_store() {
    let rspace = create_rspace().await;
    let channel = "ch1".to_string();
    let key = vec![channel.clone()];
    let patterns = vec![Pattern::Wildcard];

    let r = rspace
        .consume(key.clone(), patterns, StringsCaptor::new(), false, BTreeSet::default())
        .await;
    let d1 = rspace.store.get_data(&channel).await;
    assert_eq!(d1.len(), 0);

    let c1 = rspace.store.get_continuations(key).await;
    assert_ne!(c1.len(), 0);
    assert!(r.is_none());
}

#[tokio::test]
async fn consuming_on_three_channels_should_persist_continuation_in_store() {
    let rspace = create_rspace().await;
    let key = vec!["ch1".to_string(), "ch2".to_string(), "ch3".to_string()];
    let patterns = vec![Pattern::Wildcard, Pattern::Wildcard, Pattern::Wildcard];

    let r = rspace
        .consume(key.clone(), patterns, StringsCaptor::new(), false, BTreeSet::default())
        .await;
    let futures: Vec<_> = key.iter().map(|k| rspace.store.get_data(k)).collect();
    let d: Vec<Vec<Datum<String>>> = futures::future::join_all(futures).await;
    for seq in &d {
        assert!(seq.is_empty(), "d should be empty");
    }

    let c1 = rspace.store.get_continuations(key).await;
    assert_ne!(c1.len(), 0);
    assert!(r.is_none());
}

#[tokio::test]
async fn producing_then_consuming_on_same_channel_should_return_continuation_and_data() {
    let rspace = create_rspace().await;
    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r1 = rspace
        .produce(channel.clone(), "datum".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&channel).await;
    assert_eq!(d1, vec![Datum::create(channel.clone(), "datum".to_string(), false)]);

    let c1 = rspace.store.get_continuations(key.clone()).await;
    assert_eq!(c1.len(), 0);
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let d2 = rspace.store.get_data(&channel).await;
    assert_eq!(d2.len(), 0);

    let c2 = rspace.store.get_continuations(key).await;
    assert_eq!(c2.len(), 0);
    assert!(r2.is_some());

    let cont_results = run_k(r2);
    assert!(check_same_elements(cont_results, vec![vec!["datum".to_string()]]));
}

#[tokio::test]
async fn producing_then_consuming_on_same_channel_with_peek_should_return_continuation_and_data_and_remove_peeked_data(
) {
    let rspace = create_rspace().await;
    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r1 = rspace
        .produce(channel.clone(), "datum".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&channel).await;
    assert_eq!(d1, vec![Datum::create(channel.clone(), "datum".to_string(), false)]);

    let c1 = rspace.store.get_continuations(key.clone()).await;
    assert_eq!(c1.len(), 0);
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            std::iter::once(0).collect(),
        )
        .await;
    let d2 = rspace.store.get_data(&channel).await;
    assert_eq!(d2.len(), 0);

    let c2 = rspace.store.get_continuations(key).await;
    assert_eq!(c2.len(), 0);
    assert!(r2.is_some());

    let cont_results = run_k(r2);
    assert!(check_same_elements(cont_results, vec![vec!["datum".to_string()]]));
}

#[tokio::test]
async fn consuming_then_producing_on_same_channel_with_peek_should_return_continuation_and_data_and_remove_peeked_data(
) {
    let rspace = create_rspace().await;
    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r1 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            std::iter::once(0).collect(),
        )
        .await;
    assert!(r1.is_none());
    let c1 = rspace.store.get_continuations(key.clone()).await;
    assert_eq!(c1.len(), 1);

    let r2 = rspace
        .produce(channel.clone(), "datum".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&channel).await;
    assert!(d1.is_empty());

    let c2 = rspace.store.get_continuations(key).await;
    assert_eq!(c2.len(), 0);
    assert!(r2.is_some());

    let cont_results = run_k(r2);
    assert!(check_same_elements(cont_results, vec![vec!["datum".to_string()]]));
}

#[tokio::test]
async fn consuming_then_producing_on_same_channel_with_persistent_flag_should_return_continuation_and_data_and_not_insert_persistent_data(
) {
    let rspace = create_rspace().await;
    let channel = "ch1".to_string();
    let key = vec![channel.clone()];

    let r1 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    assert!(r1.is_none());
    let c1 = rspace.store.get_continuations(key.clone()).await;
    assert_eq!(c1.len(), 1);

    let r2 = rspace
        .produce(channel.clone(), "datum".to_string(), true)
        .await;
    let d1 = rspace.store.get_data(&channel).await;
    assert!(d1.is_empty());

    let c2 = rspace.store.get_continuations(key).await;
    assert_eq!(c2.len(), 0);
    assert!(r2.is_some());

    let cont_results = run_k(r2);
    assert!(check_same_elements(cont_results, vec![vec!["datum".to_string()]]));
}

#[tokio::test]
async fn producing_three_times_then_consuming_three_times_should_work() {
    let rspace = create_rspace().await;
    let possible_cont_results =
        vec![vec!["datum1".to_string()], vec!["datum2".to_string()], vec!["datum3".to_string()]];

    let r1 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r2 = rspace
        .produce("ch1".to_string(), "datum2".to_string(), false)
        .await;
    let r3 = rspace
        .produce("ch1".to_string(), "datum3".to_string(), false)
        .await;
    assert!(r1.is_none());
    assert!(r2.is_none());
    assert!(r3.is_none());

    let r4 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let cont_results_r4 = run_k(r4);
    assert!(possible_cont_results
        .iter()
        .any(|v| cont_results_r4.contains(v)));

    let r5 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let cont_results_r5 = run_k(r5);
    assert!(possible_cont_results
        .iter()
        .any(|v| cont_results_r5.contains(v)));

    let r6 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let cont_results_r6 = run_k(r6);
    assert!(possible_cont_results
        .iter()
        .any(|v| cont_results_r6.contains(v)));
}

// NOTE: Still not quite sure how this one works
//       The test is setup correctly though
#[tokio::test]
async fn producing_on_channel_then_consuming_on_that_channel_and_another_then_producing_on_other_channel_should_return_continuation_and_all_data(
) {
    let rspace = create_rspace().await;
    let produce_key_1 = vec!["ch1".to_string()];
    let produce_key_2 = vec!["ch2".to_string()];
    let consume_key = vec!["ch1".to_string(), "ch2".to_string()];
    let consume_pattern = vec![Pattern::Wildcard, Pattern::Wildcard];

    let r1 = rspace
        .produce(produce_key_1[0].clone(), "datum1".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&produce_key_1[0]).await;
    assert_eq!(d1, vec![Datum::create(&produce_key_1[0], "datum1".to_string(), false)]);

    let c1 = rspace.store.get_continuations(produce_key_1.clone()).await;
    assert!(c1.is_empty());
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            consume_key.clone(),
            consume_pattern,
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let d2 = rspace.store.get_data(&produce_key_1[0]).await;
    assert_eq!(d2, vec![Datum::create(&produce_key_1[0], "datum1".to_string(), false)]);

    let c2 = rspace.store.get_continuations(produce_key_1.clone()).await;
    let d3 = rspace.store.get_data(&produce_key_2[0]).await;
    let c3 = rspace.store.get_continuations(consume_key.clone()).await;
    assert!(c2.is_empty());
    assert!(d3.is_empty());
    assert_ne!(c3.len(), 0);
    assert!(r2.is_none());

    let r3 = rspace
        .produce(produce_key_2[0].clone(), "datum2".to_string(), false)
        .await;
    let c4 = rspace.store.get_continuations(consume_key).await;
    let d4 = rspace.store.get_data(&produce_key_1[0]).await;
    let d5 = rspace.store.get_data(&produce_key_2[0]).await;
    assert!(c4.is_empty());
    assert!(d4.is_empty());
    assert!(d5.is_empty());
    assert!(r3.is_some());

    let cont_results = run_k(r3);
    assert!(check_same_elements(
        cont_results,
        vec![vec!["datum1".to_string(), "datum2".to_string()]]
    ));
}

#[tokio::test]
async fn producing_on_three_channels_then_consuming_once_should_return_cont_and_all_data() {
    let rspace = create_rspace().await;
    let produce_key_1 = vec!["ch1".to_string()];
    let produce_key_2 = vec!["ch2".to_string()];
    let produce_key_3 = vec!["ch3".to_string()];
    let consume_key = vec!["ch1".to_string(), "ch2".to_string(), "ch3".to_string()];
    let patterns = vec![Pattern::Wildcard, Pattern::Wildcard, Pattern::Wildcard];

    let r1 = rspace
        .produce(produce_key_1[0].clone(), "datum1".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&produce_key_1[0]).await;
    assert_eq!(d1, vec![Datum::create(&produce_key_1[0], "datum1".to_string(), false)]);

    let c1 = rspace.store.get_continuations(produce_key_1).await;
    assert!(c1.is_empty());
    assert!(r1.is_none());

    let r2 = rspace
        .produce(produce_key_2[0].clone(), "datum2".to_string(), false)
        .await;
    let d2 = rspace.store.get_data(&produce_key_2[0]).await;
    assert_eq!(d2, vec![Datum::create(&produce_key_2[0], "datum2".to_string(), false)]);

    let c2 = rspace.store.get_continuations(produce_key_2).await;
    assert!(c2.is_empty());
    assert!(r2.is_none());

    let r3 = rspace
        .produce(produce_key_3[0].clone(), "datum3".to_string(), false)
        .await;
    let d3 = rspace.store.get_data(&produce_key_3[0]).await;
    assert_eq!(d3, vec![Datum::create(produce_key_3[0].clone(), "datum3".to_string(), false)]);

    let c3 = rspace.store.get_continuations(produce_key_3).await;
    assert!(c3.is_empty());
    assert!(r3.is_none());

    let r4 = rspace
        .consume(consume_key.clone(), patterns, StringsCaptor::new(), false, BTreeSet::default())
        .await;
    let futures: Vec<_> = consume_key
        .iter()
        .map(|k| rspace.store.get_data(k))
        .collect();
    let d4: Vec<Vec<Datum<String>>> = futures::future::join_all(futures).await;
    for seq in &d4 {
        assert!(seq.is_empty(), "d should be empty");
    }

    let c4 = rspace.store.get_continuations(consume_key).await;
    assert!(c4.is_empty());
    assert!(r4.is_some());

    let cont_results = run_k(r4);
    assert!(check_same_elements(
        cont_results,
        vec![vec!["datum1".to_string(), "datum2".to_string(), "datum3".to_string()]]
    ));
}

#[tokio::test]
async fn producing_then_consuming_three_times_on_same_channel_should_return_three_pairs_of_conts_and_data(
) {
    let rspace = create_rspace().await;
    let captor = StringsCaptor::new();
    let key = vec!["ch1".to_string()];

    let r1 = rspace
        .produce(key[0].clone(), "datum1".to_string(), false)
        .await;
    let r2 = rspace
        .produce(key[0].clone(), "datum2".to_string(), false)
        .await;
    let r3 = rspace
        .produce(key[0].clone(), "datum3".to_string(), false)
        .await;
    assert!(r1.is_none());
    assert!(r2.is_none());
    assert!(r3.is_none());

    let r4 = rspace
        .consume(key.clone(), vec![Pattern::Wildcard], captor.clone(), false, BTreeSet::default())
        .await;
    let r5 = rspace
        .consume(key.clone(), vec![Pattern::Wildcard], captor.clone(), false, BTreeSet::default())
        .await;
    let r6 = rspace
        .consume(key.clone(), vec![Pattern::Wildcard], captor, false, BTreeSet::default())
        .await;
    let c1 = rspace.store.get_continuations(key).await;
    assert!(c1.is_empty());

    let continuations = vec![r4.clone(), r5.clone(), r6.clone()];
    assert!(continuations.iter().all(Option::is_some));
    let cont_results_r4 = run_k(r4);
    let cont_results_r5 = run_k(r5);
    let cont_results_r6 = run_k(r6);
    let cont_results = [cont_results_r4, cont_results_r5, cont_results_r6].concat();
    assert!(check_same_elements(
        cont_results,
        vec![vec!["datum3".to_string()], vec!["datum2".to_string()], vec!["datum1".to_string()]]
    ));
}

#[tokio::test]
async fn consuming_then_producing_three_times_on_same_channel_should_return_conts_each_paired_with_distinct_data(
) {
    let rspace = create_rspace().await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;

    let r1 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r2 = rspace
        .produce("ch1".to_string(), "datum2".to_string(), false)
        .await;
    let r3 = rspace
        .produce("ch1".to_string(), "datum3".to_string(), false)
        .await;
    assert!(r1.is_some());
    assert!(r2.is_some());
    assert!(r3.is_some());

    let possible_cont_results =
        vec![vec!["datum1".to_string()], vec!["datum2".to_string()], vec!["datum3".to_string()]];
    let cont_results_r1 = run_k(r1);
    let cont_results_r2 = run_k(r2);
    let cont_results_r3 = run_k(r3);
    assert!(possible_cont_results
        .iter()
        .any(|v| cont_results_r1.contains(v)));
    assert!(possible_cont_results
        .iter()
        .any(|v| cont_results_r2.contains(v)));
    assert!(possible_cont_results
        .iter()
        .any(|v| cont_results_r3.contains(v)));

    assert!(!check_same_elements(cont_results_r1.clone(), cont_results_r2.clone()));
    assert!(!check_same_elements(cont_results_r1, cont_results_r3.clone()));
    assert!(!check_same_elements(cont_results_r2, cont_results_r3));
}

#[tokio::test]
async fn consuming_then_producing_three_times_on_same_channel_with_non_trivial_matches_should_return_three_conts_each_paired_with_matching_data(
) {
    let rspace = create_rspace().await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::StringMatch("datum1".to_string())],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::StringMatch("datum2".to_string())],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::StringMatch("datum3".to_string())],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;

    let r1 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r2 = rspace
        .produce("ch1".to_string(), "datum2".to_string(), false)
        .await;
    let r3 = rspace
        .produce("ch1".to_string(), "datum3".to_string(), false)
        .await;
    assert!(r1.is_some());
    assert!(r2.is_some());
    assert!(r3.is_some());

    assert_eq!(run_k(r1), vec![vec!["datum1"]]);
    assert_eq!(run_k(r2), vec![vec!["datum2"]]);
    assert_eq!(run_k(r3), vec![vec!["datum3"]]);
}

#[tokio::test]
async fn consuming_on_two_channels_then_producing_on_each_should_return_cont_with_both_data() {
    let rspace = create_rspace().await;
    let r1 = rspace
        .consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;

    let r2 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r3 = rspace
        .produce("ch2".to_string(), "datum2".to_string(), false)
        .await;

    assert!(r1.is_none());
    assert!(r2.is_none());
    assert!(r3.is_some());
    assert!(check_same_elements(run_k(r3), vec![vec!["datum1".to_string(), "datum2".to_string()]]))
}

#[tokio::test]
async fn joined_consume_with_same_channel_given_twice_followed_by_produce_should_not_error() {
    let rspace = create_rspace().await;
    let channels = vec!["ch1".to_string(), "ch1".to_string()];

    let r1 = rspace
        .consume(
            channels,
            vec![
                Pattern::StringMatch("datum1".to_string()),
                Pattern::StringMatch("datum1".to_string()),
            ],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let r2 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r3 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;

    assert!(r1.is_none());
    assert!(r2.is_none());
    assert!(r3.is_some());
    assert!(check_same_elements(run_k(r3), vec![vec!["datum1".to_string(), "datum1".to_string()]]));
}

#[tokio::test]
async fn consuming_then_producing_twice_on_same_channel_with_different_patterns_should_return_cont_with_expected_data(
) {
    let rspace = create_rspace().await;
    let channels = vec!["ch1".to_string(), "ch2".to_string()];

    let r1 = rspace
        .consume(
            channels.clone(),
            vec![
                Pattern::StringMatch("datum1".to_string()),
                Pattern::StringMatch("datum2".to_string()),
            ],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let r2 = rspace
        .consume(
            channels,
            vec![
                Pattern::StringMatch("datum3".to_string()),
                Pattern::StringMatch("datum4".to_string()),
            ],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;

    let r3 = rspace
        .produce("ch1".to_string(), "datum3".to_string(), false)
        .await;
    let r4 = rspace
        .produce("ch2".to_string(), "datum4".to_string(), false)
        .await;
    let r5 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r6 = rspace
        .produce("ch2".to_string(), "datum2".to_string(), false)
        .await;

    assert!(r1.is_none());
    assert!(r2.is_none());
    assert!(r3.is_none());
    assert!(r4.is_some());
    assert!(r5.is_none());
    assert!(r6.is_some());

    assert!(check_same_elements(run_k(r4), vec![vec!["datum3".to_string(), "datum4".to_string()]]));
    assert!(check_same_elements(run_k(r6), vec![vec!["datum1".to_string(), "datum2".to_string()]]));
}

#[tokio::test]
async fn consuming_and_producing_with_non_trivial_matches_should_work() {
    let rspace = create_rspace().await;

    let r1 = rspace
        .consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::StringMatch("datum1".to_string())],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let r2 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;

    assert!(r1.is_none());
    assert!(r2.is_none());

    let d1 = rspace.store.get_data(&"ch2".to_string()).await;
    assert!(d1.is_empty());
    let d2 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d2, vec![Datum::create("ch1".to_string(), "datum1".to_string(), false)]);

    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string(), "ch2".to_string()])
        .await;
    assert!(!c1.is_empty());
    let j1 = rspace.store.get_joins("ch1".to_string()).await;
    assert_eq!(j1, vec![vec!["ch1".to_string(), "ch2".to_string()]]);
    let j2 = rspace.store.get_joins("ch2".to_string()).await;
    assert_eq!(j2, vec![vec!["ch1".to_string(), "ch2".to_string()]]);
}

#[tokio::test]
async fn consuming_and_producing_twice_with_non_trivial_matches_should_work() {
    let rspace = create_rspace().await;

    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::StringMatch("datum1".to_string())],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let _ = rspace
        .consume(
            vec!["ch2".to_string()],
            vec![Pattern::StringMatch("datum2".to_string())],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;

    let r3 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r4 = rspace
        .produce("ch2".to_string(), "datum2".to_string(), false)
        .await;

    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(d1.is_empty());
    let d2 = rspace.store.get_data(&"ch2".to_string()).await;
    assert!(d2.is_empty());

    assert!(check_same_elements(run_k(r3), vec![vec!["datum1".to_string()]]));
    assert!(check_same_elements(run_k(r4), vec![vec!["datum2".to_string()]]));
}

#[tokio::test]
async fn consuming_on_two_channels_then_consuming_on_one_then_producing_on_both_separately_should_return_cont_paired_with_one_data(
) {
    let rspace = create_rspace().await;

    let _ = rspace
        .consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let _ = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;

    let r3 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r4 = rspace
        .produce("ch2".to_string(), "datum2".to_string(), false)
        .await;

    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string(), "ch2".to_string()])
        .await;
    assert!(!c1.is_empty());
    let c2 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!((c2.is_empty()));
    let c3 = rspace
        .store
        .get_continuations(vec!["ch2".to_string()])
        .await;
    assert!(c3.is_empty());

    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(d1.is_empty());
    let d2 = rspace.store.get_data(&"ch2".to_string()).await;
    assert_eq!(d2, vec![Datum::create("ch2".to_string(), "datum2".to_string(), false)]);

    assert!(r3.is_some());
    assert!(r4.is_none());
    assert!(check_same_elements(run_k(r3), vec![vec!["datum1".to_string()]]));

    let j1 = rspace.store.get_joins("ch1".to_string()).await;
    assert_eq!(j1, vec![vec!["ch1".to_string(), "ch2".to_string()]]);
    let j2 = rspace.store.get_joins("ch2".to_string()).await;
    assert_eq!(j2, vec![vec!["ch1".to_string(), "ch2".to_string()]]);
}

/* Persist tests */
#[tokio::test]
async fn producing_then_persistent_consume_on_same_channel_should_return_cont_and_data() {
    let rspace = create_rspace().await;
    let key = vec!["ch1".to_string()];

    let r1 = rspace
        .produce(key[0].clone(), "datum".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&key[0]).await;
    assert_eq!(d1, vec![Datum::create(key[0].clone(), "datum".to_string(), false)]);
    let c1 = rspace.store.get_continuations(key.clone()).await;
    assert!(c1.is_empty());
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2), vec![vec!["datum".to_string()]]));

    let r3 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    let d2 = rspace.store.get_data(&key[0]).await;
    assert!(d2.is_empty());
    let c2 = rspace.store.get_continuations(key).await;
    assert!(!c2.is_empty());
    assert!(r3.is_none());
}

#[tokio::test]
async fn producing_then_persistent_consume_then_producing_again_on_same_channel_should_return_cont_for_first_and_second_produce(
) {
    let rspace = create_rspace().await;
    let key = vec!["ch1".to_string()];

    let r1 = rspace
        .produce(key[0].clone(), "datum1".to_string(), false)
        .await;
    let d1 = rspace.store.get_data(&key[0]).await;
    assert_eq!(d1, vec![Datum::create(key[0].clone(), "datum1".to_string(), false)]);
    let c1 = rspace.store.get_continuations(key.clone()).await;
    assert!(c1.is_empty());
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2), vec![vec!["datum1".to_string()]]));

    let r3 = rspace
        .consume(
            key.clone(),
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    assert!(r3.is_none());

    let d2 = rspace.store.get_data(&key[0]).await;
    assert!(d2.is_empty());
    let c2 = rspace.store.get_continuations(key.clone()).await;
    assert!(!c2.is_empty());

    let r4 = rspace
        .produce(key[0].clone(), "datum2".to_string(), false)
        .await;
    assert!(r4.is_some());
    let d3 = rspace.store.get_data(&key[0]).await;
    assert!(d3.is_empty());
    let c3 = rspace.store.get_continuations(key).await;
    assert!(!c3.is_empty());
    assert!(check_same_elements(run_k(r4), vec![vec!["datum2".to_string()]]))
}

// NOTE: This test is unique because it manipulates the continuation
//       from the test case in the store on the Scala side, I think.
//       This is doable because of the way they setup their 'StringsCaptor' instance
#[tokio::test]
async fn doing_persistent_consume_and_producing_multiple_times_should_work() {
    let rspace = create_rspace().await;

    let r1 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(d1.is_empty());
    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(!c1.is_empty());
    assert!(r1.is_none());

    let r2 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let d2 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(d2.is_empty());
    let c2 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(!c2.is_empty());
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2.clone()), vec![vec!["datum1".to_string()]]));

    let r3 = rspace
        .produce("ch1".to_string(), "datum2".to_string(), false)
        .await;
    let d3 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(d3.is_empty());
    let c3 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(!c3.is_empty());
    assert!(r3.is_some());
    assert!(check_same_elements(
        run_k(r3.clone()),
        vec![vec!["datum1".to_string()], vec!["datum2".to_string()]]
    ));

    assert!(check_same_elements(run_k(r3), vec![vec!["datum2".to_string()]]));
}

#[tokio::test]
async fn consuming_and_doing_persistent_produce_should_work() {
    let rspace = create_rspace().await;

    let r1 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    assert!(r1.is_none());

    let r2 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), true)
        .await;
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2), vec![vec!["datum1".to_string()]]));

    let r3 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), true)
        .await;
    assert!(r3.is_none());
    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d1, vec![Datum::create("ch1".to_string(), "datum1".to_string(), true)]);
    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c1.is_empty());
}

#[tokio::test]
async fn consuming_then_persistent_produce_then_consuming_should_work() {
    let rspace = create_rspace().await;

    let r1 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    assert!(r1.is_none());

    let r2 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), true)
        .await;
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2), vec![vec!["datum1".to_string()]]));

    let r3 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), true)
        .await;
    assert!(r3.is_none());
    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d1, vec![Datum::create("ch1".to_string(), "datum1".to_string(), true)]);
    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c1.is_empty());

    let r4 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    assert!(r4.is_some());
    let d2 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d2, vec![Datum::create("ch1".to_string(), "datum1".to_string(), true)]);
    let c2 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c2.is_empty());
    assert!(check_same_elements(run_k(r4), vec![vec!["datum1".to_string()]]))
}

#[tokio::test]
async fn doing_persistent_produce_and_consuming_twice_should_work() {
    let rspace = create_rspace().await;

    let r1 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), true)
        .await;
    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d1, vec![Datum::create("ch1".to_string(), "datum1".to_string(), true)]);
    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c1.is_empty());
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let d2 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d2, vec![Datum::create("ch1".to_string(), "datum1".to_string(), true)]);
    let c2 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c2.is_empty());
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2), vec![vec!["datum1".to_string()]]));

    let r3 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    let d3 = rspace.store.get_data(&"ch1".to_string()).await;
    assert_eq!(d3, vec![Datum::create("ch1".to_string(), "datum1".to_string(), true)]);
    let c3 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c3.is_empty());
    assert!(r3.is_some());
    assert!(check_same_elements(run_k(r3), vec![vec!["datum1".to_string()]]));
}

#[tokio::test]
async fn producing_three_times_then_doing_persistent_consume_should_work() {
    let rspace = create_rspace().await;
    let expected_data = vec![
        Datum::create("ch1".to_string(), "datum1".to_string(), false),
        Datum::create("ch1".to_string(), "datum2".to_string(), false),
        Datum::create("ch1".to_string(), "datum3".to_string(), false),
    ];
    let expected_conts =
        vec![vec!["datum1".to_string()], vec!["datum2".to_string()], vec!["datum3".to_string()]];

    let r1 = rspace
        .produce("ch1".to_string(), "datum1".to_string(), false)
        .await;
    let r2 = rspace
        .produce("ch1".to_string(), "datum2".to_string(), false)
        .await;
    let r3 = rspace
        .produce("ch1".to_string(), "datum3".to_string(), false)
        .await;
    assert!(r1.is_none());
    assert!(r2.is_none());
    assert!(r3.is_none());

    let r4 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    let d1 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(expected_data.iter().any(|datum| d1.contains(datum)));
    let c1 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c1.is_empty());
    assert!(r4.is_some());
    let cont_results_r4 = run_k(r4);
    assert!(expected_conts
        .iter()
        .any(|cont| cont_results_r4.contains(cont)));

    let r5 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    let d2 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(expected_data.iter().any(|datum| d2.contains(datum)));
    let c2 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(c2.is_empty());
    assert!(r5.is_some());
    let cont_results_r5 = run_k(r5);
    assert!(expected_conts
        .iter()
        .any(|cont| cont_results_r5.contains(cont)));

    let r6 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    assert!(r6.is_some());
    let cont_results_r6 = run_k(r6);
    assert!(expected_conts
        .iter()
        .any(|cont| cont_results_r6.contains(cont)));

    let r7 = rspace
        .consume(
            vec!["ch1".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            true,
            BTreeSet::default(),
        )
        .await;
    let d3 = rspace.store.get_data(&"ch1".to_string()).await;
    assert!(d3.is_empty());
    let c3 = rspace
        .store
        .get_continuations(vec!["ch1".to_string()])
        .await;
    assert!(!c3.is_empty());
    assert!(r7.is_none());
}

#[tokio::test]
async fn persistent_produce_should_be_available_for_multiple_matches() {
    let rspace = create_rspace().await;
    let channel = "chan".to_string();

    let r1 = rspace
        .produce(channel.clone(), "datum".to_string(), true)
        .await;
    assert!(r1.is_none());

    let r2 = rspace
        .consume(
            vec![channel.clone(), channel.clone()],
            vec![Pattern::Wildcard, Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    assert!(r2.is_some());
    assert!(check_same_elements(run_k(r2), vec![vec!["datum".to_string(), "datum".to_string()]]));
}

#[tokio::test]
#[should_panic(expected = "RUST ERROR: channels.length must equal patterns.length")]
async fn consuming_with_different_pattern_and_channel_lengths_should_error() {
    let rspace = create_rspace().await;
    let r1 = rspace
        .consume(
            vec!["ch1".to_string(), "ch2".to_string()],
            vec![Pattern::Wildcard],
            StringsCaptor::new(),
            false,
            BTreeSet::default(),
        )
        .await;
    assert!(r1.is_none());
}