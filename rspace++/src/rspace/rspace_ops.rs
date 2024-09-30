// See rspace/src/main/scala/coop/rchain/rspace/RSpaceOps.scala

use std::fmt::Debug;
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::{Arc, Mutex},
};

use dashmap::DashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;

use super::r#match::Match;
// use super::replay_rspace_interface::IReplayRSpace;
use super::rspace_interface::ISpace;
use super::{
    checkpoint::SoftCheckpoint,
    errors::RSpaceError,
    hashing::blake2b256_hash::Blake2b256Hash,
    history::{
        history_reader::HistoryReader, history_repository::HistoryRepository,
        instances::radix_history::RadixHistory,
    },
    hot_store::{HotStore, HotStoreInstances},
    internal::{ConsumeCandidate, Datum, Install, ProduceCandidate, Row, WaitingContinuation},
    rspace_interface::{ContResult, RSpaceResult},
    space_matcher::SpaceMatcher,
    trace::{
        event::{Consume, Produce},
        Log,
    },
};

// pub struct RSpaceOps<C, P, A, K> {
//     pub history_repository: Arc<Box<dyn HistoryRepository<C, P, A, K>>>,
//     pub store: Box<dyn HotStore<C, P, A, K>>,
//     pub installs: Mutex<HashMap<Vec<C>, Install<P, K>>>,
//     pub event_log: Log,
//     pub produce_counter: BTreeMap<Produce, i32>,
//     pub matcher: Arc<Box<dyn Match<P, A>>>,
// }

// impl<C, P, A, K> SpaceMatcher<C, P, A, K> for RSpaceOps<C, P, A, K>
// where
//     C: Clone + Debug + Default + Serialize + std::hash::Hash + Ord + Eq + 'static + Sync + Send,
//     P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
// {
// }



// impl<C, P, A, K> ISpace<C, P, A, K> for RSpaceOps<C, P, A, K>
// where
//     C: Clone + Debug + Default + Serialize + std::hash::Hash + Ord + Eq + 'static + Sync + Send,
//     P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
// {
//     fn get_data(&self, channel: C) -> Vec<Datum<A>> {
//         self.store.get_data(&channel)
//     }

//     fn get_waiting_continuations(&self, channels: Vec<C>) -> Vec<WaitingContinuation<P, K>> {
//         self.store.get_continuations(channels)
//     }

//     fn get_joins(&self, channel: C) -> Vec<Vec<C>> {
//         self.store.get_joins(channel)
//     }

//     fn clear(&mut self) -> Result<(), RSpaceError> {
//         self.reset(RadixHistory::empty_root_node_hash())
//     }

//     fn reset(&mut self, root: Blake2b256Hash) -> Result<(), RSpaceError> {
//         // println!("\nhit rspace++ reset");
//         let next_history = self.history_repository.reset(&root)?;
//         self.history_repository = Arc::new(next_history);

//         self.event_log = Vec::new();
//         self.produce_counter = BTreeMap::new();

//         let history_reader = self.history_repository.get_history_reader(root)?;
//         self.create_new_hot_store(history_reader);
//         self.restore_installs();

//         Ok(())
//     }

//     fn to_map(&self) -> HashMap<Vec<C>, Row<P, A, K>> {
//         self.store.to_map()
//     }

//     fn create_soft_checkpoint(&mut self) -> SoftCheckpoint<C, P, A, K> {
//         // println!("\nhit rspace++ create_soft_checkpoint");
//         // println!("current hot_store state: {:?}", self.store.snapshot());

//         let cache_snapshot = self.store.snapshot();
//         let curr_event_log = self.event_log.clone();
//         let curr_produce_counter = self.produce_counter.clone();

//         self.event_log = Vec::new();
//         self.produce_counter = BTreeMap::new();

//         SoftCheckpoint {
//             cache_snapshot,
//             log: curr_event_log,
//             produce_counter: curr_produce_counter,
//         }
//     }

