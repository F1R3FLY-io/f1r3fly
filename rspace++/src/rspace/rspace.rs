// See rspace/src/main/scala/coop/rchain/rspace/RSpace.scala

use super::checkpoint::SoftCheckpoint;
use super::errors::HistoryRepositoryError;
use super::errors::RSpaceError;
use super::hashing::blake2b256_hash::Blake2b256Hash;
use super::history::history_reader::HistoryReader;
use super::history::instances::radix_history::RadixHistory;
use super::r#match::Match;
use super::rspace_interface::ContResult;
use super::rspace_interface::RSpaceResult;
use super::rspace_ops::MaybeActionResult;
use super::rspace_ops::MaybeProduceCandidate;
use super::rspace_ops::RSpaceOps;
use super::rspace_ops::CONSUME_COMM_LABEL;
use super::rspace_ops::PRODUCE_COMM_LABEL;
use super::trace::event::Consume;
use super::trace::event::Event;
use super::trace::event::IOEvent;
use super::trace::event::Produce;
use super::trace::event::COMM;
use super::trace::Log;
use super::tuplespace_interface::Tuplespace;
use crate::rspace::checkpoint::Checkpoint;
use crate::rspace::history::history_repository::HistoryRepository;
use crate::rspace::history::history_repository::HistoryRepositoryInstances;
use crate::rspace::hot_store::{HotStore, HotStoreInstances};
use crate::rspace::internal::*;
use crate::rspace::shared::key_value_store::KeyValueStore;
use crate::rspace::space_matcher::SpaceMatcher;
use dashmap::DashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct RSpaceStore {
    pub history: Arc<Mutex<Box<dyn KeyValueStore>>>,
    pub roots: Arc<Mutex<Box<dyn KeyValueStore>>>,
    pub cold: Arc<Mutex<Box<dyn KeyValueStore>>>,
}

#[repr(C)]
pub struct RSpace<C, P, A, K> {
    pub ops: RSpaceOps<C, P, A, K>,
}

impl<C, P, A, K> Tuplespace<C, P, A, K> for RSpace<C, P, A, K>
where
    C: Clone + Debug + Default + Serialize + Hash + Ord + Eq + 'static + Sync + Send,
    P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
{
    fn consume(
        &mut self,
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
        persist: bool,
        peeks: BTreeSet<i32>,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit consume");

        if channels.is_empty() {
            panic!("RUST ERROR: channels can't be empty");
        } else if channels.len() != patterns.len() {
            panic!("RUST ERROR: channels.length must equal patterns.length");
        } else {
            let consume_ref =
                Consume::create(channels.clone(), patterns.clone(), continuation.clone(), persist);

            let result =
                self.locked_consume(channels, patterns, continuation, persist, peeks, consume_ref);
            // println!("\nlocked_consume result: {:?}", result);
            result
        }
    }

    fn produce(&mut self, channel: C, data: A, persist: bool) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit produce");
        // println!("\nto_map: {:?}", self.store.to_map());
        // println!("\nHit produce, data: {:?}", data);
        // println!("\n\nHit produce, channel: {:?}", channel);

        let produce_ref = Produce::create(channel.clone(), data.clone(), persist);
        let result = self.locked_produce(channel, data, persist, produce_ref);
        // println!("\nlocked_produce result: {:?}", result);
        result
    }

    fn create_checkpoint(&mut self) -> Result<Checkpoint, RSpaceError> {
        // println!("\nhit rspace++ create_checkpoint");
        let changes = self.ops.store.changes();
        let next_history = self.ops.history_repository.checkpoint(&changes);
        self.ops.history_repository = Arc::new(next_history);

        let log = self.ops.event_log.clone();
        self.ops.event_log = Vec::new();
        self.ops.produce_counter = BTreeMap::new();

        let history_reader = self
            .ops
            .history_repository
            .get_history_reader(self.ops.history_repository.root())?;

        self.ops.create_new_hot_store(history_reader);
        self.ops.restore_installs();

        Ok(Checkpoint {
            root: self.ops.history_repository.root(),
            log,
        })
    }

    fn install(
        &mut self,
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
    ) -> Option<(K, Vec<A>)> {
        self.ops.install(channels, patterns, continuation)
    }
}

