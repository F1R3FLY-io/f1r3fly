// See rspace/src/main/scala/coop/rchain/rspace/merger/EventLogIndex.scala

use rayon::prelude::*;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use crate::rspace::trace::event::{Consume, Event, IOEvent, Produce};

use super::merging_logic::{NumberChannelsDiff, combine_produces_copied_by_peek};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct EventLogIndex {
    pub produces_linear: BTreeSet<Produce>,
    pub produces_persistent: BTreeSet<Produce>,
    pub produces_consumed: BTreeSet<Produce>,
    pub produces_peeked: BTreeSet<Produce>,
    pub produces_copied_by_peek: BTreeSet<Produce>,
    pub produces_touching_base_joins: BTreeSet<Produce>,
    pub consumes_linear_and_peeks: BTreeSet<Consume>,
    pub consumes_persistent: BTreeSet<Consume>,
    pub consumes_produced: BTreeSet<Consume>,
    pub produces_mergeable: BTreeSet<Produce>,
    pub consumes_mergeable: BTreeSet<Consume>,
    pub number_channels_data: NumberChannelsDiff,
}

impl EventLogIndex {
    pub fn new(
        event_log: Vec<Event>,
        produce_exists_in_pre_state: impl Fn(&Produce) -> bool,
        produce_touch_pre_state_join: impl Fn(&Produce) -> bool,
        mergeable_chs: NumberChannelsDiff,
    ) -> Self {
        // Use Arc<Mutex<>> for thread-safe collections that will be updated in parallel
        let produces_linear = Arc::new(Mutex::new(BTreeSet::new()));
        let produces_persistent = Arc::new(Mutex::new(BTreeSet::new()));
        let produces_consumed = Arc::new(Mutex::new(BTreeSet::new()));
        let produces_peeked = Arc::new(Mutex::new(BTreeSet::new()));
        let produces_copied_by_peek = Arc::new(Mutex::new(BTreeSet::new()));
        let produces_touching_base_joins = Arc::new(Mutex::new(BTreeSet::new()));
        let consumes_linear_and_peeks = Arc::new(Mutex::new(BTreeSet::new()));
        let consumes_persistent = Arc::new(Mutex::new(BTreeSet::new()));
        let consumes_produced = Arc::new(Mutex::new(BTreeSet::new()));

        // Pre-process events to collect all produces and consumes
        // This allows us to clone each event at most once
        let mut all_produces: Vec<(Produce, bool, bool)> = Vec::new(); // (produce, exists_in_pre_state, touches_join)
        let mut all_consumes: Vec<Consume> = Vec::new();
        let mut all_comms: Vec<(Consume, Vec<Produce>, bool)> = Vec::new(); // (consume, produces, has_peeks)

        // First, gather all events to avoid locking overhead in the parallel section
        for event in &event_log {
            match event {
                Event::IoEvent(IOEvent::Produce(p)) => {
                    let exists = produce_exists_in_pre_state(p);
                    let touches_join = produce_touch_pre_state_join(p);
                    all_produces.push((p.clone(), exists, touches_join));
                }
                Event::IoEvent(IOEvent::Consume(c)) => {
                    all_consumes.push(c.clone());
                }
                Event::Comm(comm) => {
                    let has_peeks = !comm.peeks.is_empty();
                    all_comms.push((comm.consume.clone(), comm.produces.clone(), has_peeks));
                }
            }
        }

        // Process the collected events in parallel
        // Produces
        all_produces
            .par_iter()
            .for_each(|(p, exists, touches_join)| {
                if *exists {
                    if let Ok(mut set) = produces_copied_by_peek.lock() {
                        set.insert(p.clone());
                    }
                }

                if *touches_join {
                    if let Ok(mut set) = produces_touching_base_joins.lock() {
                        set.insert(p.clone());
                    }
                }

                if let Ok(mut set) = if p.persistent {
                    produces_persistent.lock()
                } else {
                    produces_linear.lock()
                } {
                    set.insert(p.clone());
                }
            });

        // Consumes
        all_consumes.par_iter().for_each(|c| {
            if let Ok(mut set) = if c.persistent {
                consumes_persistent.lock()
            } else {
                consumes_linear_and_peeks.lock()
            } {
                set.insert(c.clone());
            }
        });

        // COMM events
        all_comms
            .par_iter()
            .for_each(|(consume, produces, has_peeks)| {
                if let Ok(mut set) = consumes_produced.lock() {
                    set.insert(consume.clone());
                }

                let target_set = if *has_peeks {
                    &produces_peeked
                } else {
                    &produces_consumed
                };

                for p in produces {
                    if let Ok(mut set) = target_set.lock() {
                        set.insert(p.clone());
                    }
                }
            });

        // Helper function to safely unwrap Arc<Mutex<BTreeSet<T>>> with minimal cloning
        fn unwrap_arc_mutex<T>(arc_mutex: Arc<Mutex<BTreeSet<T>>>) -> BTreeSet<T>
        where
            T: Ord + Clone,
        {
            // Try to get exclusive ownership of the Arc
            match Arc::try_unwrap(arc_mutex) {
                // Success case: we have exclusive ownership, just unwrap the mutex
                Ok(mutex) => mutex.into_inner().unwrap_or_default(),

                // Can't get exclusive ownership - we need to construct a new set
                // with minimal cloning by draining items instead of cloning the whole set
                Err(arc) => {
                    let mut result = BTreeSet::new();
                    if let Ok(guard) = arc.lock() {
                        // Insert each item individually - still requires cloning elements
                        // but avoids cloning the entire collection structure
                        for item in guard.iter() {
                            result.insert(item.clone());
                        }
                    }
                    result
                }
            }
        }

        // Unwrap the Arc<Mutex<>> to get the final collections
        let produces_linear = unwrap_arc_mutex(produces_linear);
        let produces_persistent = unwrap_arc_mutex(produces_persistent);
        let produces_consumed = unwrap_arc_mutex(produces_consumed);
        let produces_peeked = unwrap_arc_mutex(produces_peeked);
        let produces_copied_by_peek = unwrap_arc_mutex(produces_copied_by_peek);
        let produces_touching_base_joins = unwrap_arc_mutex(produces_touching_base_joins);
        let consumes_linear_and_peeks = unwrap_arc_mutex(consumes_linear_and_peeks);
        let consumes_persistent = unwrap_arc_mutex(consumes_persistent);
        let consumes_produced = unwrap_arc_mutex(consumes_produced);

        // Calculate mergeable channels more efficiently
        // First create references to avoid redundant cloning
        let all_produces_refs: BTreeSet<&Produce> = produces_linear
            .iter()
            .chain(produces_persistent.iter())
            .chain(produces_consumed.iter())
            .chain(produces_peeked.iter())
            .collect();

        // Then filter and clone only once for the final set
        let produces_mergeable: BTreeSet<Produce> = all_produces_refs
            .into_iter()
            .filter(|p| mergeable_chs.contains_key(&p.channel_hash))
            .map(|p| (*p).clone())
            .collect();

        // Same approach for consumes
        let all_consumes_refs: BTreeSet<&Consume> = consumes_linear_and_peeks
            .iter()
            .chain(consumes_persistent.iter())
            .chain(consumes_produced.iter())
            .collect();

        let consumes_mergeable: BTreeSet<Consume> = all_consumes_refs
            .into_iter()
            .filter(|c| {
                c.channel_hashes
                    .iter()
                    .any(|hash| mergeable_chs.contains_key(hash))
            })
            .map(|c| (*c).clone())
            .collect();

        EventLogIndex {
            produces_linear,
            produces_persistent,
            produces_consumed,
            produces_peeked,
            produces_copied_by_peek,
            produces_touching_base_joins,
            consumes_linear_and_peeks,
            consumes_persistent,
            consumes_produced,
            produces_mergeable,
            consumes_mergeable,
            number_channels_data: mergeable_chs,
        }
    }