//     fn revert_to_soft_checkpoint(
//         &mut self,
//         checkpoint: SoftCheckpoint<C, P, A, K>,
//     ) -> Result<(), RSpaceError> {
//         let history = &self.history_repository;
//         let history_reader = history.get_history_reader(history.root())?;
//         let hot_store = HotStoreInstances::create_from_mhs_and_hr(
//             Arc::new(Mutex::new(checkpoint.cache_snapshot)),
//             history_reader.base(),
//         );

//         self.store = hot_store;
//         self.event_log = checkpoint.log;
//         self.produce_counter = checkpoint.produce_counter;

//         Ok(())
//     }
// }

// impl<C, P, A, K> IReplayRSpace<C, P, A, K> for RSpaceOps<C, P, A, K>
// where
//     C: Clone + Debug + Default + Serialize + std::hash::Hash + Ord + Eq + 'static + Sync + Send,
//     P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
// {
// }

// impl<C, P, A, K> RSpaceOps<C, P, A, K>
// where
//     C: Clone + Debug + Default + Serialize + std::hash::Hash + Ord + Eq + 'static + Sync + Send,
//     P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
//     K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
// {
//     pub fn shuffle_with_index<D>(&self, t: Vec<D>) -> Vec<(D, i32)> {
//         let mut rng = thread_rng();
//         let mut indexed_vec = t
//             .into_iter()
//             .enumerate()
//             .map(|(i, d)| (d, i as i32))
//             .collect::<Vec<_>>();
//         indexed_vec.shuffle(&mut rng);
//         indexed_vec
//     }

//     pub fn produce_counters(&self, produce_refs: Vec<Produce>) -> BTreeMap<Produce, i32> {
//         produce_refs
//             .into_iter()
//             .map(|p| (p.clone(), self.produce_counter.get(&p).unwrap_or(&0).clone()))
//             .collect()
//     }

//     pub fn store_waiting_continuation(
//         &self,
//         channels: Vec<C>,
//         wc: WaitingContinuation<P, K>,
//     ) -> MaybeActionResult<C, P, A, K> {
//         // println!("\nHit store_waiting_continuation");
//         self.store.put_continuation(channels.clone(), wc);
//         for channel in channels.iter() {
//             self.store.put_join(channel.clone(), channels.clone());
//             // println!("consume: no data found, storing <(patterns, continuation): ({:?}, {:?})> at <channels: {:?}>", wc.patterns, wc.continuation, channels)
//         }
//         None
//     }

//     pub fn store_data(
//         &self,
//         channel: C,
//         data: A,
//         persist: bool,
//         produce_ref: Produce,
//     ) -> MaybeActionResult<C, P, A, K> {
//         // println!("\nHit store_data");
//         // println!("\nHit store_data, data: {:?}", data);
//         self.store.put_datum(
//             channel,
//             Datum {
//                 a: data,
//                 persist,
//                 source: produce_ref,
//             },
//         );
//         // println!(
//         //     "produce: persisted <data: {:?}> at <channel: {:?}>",
//         //     data, channel
//         // );

//         None
//     }

//     pub fn store_persistent_data(
//         &self,
//         mut data_candidates: Vec<ConsumeCandidate<C, A>>,
//         _peeks: &BTreeSet<i32>,
//     ) -> Option<Vec<()>> {
//         data_candidates.sort_by(|a, b| b.datum_index.cmp(&a.datum_index));
//         let results: Vec<_> = data_candidates
//             .into_iter()
//             .rev()
//             .map(|consume_candidate| {
//                 let ConsumeCandidate {
//                     channel,
//                     datum: Datum { persist, .. },
//                     removed_datum: _,
//                     datum_index,
//                 } = consume_candidate;

//                 if !persist {
//                     self.store.remove_datum(channel, datum_index)
//                 } else {
//                     Some(())
//                 }
//             })
//             .collect();

//         if results.iter().any(|res| res.is_none()) {
//             None
//         } else {
//             Some(results.into_iter().filter_map(|x| x).collect())
//         }
//     }

