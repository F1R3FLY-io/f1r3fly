use crate::rspace::checkpoint::Checkpoint;
use crate::rspace::concurrent::two_step_lock::TwoStepLock;
use crate::rspace::event::{Consume, Produce};
use crate::rspace::history::history_repository::HistoryRepository;
use crate::rspace::history::history_repository::HistoryRepositoryInstances;
use crate::rspace::hot_store::{HotStore, HotStoreInstances};
use crate::rspace::internal::*;
use crate::rspace::matcher::r#match::Match;
use crate::rspace::shared::key_value_store::KeyValueStore;
use crate::rspace::space_matcher::SpaceMatcher;
use dashmap::DashMap;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;

// See rspace/src/main/scala/coop/rchain/rspace/RSpace.scala
// NOTE: 'space_matcher' field is added on Rust side to behave like Scala's 'extend'
// NOTE: 'store' field and methods are public for testing purposes. Production should be private?
#[repr(C)]
pub struct RSpace<C: Clone + Ord, P, A, K, M: Match<P, A>> {
    history_repository: Box<dyn HistoryRepository<C, P, A, K>>,
    pub store: Box<dyn HotStore<C, P, A, K>>,
    space_matcher: SpaceMatcher<C, P, A, K, M>,
    two_step_lock: TwoStepLock<C>,
}

type MaybeProduceCandidate<C, P, A, K> = Option<ProduceCandidate<C, P, A, K>>;
type MaybeActionResult<C, P, A, K> = Option<(ContResult<C, P, K>, Vec<RSpaceResult<C, A>>)>;