impl<C, P, A, K> RSpace<C, P, A, K>
where
    C: Clone + Debug + Default + Serialize + Hash + Ord + Eq + 'static + Sync + Send,
    P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
{
    /**
     * Creates [[RSpace]] from [[HistoryRepository]] and [[HotStore]].
     */
    pub fn apply(
        history_repository: Arc<Box<dyn HistoryRepository<C, P, A, K>>>,
        store: Box<dyn HotStore<C, P, A, K>>,
        matcher: Arc<Box<dyn Match<P, A>>>,
    ) -> RSpace<C, P, A, K>
    where
        C: Clone + Debug + Ord + Hash,
        P: Clone + Debug,
        A: Clone + Debug,
        K: Clone + Debug,
    {
        RSpace {
            ops: RSpaceOps {
                history_repository,
                store,
                installs: Mutex::new(HashMap::new()),
                event_log: Vec::new(),
                produce_counter: BTreeMap::new(),
                matcher,
            },
        }
    }

    pub fn create(
        store: RSpaceStore,
        matcher: Arc<Box<dyn Match<P, A>>>,
    ) -> Result<RSpace<C, P, A, K>, HistoryRepositoryError>
    where
        C: Clone
            + Debug
            + Default
            + Send
            + Sync
            + Serialize
            + Ord
            + Hash
            + for<'a> Deserialize<'a>
            + 'static,
        P: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
        A: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
        K: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    {
        let setup = Self::create_history_repo(store).unwrap();
        let (history_reader, store) = setup;
        let space = Self::apply(Arc::new(history_reader), store, matcher);
        Ok(space)
    }

    /**
     * Creates [[HistoryRepository]] and [[HotStore]].
     */
    pub fn create_history_repo(
        store: RSpaceStore,
    ) -> Result<
        (Box<dyn HistoryRepository<C, P, A, K>>, Box<dyn HotStore<C, P, A, K>>),
        HistoryRepositoryError,
    >
    where
        C: Clone
            + Debug
            + Default
            + Send
            + Sync
            + Serialize
            + for<'a> Deserialize<'a>
            + Eq
            + Hash
            + 'static,
        P: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
        A: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
        K: Clone + Debug + Default + Send + Sync + Serialize + for<'a> Deserialize<'a> + 'static,
    {
        let history_repo =
            HistoryRepositoryInstances::lmdb_repository(store.history, store.roots, store.cold)?;

        let history_reader = history_repo.get_history_reader(history_repo.root())?;

        let hot_store = HotStoreInstances::create_from_hr(history_reader.base());

        Ok((history_repo, hot_store))
    }

    fn locked_consume(
        &mut self,
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
        persist: bool,
        peeks: BTreeSet<i32>,
        consume_ref: Consume,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit locked_consume");
        // println!(
        //     "consume: searching for data matching <patterns: {:?}> at <channels: {:?}>",
        //     patterns, channels
        // );

        self.log_consume(consume_ref.clone(), &channels, &patterns, &continuation, persist, &peeks);

        let channel_to_indexed_data = self.ops.fetch_channel_to_index_data(&channels);
        // println!("\nchannel_to_indexed_data: {:?}", channel_to_indexed_data);
        let zipped: Vec<(C, P)> = channels
            .iter()
            .cloned()
            .zip(patterns.iter().cloned())
            .collect();
        let options: Option<Vec<ConsumeCandidate<C, A>>> = self
            .ops
            .extract_data_candidates(&self.ops.matcher, zipped, channel_to_indexed_data, Vec::new())
            .into_iter()
            .collect();

        // println!("\noptions: {:?}", options);

        let wk = WaitingContinuation {
            patterns,
            continuation,
            persist,
            peeks: peeks.clone(),
            source: consume_ref.clone(),
        };

        match options {
            Some(data_candidates) => {
                let produce_counters_closure =
                    |produces: Vec<Produce>| self.ops.produce_counters(produces);

                self.log_comm(
                    &data_candidates,
                    &channels,
                    wk.clone(),
                    COMM::new(
                        data_candidates.clone(),
                        consume_ref.clone(),
                        peeks.clone(),
                        produce_counters_closure,
                    ),
                    CONSUME_COMM_LABEL,
                );
                self.ops
                    .store_persistent_data(data_candidates.clone(), &peeks);
                // println!(
                //     "consume: data found for <patterns: {:?}> at <channels: {:?}>",
                //     patterns, channels
                // );
                self.ops
                    .wrap_result(channels, wk, consume_ref, data_candidates)
            }
            None => {
                self.ops.store_waiting_continuation(channels, wk);
                None
            }
        }
    }

    fn locked_produce(
        &mut self,
        channel: C,
        data: A,
        persist: bool,
        produce_ref: Produce,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit locked_produce");
        let grouped_channels = self.ops.store.get_joins(channel.clone());
        // println!("\ngrouped_channels: {:?}", grouped_channels);
        // println!(
        //     "produce: searching for matching continuations at <grouped_channels: {:?}>",
        //     grouped_channels
        // );
        let _ = self.log_produce(produce_ref.clone(), &channel, &data, persist);
        let extracted = self.extract_produce_candidate(
            grouped_channels,
            channel.clone(),
            Datum {
                a: data.clone(),
                persist,
                source: produce_ref.clone(),
            },
        );

        // println!("extracted in lockedProduce: {:?}", extracted);

        match extracted {
            Some(produce_candidate) => self.process_match_found(produce_candidate),
            None => self.ops.store_data(channel, data, persist, produce_ref),
        }
    }

    /*
     * Find produce candidate
     *
     * NOTE: On Rust side, we are NOT passing functions through. Instead just the data.
     * And then in 'run_matcher_for_channels' we call the functions defined below
     */
    fn extract_produce_candidate(
        &self,
        grouped_channels: Vec<Vec<C>>,
        bat_channel: C,
        data: Datum<A>,
    ) -> MaybeProduceCandidate<C, P, A, K> {
        // println!("\nHit extract_produce_candidate");

        let fetch_matching_continuations =
            |channels: Vec<C>| -> Vec<(WaitingContinuation<P, K>, i32)> {
                let continuations = self.ops.store.get_continuations(channels);
                self.ops.shuffle_with_index(continuations)
            };

        /*
         * Here, we create a cache of the data at each channel as `channelToIndexedData`
         * which is used for finding matches.  When a speculative match is found, we can
         * remove the matching datum from the remaining data candidates in the cache.
         *
         * Put another way, this allows us to speculatively remove matching data without
         * affecting the actual store contents.
         *
         * In this version, we also add the produced data directly to this cache.
         */
        let fetch_matching_data = |channel| -> (C, Vec<(Datum<A>, i32)>) {
            let data_vec = self.ops.store.get_data(&channel);
            let mut shuffled_data = self.ops.shuffle_with_index(data_vec);
            if channel == bat_channel {
                shuffled_data.insert(0, (data.clone(), -1));
            }
            (channel, shuffled_data)
        };

        self.ops.run_matcher_for_channels(
            grouped_channels,
            fetch_matching_continuations,
            fetch_matching_data,
        )
    }

    fn process_match_found(
        &mut self,
        pc: ProduceCandidate<C, P, A, K>,
    ) -> MaybeActionResult<C, P, A, K> {
        let ProduceCandidate {
            channels,
            continuation,
            continuation_index,
            data_candidates,
        } = pc;

        let WaitingContinuation {
            patterns: _patterns,
            continuation: _cont,
            persist,
            peeks,
            source: consume_ref,
        } = &continuation;

        let produce_counters_closure = |produces: Vec<Produce>| self.ops.produce_counters(produces);
        self.log_comm(
            &data_candidates,
            &channels,
            continuation.clone(),
            COMM::new(
                data_candidates.clone(),
                consume_ref.clone(),
                peeks.clone(),
                produce_counters_closure,
            ),
            PRODUCE_COMM_LABEL,
        );

        if !persist {
            self.ops
                .store
                .remove_continuation(channels.clone(), continuation_index);
        }

        self.ops
            .remove_matched_datum_and_join(channels.clone(), data_candidates.clone());

        // println!(
        //     "produce: matching continuation found at <channels: {:?}>",
        //     channels
        // );

        self.ops
            .wrap_result(channels, continuation.clone(), consume_ref.clone(), data_candidates)
    }

    fn log_comm(
        &mut self,
        _data_candidates: &Vec<ConsumeCandidate<C, A>>,
        _channels: &Vec<C>,
        _wk: WaitingContinuation<P, K>,
        comm: COMM,
        _label: &str,
    ) -> COMM {
        self.ops.event_log.insert(0, Event::Comm(comm.clone()));
        comm
    }

    fn log_consume(
        &mut self,
        consume_ref: Consume,
        _channels: &Vec<C>,
        _patterns: &Vec<P>,
        _continuation: &K,
        _persist: bool,
        _peeks: &BTreeSet<i32>,
    ) -> Consume {
        self.ops
            .event_log
            .insert(0, Event::IoEvent(IOEvent::Consume(consume_ref.clone())));

        consume_ref
    }

    fn log_produce(
        &mut self,
        produce_ref: Produce,
        _channel: &C,
        _data: &A,
        persist: bool,
    ) -> Produce {
        self.ops
            .event_log
            .insert(0, Event::IoEvent(IOEvent::Produce(produce_ref.clone())));
        if !persist {
            // let entry = self.produce_counter.entry(produce_ref.clone()).or_insert(0);
            // *entry += 1;
            match self.ops.produce_counter.get(&produce_ref) {
                Some(current_count) => self
                    .ops
                    .produce_counter
                    .insert(produce_ref.clone(), current_count + 1),
                None => self.ops.produce_counter.insert(produce_ref.clone(), 1),
            };
        }

        produce_ref
    }

    pub fn spawn(&self) -> Result<Self, RSpaceError> {
        let history_repo = &self.ops.history_repository;
        let next_history = history_repo.reset(&history_repo.root())?;
        let history_reader = next_history.get_history_reader(next_history.root())?;
        let hot_store = HotStoreInstances::create_from_hr(history_reader.base());
        let mut rspace = Self::apply(Arc::new(next_history), hot_store, self.ops.matcher.clone());
        rspace.ops.restore_installs();

        // println!("\nRSpace Store in spawn: ");
        // rspace.store.print().await;

        // println!("\nRSpace History Store in spawn: ");
        // rspace.history_repository.

        Ok(rspace)
    }
}
