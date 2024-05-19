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
use std::hash::Hash;

// See rspace/src/test/scala/coop/rchain/rspace/HotStoreSpec.scala

type Channel = String;
type Data = Datum<String>;
type Continuation = WaitingContinuation<Pattern, StringsCaptor>;
type Testing = (Pattern, StringsCaptor);
type Join = Vec<Channel>;
type Joins = Vec<Join>;

const SIZE_RANGE: usize = 2; // 10

proptest! {
  #![proptest_config(ProptestConfig {
    cases: 1, // 20
    failure_persistence: None,
    .. ProptestConfig::default()
})]

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