// NOTE: Currently NOT implementing any 'Log' functions
// NOTE: Implementing 'RSpaceOps' functions in this file
impl<
        C: Hash + Eq + Clone + Debug + Serialize + Ord,
        P: Clone + Debug + Serialize,
        A: Clone + Debug + Serialize,
        K: Clone + Debug + Serialize,
        M: Match<P, A>,
    > RSpace<C, P, A, K, M>
{
    fn locked_consume(
        &self,
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

        let channel_to_indexed_data = self.fetch_channel_to_index_data(&channels).unwrap();
        // println!("\nchannel_to_indexed_data: {:?}", channel_to_indexed_data);
        let zipped: Vec<(C, P)> = channels
            .iter()
            .cloned()
            .zip(patterns.iter().cloned())
            .collect();
        let options: Option<Vec<ConsumeCandidate<C, A>>> = self
            .space_matcher
            .extract_data_candidates(zipped, channel_to_indexed_data, Vec::new())
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
                // println!(
                //     "consume: data found for <patterns: {:?}> at <channels: {:?}>",
                //     patterns, channels
                // );
                self.store_persistent_data(data_candidates.clone(), peeks);
                self.wrap_result(channels, wk, consume_ref, data_candidates)
            }
            None => {
                self.store_waiting_continuation(channels, wk);
                None
            }
        }
    }

    /*
     * Here, we create a cache of the data at each channel as `channelToIndexedData`
     * which is used for finding matches.  When a speculative match is found, we can
     * remove the matching datum from the remaining data candidates in the cache.
     *
     * Put another way, this allows us to speculatively remove matching data without
     * affecting the actual store contents.
     */
    fn fetch_channel_to_index_data(
        &self,
        channels: &Vec<C>,
    ) -> Option<DashMap<C, Vec<(Datum<A>, i32)>>> {
        let map = DashMap::new();
        for c in channels {
            let data = self.store.get_data(c);
            let shuffled_data = self.shuffle_with_index(data);
            map.insert(c.clone(), shuffled_data);
        }
        Some(map)
    }

    fn locked_produce(
        &self,
        channel: C,
        data: A,
        persist: bool,
        produce_ref: Produce,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit locked_produce");
        let grouped_channels = self.store.get_joins(channel.clone());
        // println!("\ngrouped_channels: {:?}", grouped_channels);
        // println!(
        //     "produce: searching for matching continuations at <grouped_channels: {:?}>",
        //     grouped_channels
        // );
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
            None => self.store_data(channel, data, persist, produce_ref),
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
        self.run_matcher_for_channels(grouped_channels, bat_channel, data)
    }

    fn process_match_found(
        &self,
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
            peeks: _peeks,
            source,
        } = &continuation;

        if !persist {
            self.store
                .remove_continuation(channels.clone(), continuation_index);
        }

        self.remove_matched_datum_and_join(channels.clone(), data_candidates.clone());

        // println!(
        //     "produce: matching continuation found at <channels: {:?}>",
        //     channels
        // );

        self.wrap_result(channels, continuation.clone(), source.clone(), data_candidates)
    }

    fn create_checkpoint(&self) -> Checkpoint {
        let changes = self.store.changes();
        todo!()
    }

    /* RSpaceOps */

    fn shuffle_with_index<D>(&self, t: Vec<D>) -> Vec<(D, i32)> {
        let mut rng = thread_rng();
        let mut indexed_vec = t
            .into_iter()
            .enumerate()
            .map(|(i, d)| (d, i as i32))
            .collect::<Vec<_>>();
        indexed_vec.shuffle(&mut rng);
        indexed_vec
    }

    fn store_waiting_continuation(
        &self,
        channels: Vec<C>,
        wc: WaitingContinuation<P, K>,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit store_waiting_continuation");
        self.store.put_continuation(channels.clone(), wc);
        for channel in channels.iter() {
            self.store.put_join(channel.clone(), channels.clone());
            // println!("consume: no data found, storing <(patterns, continuation): ({:?}, {:?})> at <channels: {:?}>", wc.patterns, wc.continuation, channels)
        }
        None
    }

    fn store_data(
        &self,
        channel: C,
        data: A,
        persist: bool,
        produce_ref: Produce,
    ) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit store_data");
        let _ = self.store.put_datum(
            channel,
            Datum {
                a: data,
                persist,
                source: produce_ref,
            },
        );
        // println!(
        //     "produce: persisted <data: {:?}> at <channel: {:?}>",
        //     data, channel
        // );

        None
    }

    fn store_persistent_data(
        &self,
        data_candidates: Vec<ConsumeCandidate<C, A>>,
        _peeks: BTreeSet<i32>,
    ) -> Option<Vec<()>> {
        data_candidates
            .into_iter()
            .rev()
            .map(|consume_candidate| {
                let ConsumeCandidate {
                    channel,
                    datum: Datum { persist, .. },
                    removed_datum: _,
                    datum_index,
                } = consume_candidate;
                if !persist {
                    self.store.remove_datum(channel, datum_index)
                } else {
                    Some(())
                }
            })
            .collect()
    }

    pub fn consume(
        &self,
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
            println!("\nlocked_consume result: {:?}", result);
            result
        }
    }

    // pub async fn produce(
    //     &self,
    //     channel: C,
    //     data: A,
    //     persist: bool,
    // ) -> MaybeActionResult<C, P, A, K> {
    //     let produce_ref = Produce::create(channel.clone(), data.clone(), persist);
    //     let channel_clone = channel.clone();
    //     let locked_produce_result = self.two_step_lock.acquire(
    //         vec![channel_clone.clone()],
    //         || {
    //             self.store
    //                 .get_joins(channel_clone)
    //                 .map(|joins| joins.into_iter().flatten().collect::<Vec<_>>())
    //                 .unwrap()
    //         },
    //         || self.locked_produce(channel, data, persist, produce_ref),
    //     );
    //     locked_produce_result.await
    // }
    pub fn produce(&self, channel: C, data: A, persist: bool) -> MaybeActionResult<C, P, A, K> {
        // println!("\nHit produce");
        // println!("\nto_map: {:?}", self.store.to_map());
        let produce_ref = Produce::create(channel.clone(), data.clone(), persist);
        let result = self.locked_produce(channel, data, persist, produce_ref);
        println!("\nlocked_produce result: {:?}", result);
        result
    }

    pub fn install(
        &self,
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
    ) -> Option<(K, Vec<A>)> {
        self.locked_install(channels, patterns, continuation)
    }

    // From the logic in the Scala implementation, I think this either only returns 'None' or an error
    fn locked_install(
        &self,
        channels: Vec<C>,
        patterns: Vec<P>,
        continuation: K,
    ) -> Option<(K, Vec<A>)> {
        // println!("\nhit locked_install");
        // println!("channels: {:?}", channels);
        // println!("patterns: {:?}", patterns);
        // println!("continuation: {:?}", continuation);
        if channels.len() != patterns.len() {
            panic!("RUST ERROR: channels.length must equal patterns.length");
        } else {
            // println!(
            //     "install: searching for data matching <patterns: {:?}> at <channels: {:?}>",
            //     patterns, channels
            // );

            let consume_ref =
                Consume::create(channels.clone(), patterns.clone(), continuation.clone(), true);
            let channel_to_indexed_data = self.fetch_channel_to_index_data(&channels)?;
            let zipped: Vec<(C, P)> = channels
                .iter()
                .cloned()
                .zip(patterns.iter().cloned())
                .collect();
            let options: Option<Vec<ConsumeCandidate<C, A>>> = self
                .space_matcher
                .extract_data_candidates(zipped, channel_to_indexed_data, Vec::new())
                .into_iter()
                .collect();

            // println!("options in locked_install: {:?}", options);

            let result = match options {
                None => {
                    self.store.install_continuation(
                        channels.clone(),
                        WaitingContinuation {
                            patterns,
                            continuation,
                            persist: true,
                            peeks: BTreeSet::default(),
                            source: consume_ref,
                        },
                    );
                    for channel in channels.iter() {
                        self.store.install_join(channel.clone(), channels.clone());
                    }
                    // println!(
                    //     "storing <(patterns, continuation): ({:?}, {:?})> at <channels: {:?}>",
                    //     patterns, continuation, channels
                    // );
                    // println!("to_map: {:?}\n", self.store.to_map());
                    None
                }
                Some(_) => {
                    panic!("RUST ERROR: Installing can be done only on startup")
                }
            };
            result
        }
    }

    // pub fn clear(&self) -> () {
    //     self.two_step_lock.clean_up();
    //     self.store.clear()
    // }

    fn wrap_result(
        &self,
        channels: Vec<C>,
        wk: WaitingContinuation<P, K>,
        _consume_ref: Consume,
        data_candidates: Vec<ConsumeCandidate<C, A>>,
    ) -> MaybeActionResult<C, P, A, K> {
        let cont_result = ContResult {
            continuation: wk.continuation,
            persistent: wk.persist,
            channels,
            patterns: wk.patterns,
            peek: !wk.peeks.is_empty(),
        };

        let rspace_results = data_candidates
            .into_iter()
            .map(|data_candidate| RSpaceResult {
                channel: data_candidate.channel,
                matched_datum: data_candidate.datum.a,
                removed_datum: data_candidate.removed_datum,
                persistent: data_candidate.datum.persist,
            })
            .collect();

        Some((cont_result, rspace_results))
    }

    fn remove_matched_datum_and_join(
        &self,
        channels: Vec<C>,
        data_candidates: Vec<ConsumeCandidate<C, A>>,
    ) -> Option<Vec<()>> {
        data_candidates
            .into_iter()
            .rev()
            .map(|consume_candidate| {
                let ConsumeCandidate {
                    channel,
                    datum: Datum { persist, .. },
                    removed_datum: _,
                    datum_index,
                } = consume_candidate;
                if datum_index >= 0 && !persist {
                    self.store.remove_datum(channel.clone(), datum_index);
                }
                self.store.remove_join(channel, channels.clone());

                Some(())
            })
            .collect()
    }

    // NOTE: This function is a parameter on the Scala side for the function 'run_matcher_for_channels'
    fn fetch_matching_continuations(
        &self,
        channels: Vec<C>,
    ) -> Vec<(WaitingContinuation<P, K>, i32)> {
        let continuations = self.store.get_continuations(channels);
        self.shuffle_with_index(continuations)
    }

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
    // NOTE: This function is a parameter on the Scala side for the function 'run_matcher_for_channels'
    fn fetch_matching_data(
        &self,
        channel: C,
        bat_channel: C,
        data: Datum<A>,
    ) -> Option<(C, Vec<(Datum<A>, i32)>)> {
        let data_vec = self.store.get_data(&channel);
        let mut shuffled_data = self.shuffle_with_index(data_vec);
        if channel == bat_channel {
            shuffled_data.insert(0, (data, -1));
        }
        Some((channel, shuffled_data))
    }

    /*
     * NOTE: On Rust side we have removed the two function parameters:
     * 'fetchMatchingContinuations' and 'fetchMatchingData'.
     * Instead we call them directly with the corresponding data passed through
     */
    fn run_matcher_for_channels(
        &self,
        grouped_channels: Vec<Vec<C>>,
        bat_channel_for_data_function: C,
        data_for_data_function: Datum<A>,
    ) -> MaybeProduceCandidate<C, P, A, K> {
        let mut remaining = grouped_channels;

        loop {
            match remaining.split_first() {
                Some((channels, rest)) => {
                    let match_candidates = self.fetch_matching_continuations(channels.to_vec());
                    // println!("match_candidates: {:?}", match_candidates);
                    let channel_to_indexed_data_list: Vec<(C, Vec<(Datum<A>, i32)>)> = channels
                        .iter()
                        .filter_map(|c| {
                            self.fetch_matching_data(
                                c.clone(),
                                bat_channel_for_data_function.clone(),
                                data_for_data_function.clone(),
                            )
                        })
                        .collect();
                    // println!("channel_to_indexed_data_list: {:?}", channel_to_indexed_data_list);

                    let first_match = self.space_matcher.extract_first_match(
                        channels.to_vec(),
                        match_candidates,
                        channel_to_indexed_data_list.into_iter().collect(),
                    );

                    // println!("first_match in run_matcher_for_channels: {:?}", first_match);

                    match first_match {
                        Some(produce_candidate) => return Some(produce_candidate),
                        None => remaining = rest.to_vec(),
                    }
                }
                None => {
                    // println!("returning none in in run_matcher_for_channels");
                    return None;
                }
            }
        }
    }
}

