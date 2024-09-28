// See /home/spreston/src/firefly/f1r3fly/rspace/src/main/scala/coop/rchain/rspace/ReplayRSpace.scala

use super::checkpoint::SoftCheckpoint;
use super::errors::HistoryRepositoryError;
use super::errors::RSpaceError;
use super::hashing::blake2b256_hash::Blake2b256Hash;
use super::history::history_reader::HistoryReader;
use super::history::instances::radix_history::RadixHistory;
use super::r#match::Match;
use super::replay_rspace_interface::IReplayRSpace;
use super::rspace_interface::ContResult;
use super::rspace_interface::ISpace;
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

#[repr(C)]
pub struct ReplayRSpace<C, P, A, K> {
    ops: RSpaceOps<C, P, A, K>,
    replay_data: MultisetMultiMap<IOEvent, COMM>,
}

impl<C, P, A, K> Tuplespace<C, P, A, K> for ReplayRSpace<C, P, A, K>
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

        self.ops.check_replay_data(self.replay_data.clone());

        let changes = self.ops.store.changes();
        let next_history = self.ops.history_repository.checkpoint(&changes);
        self.ops.history_repository = Arc::new(next_history);

        let history_reader = self
            .ops
            .history_repository
            .get_history_reader(self.ops.history_repository.root())?;

        self.ops.create_new_hot_store(history_reader);
        self.ops.restore_installs();

        Ok(Checkpoint {
            root: self.ops.history_repository.root(),
            log: Vec::new(),
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

impl<C, P, A, K> ReplayRSpace<C, P, A, K>
where
    C: Clone + Debug + Default + Serialize + Hash + Ord + Eq + 'static + Sync + Send,
    P: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    A: Clone + Debug + Default + Serialize + 'static + Sync + Send,
    K: Clone + Debug + Default + Serialize + 'static + Sync + Send,
{
    /**
     * Creates [[ReplayRSpace]] from [[HistoryRepository]] and [[HotStore]].
     */
    pub fn apply(
        history_repository: Arc<Box<dyn HistoryRepository<C, P, A, K>>>,
        store: Box<dyn HotStore<C, P, A, K>>,
        matcher: Arc<Box<dyn Match<P, A>>>,
    ) -> ReplayRSpace<C, P, A, K>
    where
        C: Clone + Debug + Ord + Hash,
        P: Clone + Debug,
        A: Clone + Debug,
        K: Clone + Debug,
    {
        ReplayRSpace {
            ops: RSpaceOps {
                history_repository,
                store,
                installs: Mutex::new(HashMap::new()),
                event_log: Vec::new(),
                produce_counter: BTreeMap::new(),
                matcher,
            },
            replay_data: MultisetMultiMap::empty(),
        }
    }

    pub fn consume(
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
            // println!("\n{:#?}", self.store.to_map().len());
            // println!("\nreplay_consume none?: {:?}", result.is_none());
            result
        }
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
        // println!(
        //     "consume: searching for data matching <patterns: {:?}> at <channels: {:?}>",
        //     patterns, channels
        // );

        self.log_consume(consume_ref.clone(), &channels, &patterns, &continuation, persist, &peeks);

        let wk = WaitingContinuation {
            patterns: patterns.clone(),
            continuation,
            persist,
            peeks: peeks.clone(),
            source: consume_ref.clone(),
        };

        // println!("\nreplay_data len in replay_consume: {:?}", self.replay_data.map.len());

        let comms_option = self
            .replay_data
            .map
            .get(&IOEvent::Consume(consume_ref.clone()))
            .map(|comms| {
                comms
                    .iter()
                    .map(|tuple| tuple.0.clone())
                    .collect::<Vec<_>>()
            });

        // println!("\ncomms_options in replay_consume Some?: {:?}", comms_option.is_some());

        match comms_option {
            None => self.ops.store_waiting_continuation(channels, wk),
            Some(comms_list) => {
                match self.get_comm_and_consume_candidates(
                    channels.clone(),
                    patterns,
                    comms_list.clone(),
                ) {
                    None => {
                        // println!("\nwas none");
                        self.ops.store_waiting_continuation(channels, wk)
                    }
                    Some((_, data_candidates)) => {
                        let comm_ref = {
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
                            )
                        };

                        assert!(
                            comms_list.contains(&comm_ref),
                            "{}",
                            format!(
                                "COMM Event {:?} was not contained in the trace {:?}",
                                comm_ref, comms_list
                            )
                        );

                        let _ = self
                            .ops
                            .store_persistent_data(data_candidates.clone(), &peeks);
                        // println!(
                        //     "consume: data found for <patterns: {:?}> at <channels: {:?}>",
                        //     patterns, channels
                        // );
                        let _ = self.remove_bindings_for(comm_ref);
                        self.ops
                            .wrap_result(channels, wk, consume_ref, data_candidates)
                    }
                }
            }
        }
    }

    fn get_comm_and_consume_candidates(
        &self,
        channels: Vec<C>,
        patterns: Vec<P>,
        comms: Vec<COMM>,
    ) -> Option<(COMM, Vec<ConsumeCandidate<C, A>>)> {
        let run_matcher = |comm: COMM| -> Option<Vec<ConsumeCandidate<C, A>>> {
            self.run_matcher_consume(channels.clone(), patterns.clone(), comm)
        };

        self.get_comm_or_candidate(comms, run_matcher)
    }

    fn run_matcher_consume(
        &self,
        channels: Vec<C>,
        patterns: Vec<P>,
        comm: COMM,
    ) -> Option<Vec<ConsumeCandidate<C, A>>> {
        let mut channel_to_indexed_data_list: Vec<(C, Vec<(Datum<A>, i32)>)> = Vec::new();

        for c in &channels {
            let data = self.ops.store.get_data(c);
            // println!("\ndata len: {}", data.len());
            let filtered_data: Vec<(Datum<A>, i32)> = data
                .into_iter()
                .zip(0..)
                .filter(|(d, i)| self.matches(comm.clone(), (d.clone(), *i)))
                .collect();
            channel_to_indexed_data_list.push((c.clone(), filtered_data));
        }

        // println!("\nchannelToIndexedDataList: {:#?}", channel_to_indexed_data_list);

        let channel_to_indexed_data_map: DashMap<C, Vec<(Datum<A>, i32)>> =
            channel_to_indexed_data_list.into_iter().collect();

        let result = self
            .ops
            .extract_data_candidates(
                &self.ops.matcher,
                channels.into_iter().zip(patterns.into_iter()).collect(),
                channel_to_indexed_data_map,
                vec![],
            )
            .into_iter()
            .collect::<Option<Vec<_>>>();

        result
    }

    pub fn produce(&mut self, channel: C, data: A, persist: bool) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit replay_produce");
        // println!("\nto_map: {:?}", self.store.to_map());
        // println!("\nHit produce, data: {:?}", data);
        // println!("\n\nHit produce, channel: {:?}", channel);

        let produce_ref = Produce::create(channel.clone(), data.clone(), persist);
        let result = self.locked_produce(channel, data, persist, produce_ref);
        // println!("\nreplay_locked_produce result: {:?}", result);
        result
    }

    fn locked_produce(
        &mut self,
        channel: C,
        data: A,
        persist: bool,
        produce_ref: Produce,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit replay_locked_produce");

        let grouped_channels = self.ops.store.get_joins(channel.clone());
        // println!(
        //     "produce: searching for matching continuations at <grouped_channels: {:?}>",
        //     grouped_channels
        // );
        let _ = self.log_produce(produce_ref.clone(), &channel, &data, persist);

        let comms_option = self
            .replay_data
            .map
            .get(&IOEvent::Produce(produce_ref.clone()))
            .map(|comms| {
                comms
                    .iter()
                    .map(|tuple| tuple.0.clone())
                    .collect::<Vec<_>>()
            });

        // println!("\ncomms_options in replay_produce Some?: {:?}", comms_option.is_some());

        match comms_option {
            None => self.ops.store_data(channel, data, persist, produce_ref),
            Some(comms_list) => {
                match self.get_comm_or_produce_candidate(
                    channel.clone(),
                    data.clone(),
                    persist,
                    comms_list.clone(),
                    produce_ref.clone(),
                    grouped_channels,
                ) {
                    Some((_, pc)) => self.handle_match(pc, comms_list),
                    None => {
                        // println!("\nwas none");
                        self.ops.store_data(channel, data, persist, produce_ref)
                    }
                }
            }
        }
    }

    fn get_comm_or_produce_candidate(
        &self,
        channel: C,
        data: A,
        persist: bool,
        comms: Vec<COMM>,
        produce_ref: Produce,
        grouped_channels: Vec<Vec<C>>,
    ) -> Option<(COMM, ProduceCandidate<C, P, A, K>)> {
        let run_matcher = |comm: COMM| -> Option<ProduceCandidate<C, P, A, K>> {
            self.run_matcher_produce(
                channel.clone(),
                data.clone(),
                persist,
                comm,
                produce_ref.clone(),
                grouped_channels.clone(),
            )
        };

        self.get_comm_or_candidate(comms, run_matcher)
    }

    fn run_matcher_produce(
        &self,
        channel: C,
        data: A,
        persist: bool,
        comm: COMM,
        produce_ref: Produce,
        grouped_channels: Vec<Vec<C>>,
    ) -> Option<ProduceCandidate<C, P, A, K>> {
        self.ops.run_matcher_for_channels(
            grouped_channels,
            |channels| {
                let continuations = self.ops.store.get_continuations(channels);
                continuations
                    .into_iter()
                    .enumerate()
                    .filter(|(_, wc)| comm.consume == wc.source)
                    .map(|(i, wc)| (wc, i as i32))
                    .collect::<Vec<_>>()
            },
            |c| {
                let store_data = self.ops.store.get_data(&c);
                let datum_tuples = store_data
                    .into_iter()
                    .enumerate()
                    .map(|(i, d)| (d, i as i32))
                    .collect::<Vec<_>>();

                let mut result = datum_tuples;
                if c == channel {
                    result.insert(
                        0,
                        (
                            Datum {
                                a: data.clone(),
                                persist,
                                source: produce_ref.clone(),
                            },
                            -1,
                        ),
                    );
                }

                (
                    c.clone(),
                    result
                        .into_iter()
                        .filter(|(datum, i)| self.matches(comm.clone(), (datum.clone(), *i)))
                        .collect(),
                )
            },
        )
    }

    fn matches(&self, comm: COMM, datum_with_index: (Datum<A>, i32)) -> bool {
        // println!("\ncomm in matches: {:?}", comm);
        let datum = datum_with_index.0;
        let x = comm.produces.contains(&datum.source);
        let res = x && self.was_repeated_enough_times(comm, datum);
        // println!("\ncomm.produce.contains: {:?}", x);
        // println!("\nmatches result: {:?}", res);
        res
    }

    fn was_repeated_enough_times(&self, comm: COMM, datum: Datum<A>) -> bool {
        // println!("\ncomm in was_repeated_enough_times: {:?}", comm);
        // println!("\n\ndatum in was_repeated_enough_times: {:?}", datum);
        // println!("\nproduce_counter: {:?}", self.produce_counter);
        if !datum.persist {
            let x = comm.times_repeated.get(&datum.source).unwrap_or(&0)
                == self.ops.produce_counter.get(&datum.source).unwrap_or(&0);
            // println!("\nwas_repeated_enough_times result: {:?}", x);
            x
        } else {
            true
        }
    }

    fn handle_match(
        &mut self,
        pc: ProduceCandidate<C, P, A, K>,
        comms: Vec<COMM>,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nhit handle_match");

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
        let comm_ref = self.log_comm(
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

        assert!(
            comms.contains(&comm_ref),
            "COMM Event {:?} was not contained in the trace {:?}",
            comm_ref,
            comms
        );

        if !persist {
            let _ = self
                .ops
                .store
                .remove_continuation(channels.clone(), continuation_index);
        };

        let _ = self
            .ops
            .remove_matched_datum_and_join(channels.clone(), data_candidates.clone());
        // println!("produce: matching continuation found at <channels: {:?}>", channels);
        let _ = self.remove_bindings_for(comm_ref);
        self.ops
            .wrap_result(channels, continuation.clone(), consume_ref.clone(), data_candidates)
    }

    fn remove_bindings_for(&mut self, comm_ref: COMM) -> () {
        // println!("\nhit remove_bindings_for");

        let mut updated_replays = remove_binding(
            self.replay_data.clone(),
            IOEvent::Consume(comm_ref.clone().consume),
            comm_ref.clone(),
        );

        for produce_ref in comm_ref.produces.iter() {
            updated_replays = remove_binding(
                updated_replays,
                IOEvent::Produce(produce_ref.clone()),
                comm_ref.clone(),
            );
        }

        self.replay_data = updated_replays;
    }

    pub fn clear(&mut self) -> Result<(), RSpaceError> {
        self.replay_data.clear();
        self.ops.clear()
    }

    fn log_comm(
        &mut self,
        _data_candidates: &Vec<ConsumeCandidate<C, A>>,
        _channels: &Vec<C>,
        _wk: WaitingContinuation<P, K>,
        comm: COMM,
        _label: &str,
    ) -> COMM {
        // TODO: Metrics?
        // self.event_log.insert(0, Event::Comm(comm.clone()));
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
        consume_ref
    }

    fn log_produce(
        &mut self,
        produce_ref: Produce,
        _channel: &C,
        _data: &A,
        persist: bool,
    ) -> Produce {
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

    fn get_comm_or_candidate<Candidate>(
        &self,
        comms: Vec<COMM>,
        run_matcher: impl Fn(COMM) -> Option<Candidate>,
    ) -> Option<(COMM, Candidate)> {
        let go = |cs: Vec<COMM>| match cs.as_slice() {
            [] => {
                let msg = "List comms must not be empty";
                panic!("{}", msg);
            }
            [comm_ref] => match run_matcher(comm_ref.clone()) {
                Some(data_candidates) => Ok(Ok((comm_ref.clone(), data_candidates))),
                None => Ok(Err(comm_ref.clone())),
            },
            [comm_ref, rem @ ..] => match run_matcher(comm_ref.clone()) {
                Some(data_candidates) => Ok(Ok((comm_ref.clone(), data_candidates))),
                None => Err(rem.to_vec()),
            },
        };

        let mut cs = comms;
        loop {
            match go(cs.clone()) {
                Ok(Ok(comm_or_candidate)) => return Some(comm_or_candidate),
                Ok(Err(_)) => return None,
                Err(new_cs) => cs = new_cs,
            }
        }
    }

    // This function may need to clear 'replay_data'
    pub fn spawn(&self) -> Result<Self, RSpaceError> {
        let history_repo = &self.ops.history_repository;
        let next_history = history_repo.reset(&history_repo.root())?;
        let history_reader = next_history.get_history_reader(next_history.root())?;
        let hot_store = HotStoreInstances::create_from_hr(history_reader.base());
        let mut rspace = Self::apply(Arc::new(next_history), hot_store, self.ops.matcher.clone());
        rspace.ops.restore_installs();

        Ok(rspace)
    }
}
