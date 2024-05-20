use std::{
    collections::LinkedList,
    sync::{Arc, Mutex},
};

use dashmap::DashMap;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest_derive::Arbitrary;
use rspace_plus_plus::rspace::{
    history::history_reader::HistoryReaderBase,
    hot_store::{HotStore, HotStoreInstances, HotStoreState},
    internal::{Datum, WaitingContinuation},
    matcher::r#match::Match,
};
use rstest::*;
use std::fmt::Debug;
use std::hash::Hash;

// See rspace/src/test/scala/coop/rchain/rspace/HotStoreSpec.scala

type Channel = String;
type Data = Datum<String>;
type Continuation = WaitingContinuation<Pattern, StringsCaptor>;
type Join = Vec<Channel>;
type Joins = Vec<Join>;

const SIZE_RANGE: usize = 2; // 10

proptest! {
  #![proptest_config(ProptestConfig {
    cases: 1, // 20
    failure_persistence: None,
    .. ProptestConfig::default()
})]

  // Double check these tests perform same logic as Scala tests for Joins
  // For example, 'arbitraryJoins' and double or nested vectors
  // What is difference between 'vec(any::<T>())' and 'any::<Vec<T>>()' in proptest?

  #[test]
  fn get_continuations_when_cache_is_empty_should_read_from_history_and_put_into_cache(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE), history_continuations
    in vec(any::<Continuation>(), 0..=SIZE_RANGE)) {
      let (state, history, hot_store) = fixture();

      history.put_continuations(channels.clone(), history_continuations.clone());

      let cache = state.lock().unwrap();
      assert!(cache.continuations.is_empty());
      drop(cache);

      let read_continuations = hot_store.get_continuations(channels.clone());
      let cache = state.lock().unwrap();
      assert_eq!(cache.continuations.get(&channels).unwrap().clone(), history_continuations);
      assert_eq!(read_continuations, history_continuations);
  }

  #[test]
  fn get_continuations_when_cache_contains_data_should_read_from_cache_ignoring_history(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE), history_continuations
    in vec(any::<Continuation>(), 0..=SIZE_RANGE), cached_continuations in vec(any::<Continuation>(), 0..=SIZE_RANGE)) {
      let (state, history, hot_store) = fixture();

      history.put_continuations(channels.clone(), history_continuations.clone());
      let mut state_lock = state.lock().unwrap();
      *state_lock = HotStoreState { continuations: DashMap::from_iter(vec![(channels.clone(), cached_continuations.clone())]), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(state_lock);

      let read_continuations = hot_store.get_continuations(channels.clone());
      let cache = state.lock().unwrap();
      assert_eq!(cache.continuations.get(&channels).unwrap().clone(), cached_continuations);
      assert_eq!(read_continuations, cached_continuations);
  }

  #[test]
  fn get_continuations_should_include_installed_continuations(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE), mut cached_continuations
    in vec(any::<Continuation>(), 0..=SIZE_RANGE), installed_continuation in any::<Continuation>()) {
      let (state, _, hot_store) = fixture();

      let mut state_lock = state.lock().unwrap();
      *state_lock = HotStoreState { continuations: DashMap::from_iter(vec![(channels.clone(), cached_continuations.clone())]), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(state_lock);

      hot_store.install_continuation(channels.clone(), installed_continuation.clone());
      let res = hot_store.get_continuations(channels);
      cached_continuations.insert(0, installed_continuation);
      assert_eq!(res, cached_continuations);
  }

  #[test]
  fn put_continuation_when_cache_is_empty_should_read_from_history_and_add_to_it(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE), mut history_continuations
    in vec(any::<Continuation>(), 0..=SIZE_RANGE), inserted_continuation in any::<Continuation>()) {
      let (state, history, hot_store) = fixture();

      history.put_continuations(channels.clone(), history_continuations.clone());
      hot_store.put_continuation(channels.clone(), inserted_continuation.clone());

      let cache = state.lock().unwrap();
      history_continuations.insert(0, inserted_continuation);
      assert_eq!(cache.continuations.get(&channels).unwrap().clone(), history_continuations);
  }

  #[test]
  fn put_continuation_when_cache_contains_data_should_read_from_cache_and_add_to_it(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE), history_continuations
    in vec(any::<Continuation>(), 0..=SIZE_RANGE), mut cached_continuations in vec(any::<Continuation>(), 0..=SIZE_RANGE), inserted_continuation in any::<Continuation>()) {
      let (state, history, hot_store) = fixture();

      history.put_continuations(channels.clone(), history_continuations.clone());
      let mut state_lock = state.lock().unwrap();
      *state_lock = HotStoreState { continuations: DashMap::from_iter(vec![(channels.clone(), cached_continuations.clone())]), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(state_lock);

      hot_store.put_continuation(channels.clone(),inserted_continuation.clone());

      let cache = state.lock().unwrap();
      cached_continuations.insert(0, inserted_continuation);
      assert_eq!(cache.continuations.get(&channels).unwrap().clone(), cached_continuations);
  }

  #[test]
  fn install_continuation_should_cache_installed_continuations_separately(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE), mut cached_continuations
    in vec(any::<Continuation>(), 0..=SIZE_RANGE), inserted_continuation in any::<Continuation>(), installed_continuation in any::<Continuation>()) {
      prop_assume!(inserted_continuation != installed_continuation);
      let (state, _, hot_store) = fixture();

      let mut state_lock = state.lock().unwrap();
      *state_lock = HotStoreState { continuations: DashMap::from_iter(vec![(channels.clone(), cached_continuations.clone())]), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(state_lock);

      hot_store.install_continuation(channels.clone(), installed_continuation.clone());
      hot_store.put_continuation(channels.clone(), inserted_continuation.clone());

      let cache = state.lock().unwrap();
      cached_continuations.insert(0, inserted_continuation);
      assert_eq!(cache.installed_continuations.get(&channels).unwrap().clone(), installed_continuation);
      assert_eq!(cache.continuations.get(&channels).unwrap().clone(), cached_continuations);
  }

  #[test]
  fn remove_continuation_when_cache_is_empty_should_read_from_history_and_remove_the_continuation_from_loaded_data(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE),
    history_continuations in vec(any::<Continuation>(), 0..=SIZE_RANGE), index in any::<i32>()) {
      let (state, history, hot_store) = fixture();

      history.put_continuations(channels.clone(), history_continuations.clone());
      let res = hot_store.remove_continuation(channels.clone(), index);

      let state_lock = state.lock().unwrap();
      assert!(check_removal_works_or_fails_on_error(res, state_lock.continuations.get(&channels).map_or(Vec::new(), |x| x.clone()), history_continuations, index).is_ok());
  }

  #[test]
  fn remove_continuation_when_cache_contains_data_should_read_from_cache_and_remove_continuation_from_loaded_data(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE),
    history_continuations in vec(any::<Continuation>(), 0..=SIZE_RANGE), cached_continuations in vec(any::<Continuation>(), 0..=SIZE_RANGE), index in any::<i32>()) {
      let (state, history, hot_store) = fixture();

      history.put_continuations(channels.clone(), history_continuations.clone());
      let mut state_lock = state.lock().unwrap();
      *state_lock = HotStoreState { continuations: DashMap::from_iter(vec![(channels.clone(), cached_continuations.clone())]), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(state_lock);

      let res = hot_store.remove_continuation(channels.clone(), index);
      let state_lock = state.lock().unwrap();
      assert!(check_removal_works_or_fails_on_error(res, state_lock.continuations.get(&channels).map_or(Vec::new(), |x| x.clone()), cached_continuations, index).is_ok());
  }

  #[test]
  fn remove_continuation_when_installed_continuation_is_present_should_not_allow_its_removal(channels in  vec(any::<Channel>(), 0..=SIZE_RANGE),
    mut cached_continuations in vec(any::<Continuation>(), 0..=SIZE_RANGE), installed_continuation in any::<Continuation>(), index in any::<i32>()) {
      let (state, _, hot_store) = fixture();

      let mut state_lock = state.lock().unwrap();
      *state_lock = HotStoreState { continuations: DashMap::from_iter(vec![(channels.clone(), cached_continuations.clone())]), installed_continuations:  DashMap::from_iter(vec![(channels.clone(), installed_continuation.clone())]),
        data: DashMap::new(), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(state_lock);

      let res = hot_store.remove_continuation(channels.clone(), index);
      if index == 0 {
        assert!(res.is_none());
      } else {
        // index of the removed continuation includes the installed
        let conts = hot_store.get_continuations(channels);
        cached_continuations.insert(0, installed_continuation);
        assert!(check_removal_works_or_fails_on_error(res, conts, cached_continuations, index).is_ok());
      }
  }

  #[test]
  fn get_data_when_cache_is_empty_should_read_from_history_and_put_into_the_cache(channel in  any::<Channel>(), history_data in vec(any::<Datum<String>>(), 0..=SIZE_RANGE)) {
      let (state, history, hot_store) = fixture();

      history.put_data(channel.clone(), history_data.clone());
      let cache = state.lock().unwrap();
      assert!(cache.data.is_empty());
      drop(cache);

      let read_data = hot_store.get_data(&channel);
      let cache = state.lock().unwrap();
      assert_eq!(cache.data.get(&channel).unwrap().clone(), history_data);
      assert_eq!(read_data, history_data);
  }

  #[test]
  fn get_data_when_cache_contains_data_should_read_from_cache_ignoring_history(channel in  any::<Channel>(), history_data in vec(any::<Data>(), 0..=SIZE_RANGE),
    cached_data in vec(any::<Data>(), 0..=SIZE_RANGE)) {
      let (state, history, hot_store) = fixture();

      history.put_data(channel.clone(), history_data.clone());
      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::from_iter(vec![(channel.clone(), cached_data.clone())]), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(cache);

      let read_data = hot_store.get_data(&channel);
      let cache = state.lock().unwrap();
      assert_eq!(cache.data.get(&channel).unwrap().clone(), cached_data);
      assert_eq!(read_data, cached_data);
  }

  #[test]
  fn put_datum_when_cache_is_empty_should_read_from_history_and_add_to_it(channel in  any::<Channel>(), mut history_data in vec(any::<Data>(), 0..=SIZE_RANGE),
    inserted_data in any::<Data>()) {
      let (state, history, hot_store) = fixture();

      history.put_data(channel.clone(), history_data.clone());
      hot_store.put_datum(channel.clone(), inserted_data.clone());

      let cache = state.lock().unwrap();
      history_data.insert(0, inserted_data);
      assert_eq!(cache.data.get(&channel).unwrap().clone(), history_data);
  }

  #[test]
  fn put_datum_when_contains_data_should_read_from_cache_and_add_to_it(channel in  any::<Channel>(), history_data in vec(any::<Data>(), 0..=SIZE_RANGE),
    mut cached_data in vec(any::<Data>(), 0..=SIZE_RANGE), inserted_data in any::<Data>()) {
      let (state, history, hot_store) = fixture();

      history.put_data(channel.clone(), history_data.clone());
      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::from_iter(vec![(channel.clone(), cached_data.clone())]), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(cache);

      hot_store.put_datum(channel.clone(), inserted_data.clone());
      let cache = state.lock().unwrap();
      cached_data.insert(0, inserted_data);
      assert_eq!(cache.data.get(&channel).unwrap().clone(), cached_data);
  }

  #[test]
  fn remove_datum_when_cache_is_empty_should_read_from_history_and_remove_datum_at_index(channel in  any::<Channel>(), history_data in vec(any::<Data>(), 0..=SIZE_RANGE),
    index in any::<i32>()) {
      let (state, history, hot_store) = fixture();

      history.put_data(channel.clone(), history_data.clone());
      let res = hot_store.remove_datum(channel.clone(), index);

      let cache = state.lock().unwrap();
      assert!(check_removal_works_or_fails_on_error(res, cache.data.get(&channel).map_or(Vec::new(), |x| x.clone()), history_data, index).is_ok());
  }

  #[test]
  fn remove_datum_when_cache_contains_data_should_read_from_cache_and_remove_datum(channel in  any::<Channel>(), history_data in vec(any::<Data>(), 0..=SIZE_RANGE),
    cached_data in vec(any::<Data>(), 0..=SIZE_RANGE), index in any::<i32>()) {
      let (state, history, hot_store) = fixture();

      history.put_data(channel.clone(), history_data.clone());
      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::from_iter(vec![(channel.clone(), cached_data.clone())]), joins: DashMap::new(), installed_joins: DashMap::new() };
      drop(cache);

      let res = hot_store.remove_datum(channel.clone(), index);
      let cache = state.lock().unwrap();
      assert!(check_removal_works_or_fails_on_error(res, cache.data.get(&channel).unwrap().clone(), cached_data, index).is_ok());
  }

  #[test]
  fn get_joins_when_cache_is_empty_should_read_from_history_and_put_into_the_cache(channel in  any::<Channel>(), history_joins in vec(any::<Vec<Channel>>(), 0..=SIZE_RANGE)) {
      let (state, history, hot_store) = fixture();

      history.put_joins(channel.clone(), history_joins.clone());
      let cache = state.lock().unwrap();
      assert!(cache.joins.is_empty());
      drop(cache);

      let read_joins = hot_store.get_joins(channel.clone());
      let cache = state.lock().unwrap();
      assert_eq!(cache.joins.get(&channel).unwrap().clone(), history_joins);
      assert_eq!(read_joins, history_joins);
  }

  #[test]
  fn get_joins_when_cache_contains_data_should_read_from_cache_ignoring_history(channel in  any::<Channel>(), history_joins in vec(any::<Vec<Channel>>(), 0..=SIZE_RANGE),
    cached_joins in vec(any::<Vec<Channel>>(), 0..=SIZE_RANGE)) {
      let (state, history, hot_store) = fixture();

      history.put_joins(channel.clone(), history_joins.clone());
      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::from_iter(vec![(channel.clone(), cached_joins.clone())]), installed_joins: DashMap::new() };
      drop(cache);

      let read_joins = hot_store.get_joins(channel.clone());
      let cache = state.lock().unwrap();
      assert_eq!(cache.joins.get(&channel).unwrap().clone(), cached_joins);
      assert_eq!(read_joins, cached_joins);
  }

  #[test]
  fn put_join_when_cache_is_empty_should_read_from_history_and_add_to_it(channel in  any::<Channel>(), mut history_joins in any::<Joins>(), inserted_join in any::<Join>()) {
      prop_assume!(!history_joins.contains(&inserted_join));
      let (state, history, hot_store) = fixture();

      history.put_joins(channel.clone(), history_joins.clone());
      hot_store.put_join(channel.clone(), inserted_join.clone());

      let cache = state.lock().unwrap();
      history_joins.insert(0, inserted_join);
      assert_eq!(cache.joins.get(&channel).unwrap().clone(), history_joins);
  }

  #[test]
  fn put_join_when_cache_contains_data_should_read_from_cache_and_add_to_it(channel in  any::<Channel>(), history_joins in any::<Joins>(), mut cached_joins in any::<Joins>(),
    inserted_join in any::<Join>()) {
      prop_assume!(!history_joins.contains(&inserted_join));
      let (state, history, hot_store) = fixture();

      history.put_joins(channel.clone(), history_joins.clone());
      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::from_iter(vec![(channel.clone(), cached_joins.clone())]), installed_joins: DashMap::new() };
      drop(cache);

      hot_store.put_join(channel.clone(), inserted_join.clone());
      let cache = state.lock().unwrap();
      cached_joins.insert(0, inserted_join);
      assert_eq!(cache.joins.get(&channel).unwrap().clone(), cached_joins);
  }

  #[test]
  fn put_join_should_not_allow_inserting_duplicate_joins(channel in  any::<Channel>(), mut cached_joins in any::<Joins>(), inserted_join in any::<Join>()) {
      let (state, _, hot_store) = fixture();

      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::from_iter(vec![(channel.clone(), cached_joins.clone())]), installed_joins: DashMap::new() };
      drop(cache);

      hot_store.put_join(channel.clone(), inserted_join.clone());
      let cache = state.lock().unwrap();

      if !cached_joins.contains(&inserted_join) {
        cached_joins.insert(0, inserted_join);
        assert_eq!(cache.joins.get(&channel).unwrap().clone(), cached_joins);
      } else {
        assert_eq!(cache.joins.get(&channel).unwrap().clone(), cached_joins);
      }
  }

  #[test]
  fn install_join_should_cache_installed_joins_separately(channel in  any::<Channel>(), mut cached_joins in any::<Joins>(), inserted_join in any::<Join>(),
    installed_join in any::<Join>()) {
      let (state, _, hot_store) = fixture();

      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::from_iter(vec![(channel.clone(), cached_joins.clone())]), installed_joins: DashMap::new() };
      drop(cache);

      hot_store.put_join(channel.clone(), inserted_join.clone());
      hot_store.install_join(channel.clone(), installed_join.clone());

      let cache = state.lock().unwrap();
      assert_eq!(cache.installed_joins.get(&channel).unwrap().clone(), vec![installed_join]);
      cached_joins.insert(0, inserted_join);
      assert_eq!(cache.joins.get(&channel).unwrap().clone(), cached_joins);
  }

  #[test]
  fn install_join_should_not_allow_installing_duplicate_joins_per_channel(channel in  any::<Channel>(), cached_joins in any::<Joins>(), installed_join in any::<Join>()) {
      let (state, _, hot_store) = fixture();

      let mut cache = state.lock().unwrap();
      *cache = HotStoreState { continuations: DashMap::new(), installed_continuations: DashMap::new(), data: DashMap::new(), joins: DashMap::from_iter(vec![(channel.clone(), cached_joins.clone())]), installed_joins: DashMap::new() };
      drop(cache);

      hot_store.install_join(channel.clone(), installed_join.clone());
      hot_store.install_join(channel.clone(), installed_join.clone());

      let cache = state.lock().unwrap();
      assert_eq!(cache.installed_joins.get(&channel).unwrap().clone(), vec![installed_join]);
  }
}