pub struct RSpaceInstances;

pub struct RSpaceStore {
    pub history: Box<dyn KeyValueStore>,
    pub roots: Box<dyn KeyValueStore>,
    pub cold: Box<dyn KeyValueStore>,
}

impl RSpaceInstances {
    /**
     * Creates [[RSpace]] from [[HistoryRepository]] and [[HotStore]].
     */
    pub fn apply<
        C: Clone + Ord + Hash + Debug,
        P: Clone + Debug,
        A: Clone + Debug,
        K: Clone + Debug,
        M: Match<P, A>,
    >(
        history_repository: Box<dyn HistoryRepository<C, P, A, K>>,
        store: Box<dyn HotStore<C, P, A, K>>,
        matcher: M,
    ) -> RSpace<C, P, A, K, M> {
        RSpace {
            history_repository,
            store,
            space_matcher: SpaceMatcher::create(matcher),
            two_step_lock: TwoStepLock::new(),
        }
    }

    pub fn create<
        C: Clone + Ord + Default + Debug + Hash,
        P: Clone + Debug + Default,
        A: Clone + Debug + Default,
        K: Clone + Debug + Default,
        U: KeyValueStore + Clone,
        M: Match<P, A>,
    >(
        store: RSpaceStore,
        matcher: M,
    ) -> RSpace<C, P, A, K, M> {
        let setup = RSpaceInstances::create_history_repo::<C, P, A, K, U>(store);
        let (history_reader, store) = setup;
        let space = RSpaceInstances::apply(history_reader, store, matcher);
        space
    }

    /**
     * Creates [[HistoryRepository]] and [[HotStore]].
     */
    pub fn create_history_repo<
        C: Clone + Eq + Hash + Debug + Default,
        P: Clone + Default + Debug,
        A: Clone + Default + Debug,
        K: Clone + Default + Debug,
        U: KeyValueStore + Clone,
    >(
        store: RSpaceStore,
    ) -> (impl HistoryRepository<C, P, A, K>, impl HotStore<C, P, A, K>) {
        let history_repo = HistoryRepositoryInstances::lmdb_repository::<U>(
            store.history,
            store.roots,
            store.cold,
        );

        let history_reader = history_repo.get_history_reader(history_repo.root());

        let hot_store = HotStoreInstances::create_from_hr(history_reader.base());

        (history_repo, hot_store)
    }
}