//     pub fn restore_installs(&mut self) -> () {
//         let installs = self.installs.lock().unwrap().clone();
//         // println!("\ninstalls: {:?}", installs);
//         for (channels, install) in installs {
//             self.install(channels, install.patterns, install.continuation);
//         }
//     }

//     pub fn install(
//         &mut self,
//         channels: Vec<C>,
//         patterns: Vec<P>,
//         continuation: K,
//     ) -> Option<(K, Vec<A>)> {
//         self.locked_install(channels, patterns, continuation)
//     }

//     /*
//      * Here, we create a cache of the data at each channel as `channelToIndexedData`
//      * which is used for finding matches.  When a speculative match is found, we can
//      * remove the matching datum from the remaining data candidates in the cache.
//      *
//      * Put another way, this allows us to speculatively remove matching data without
//      * affecting the actual store contents.
//      */
//     pub fn fetch_channel_to_index_data(
//         &self,
//         channels: &Vec<C>,
//     ) -> DashMap<C, Vec<(Datum<A>, i32)>> {
//         let map = DashMap::new();
//         for c in channels {
//             let data = self.store.get_data(c);
//             let shuffled_data = self.shuffle_with_index(data);
//             map.insert(c.clone(), shuffled_data);
//         }
//         map
//     }

//     fn locked_install(
//         &mut self,
//         channels: Vec<C>,
//         patterns: Vec<P>,
//         continuation: K,
//     ) -> Option<(K, Vec<A>)> {
//         // println!("\nhit locked_install");
//         // println!("channels: {:?}", channels);
//         // println!("patterns: {:?}", patterns);
//         // println!("continuation: {:?}", continuation);
//         if channels.len() != patterns.len() {
//             panic!("RUST ERROR: channels.length must equal patterns.length");
//         } else {
//             // println!(
//             //     "install: searching for data matching <patterns: {:?}> at <channels: {:?}>",
//             //     patterns, channels
//             // );

//             let consume_ref =
//                 Consume::create(channels.clone(), patterns.clone(), continuation.clone(), true);
//             let channel_to_indexed_data = self.fetch_channel_to_index_data(&channels);
//             // println!("channel_to_indexed_data in locked_install: {:?}", channel_to_indexed_data);
//             let zipped: Vec<(C, P)> = channels
//                 .iter()
//                 .cloned()
//                 .zip(patterns.iter().cloned())
//                 .collect();
//             let options: Option<Vec<ConsumeCandidate<C, A>>> = self
//                 .extract_data_candidates(&self.matcher, zipped, channel_to_indexed_data, Vec::new())
//                 .into_iter()
//                 .collect();

//             // println!("options in locked_install: {:?}", options);

//             let result = match options {
//                 None => {
//                     self.installs.lock().unwrap().insert(
//                         channels.clone(),
//                         Install {
//                             patterns: patterns.clone(),
//                             continuation: continuation.clone(),
//                         },
//                     );

//                     self.store.install_continuation(
//                         channels.clone(),
//                         WaitingContinuation {
//                             patterns,
//                             continuation,
//                             persist: true,
//                             peeks: BTreeSet::default(),
//                             source: consume_ref,
//                         },
//                     );

//                     for channel in channels.iter() {
//                         self.store.install_join(channel.clone(), channels.clone());
//                     }
//                     // println!(
//                     //     "storing <(patterns, continuation): ({:?}, {:?})> at <channels: {:?}>",
//                     //     patterns, continuation, channels
//                     // );
//                     // println!("store length after install: {:?}\n", self.store.to_map().len());
//                     None
//                 }
//                 Some(_) => {
//                     panic!("RUST ERROR: Installing can be done only on startup")
//                 }
//             };
//             result
//         }
//     }