fn check_removal_works_or_fails_on_error<T>(
    res: Option<()>,
    actual: Vec<T>,
    initial: Vec<T>,
    index: i32,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: PartialEq + Debug + Clone,
{
    if index < 0 || index >= initial.len().try_into().unwrap() {
        assert!(res.is_none());
        assert_eq!(actual, initial);
    } else {
        assert!(res.is_some());
        let expected: Vec<T> = initial
            .iter()
            .enumerate()
            .filter(|&(i, _)| i as i32 != index)
            .map(|(_, item)| item.clone())
            .collect();
        assert_eq!(actual, expected);
    }
    Ok(())
}

// See rspace/src/main/scala/coop/rchain/rspace/examples/StringExamples.scala
#[derive(Clone, Debug, Default, PartialEq, Arbitrary)]
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

#[derive(Clone, Debug, Default, PartialEq, Arbitrary)]
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

// See rspace/src/test/scala/coop/rchain/rspace/HotStoreSpec.scala
#[derive(Clone)]
struct TestHistory<C: Eq + Hash, P: Clone, A: Clone, K: Clone> {
    state: Arc<Mutex<HotStoreState<C, P, A, K>>>,
}

impl<C: Clone + Eq + Hash + Send, P: Clone + Send, A: Clone + Send, K: Clone + Send>
    HistoryReaderBase<C, P, A, K> for TestHistory<C, P, A, K>
{
    fn get_data(&self, channel: &C) -> Vec<Datum<A>> {
        let state_lock = self.state.lock().unwrap();
        let data = state_lock
            .data
            .get(channel)
            .map(|v| v.to_vec())
            .unwrap_or_else(|| Vec::new());
        data
    }

    fn get_continuations(&self, channels: &Vec<C>) -> Vec<WaitingContinuation<P, K>> {
        let state_lock = self.state.lock().unwrap();
        let continuations = state_lock
            .continuations
            .get(channels)
            .map(|v| v.to_vec())
            .unwrap_or_else(|| Vec::new());
        continuations
    }

    fn get_joins(&self, channel: &C) -> Vec<Vec<C>> {
        let state_lock = self.state.lock().unwrap();
        let joins = state_lock
            .joins
            .get(channel)
            .map(|v| v.to_vec())
            .unwrap_or_else(|| Vec::new());
        joins
    }

    fn get_data_proj(&self, _key: &C) -> Vec<Datum<A>> {
        todo!()
    }

    fn get_continuations_proj(&self, _key: &Vec<C>) -> Vec<WaitingContinuation<P, K>> {
        todo!()
    }

    fn get_joins_proj(&self, _key: &C) -> Vec<Vec<C>> {
        todo!()
    }
}

impl<C: Eq + Hash, P: Clone, A: Clone, K: Clone> TestHistory<C, P, A, K> {
    fn put_data(&self, channel: C, data: Vec<Datum<A>>) -> () {
        let state = self.state.lock().unwrap();
        state.data.insert(channel, data);
    }

    fn put_continuations(
        &self,
        channels: Vec<C>,
        continuations: Vec<WaitingContinuation<P, K>>,
    ) -> () {
        let state = self.state.lock().unwrap();
        state.continuations.insert(channels, continuations);
    }

    fn put_joins(&self, channel: C, joins: Vec<Vec<C>>) -> () {
        let state = self.state.lock().unwrap();
        state.joins.insert(channel, joins);
    }
}

type StateSetup = (
    Arc<Mutex<HotStoreState<String, Pattern, String, StringsCaptor>>>,
    TestHistory<String, Pattern, String, StringsCaptor>,
    Box<dyn HotStore<String, Pattern, String, StringsCaptor>>,
);

#[fixture]
pub fn fixture() -> StateSetup {
    let history_state =
        Arc::new(Mutex::new(HotStoreState::<String, Pattern, String, StringsCaptor>::default()));

    let history = TestHistory {
        state: history_state.clone(),
    };

    let cache =
        Arc::new(Mutex::new(HotStoreState::<String, Pattern, String, StringsCaptor>::default()));

    let hot_store =
        HotStoreInstances::create_from_mhs_and_hr(cache.clone(), Box::new(history.clone()));
    (cache, history, Box::new(hot_store))
}