    pub fn empty() -> Self {
        EventLogIndex {
            produces_linear: BTreeSet::new(),
            produces_persistent: BTreeSet::new(),
            produces_consumed: BTreeSet::new(),
            produces_peeked: BTreeSet::new(),
            produces_copied_by_peek: BTreeSet::new(),
            produces_touching_base_joins: BTreeSet::new(),
            consumes_linear_and_peeks: BTreeSet::new(),
            consumes_persistent: BTreeSet::new(),
            consumes_produced: BTreeSet::new(),
            produces_mergeable: BTreeSet::new(),
            consumes_mergeable: BTreeSet::new(),
            number_channels_data: NumberChannelsDiff::new(),
        }
    }

    pub fn combine(x: Self, y: Self) -> Self {
        EventLogIndex {
            produces_linear: x
                .produces_linear
                .union(&y.produces_linear)
                .cloned()
                .collect(),
            produces_persistent: x
                .produces_persistent
                .union(&y.produces_persistent)
                .cloned()
                .collect(),
            produces_consumed: x
                .produces_consumed
                .union(&y.produces_consumed)
                .cloned()
                .collect(),
            produces_peeked: x
                .produces_peeked
                .union(&y.produces_peeked)
                .cloned()
                .collect(),
            produces_copied_by_peek: combine_produces_copied_by_peek(&x, &y),
            //TODO this joins combination is very restrictive. Join might be originated inside aggregated event log - OLD
            produces_touching_base_joins: x
                .produces_touching_base_joins
                .union(&y.produces_touching_base_joins)
                .cloned()
                .collect(),
            consumes_linear_and_peeks: x
                .consumes_linear_and_peeks
                .union(&y.consumes_linear_and_peeks)
                .cloned()
                .collect(),
            consumes_persistent: x
                .consumes_persistent
                .union(&y.consumes_persistent)
                .cloned()
                .collect(),
            consumes_produced: x
                .consumes_produced
                .union(&y.consumes_produced)
                .cloned()
                .collect(),
            // Combine mergeable produces and consumes
            produces_mergeable: x
                .produces_mergeable
                .union(&y.produces_mergeable)
                .cloned()
                .collect(),
            consumes_mergeable: x
                .consumes_mergeable
                .union(&y.consumes_mergeable)
                .cloned()
                .collect(),
            // Merge number channels (add differences)
            number_channels_data: x
                .number_channels_data
                .iter()
                .chain(y.number_channels_data.iter())
                .fold(NumberChannelsDiff::new(), |mut acc, (key, value)| {
                    // Sum the values if key exists, otherwise just insert
                    acc.entry(key.clone())
                        .and_modify(|existing| *existing += value)
                        .or_insert(*value);
                    acc
                }),
        }
    }
}