//     pub fn create_new_hot_store(
//         &mut self,
//         history_reader: Box<dyn HistoryReader<Blake2b256Hash, C, P, A, K>>,
//     ) -> () {
//         let next_hot_store = HotStoreInstances::create_from_hr(history_reader.base());
//         self.store = next_hot_store;
//     }

//     pub fn wrap_result(
//         &self,
//         channels: Vec<C>,
//         wk: WaitingContinuation<P, K>,
//         _consume_ref: Consume,
//         data_candidates: Vec<ConsumeCandidate<C, A>>,
//     ) -> MaybeActionResult<C, P, A, K> {
//         // println!("\nhit wrap_result");

//         let cont_result = ContResult {
//             continuation: wk.continuation,
//             persistent: wk.persist,
//             channels,
//             patterns: wk.patterns,
//             peek: !wk.peeks.is_empty(),
//         };

//         let rspace_results = data_candidates
//             .into_iter()
//             .map(|data_candidate| RSpaceResult {
//                 channel: data_candidate.channel,
//                 matched_datum: data_candidate.datum.a,
//                 removed_datum: data_candidate.removed_datum,
//                 persistent: data_candidate.datum.persist,
//             })
//             .collect();

//         Some((cont_result, rspace_results))
//     }

//     pub fn remove_matched_datum_and_join(
//         &self,
//         channels: Vec<C>,
//         mut data_candidates: Vec<ConsumeCandidate<C, A>>,
//     ) -> Option<Vec<()>> {
//         data_candidates.sort_by(|a, b| b.datum_index.cmp(&a.datum_index));
//         let results: Vec<_> = data_candidates
//             .into_iter()
//             .rev()
//             .map(|consume_candidate| {
//                 let ConsumeCandidate {
//                     channel,
//                     datum: Datum { persist, .. },
//                     removed_datum: _,
//                     datum_index,
//                 } = consume_candidate;

//                 let channels_clone = channels.clone();
//                 if datum_index >= 0 && !persist {
//                     self.store.remove_datum(channel.clone(), datum_index);
//                 }
//                 self.store.remove_join(channel, channels_clone);

//                 Some(())
//             })
//             .collect();

//         if results.iter().any(|res| res.is_none()) {
//             None
//         } else {
//             Some(results.into_iter().filter_map(|x| x).collect())
//         }
//     }

//     pub fn run_matcher_for_channels(
//         &self,
//         grouped_channels: Vec<Vec<C>>,
//         fetch_matching_continuations: impl Fn(Vec<C>) -> Vec<(WaitingContinuation<P, K>, i32)>,
//         fetch_matching_data: impl Fn(C) -> (C, Vec<(Datum<A>, i32)>),
//     ) -> MaybeProduceCandidate<C, P, A, K> {
//         let mut remaining = grouped_channels;

//         loop {
//             match remaining.split_first() {
//                 Some((channels, rest)) => {
//                     let match_candidates = fetch_matching_continuations(channels.to_vec());
//                     // println!("match_candidates: {:?}", match_candidates);
//                     let fetch_data: Vec<_> = channels
//                         .iter()
//                         .map(|c| fetch_matching_data(c.clone()))
//                         .collect();

//                     let channel_to_indexed_data_list: Vec<(C, Vec<(Datum<A>, i32)>)> =
//                         fetch_data.into_iter().filter_map(|x| Some(x)).collect();
//                     // println!("channel_to_indexed_data_list: {:?}", channel_to_indexed_data_list);

//                     let first_match = self.extract_first_match(
//                         &self.matcher,
//                         channels.to_vec(),
//                         match_candidates,
//                         channel_to_indexed_data_list.into_iter().collect(),
//                     );

//                     // println!("first_match in run_matcher_for_channels: {:?}", first_match);

//                     match first_match {
//                         Some(produce_candidate) => return Some(produce_candidate),
//                         None => remaining = rest.to_vec(),
//                     }
//                 }
//                 None => {
//                     // println!("returning none in in run_matcher_for_channels");
//                     return None;
//                 }
//             }
//         }
//     }
// }
