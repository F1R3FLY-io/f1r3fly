// See rspace/src/main/scala/coop/rchain/rspace/merger/EventLogIndex.scala

use std::collections::HashSet;

use crate::rspace::trace::event::{Consume, Event, IOEvent, Produce};

use super::merging_logic::NumberChannelsDiff;

pub struct EventLogIndex {
    pub produces_linear: HashSet<Produce>,
    pub produces_persistent: HashSet<Produce>,
    pub produces_consumed: HashSet<Produce>,
    pub produces_peeked: HashSet<Produce>,
    pub produces_copied_by_peek: HashSet<Produce>,
    pub produces_touching_base_joins: HashSet<Produce>,
    pub consumes_linear_and_peeks: HashSet<Consume>,
    pub consumes_persistent: HashSet<Consume>,
    pub consumes_produced: HashSet<Consume>,
    pub produces_mergeable: HashSet<Produce>,
    pub consumes_mergeable: HashSet<Consume>,
    pub number_channels_data: NumberChannelsDiff,
}

impl EventLogIndex {
    pub fn new(
        event_log: Vec<Event>,
        produce_exists_in_pre_state: impl Fn(&Produce) -> bool,
        produce_touch_pre_state_join: impl Fn(&Produce) -> bool,
        mergeable_chs: NumberChannelsDiff,
    ) -> Self {
        let mut produces_linear = HashSet::new();
        let mut produces_persistent = HashSet::new();
        let mut produces_consumed = HashSet::new();
        let mut produces_peeked = HashSet::new();
        let mut produces_copied_by_peek = HashSet::new();
        let mut produces_touching_base_joins = HashSet::new();
        let mut consumes_linear_and_peeks = HashSet::new();
        let mut consumes_persistent = HashSet::new();
        let mut consumes_produced = HashSet::new();

        for event in event_log {
            match event {
                Event::IoEvent(IOEvent::Produce(p)) => {
                    let exists_in_pre_state = produce_exists_in_pre_state(&p);
                    let touch_pre_state_join = produce_touch_pre_state_join(&p);

                    if exists_in_pre_state {
                        produces_copied_by_peek.insert(p.clone());
                    }

                    if touch_pre_state_join {
                        produces_touching_base_joins.insert(p.clone());
                    }

                    if p.persistent {
                        produces_persistent.insert(p);
                    } else {
                        produces_linear.insert(p);
                    }
                }
                Event::IoEvent(IOEvent::Consume(c)) => {
                    if c.persistent {
                        consumes_persistent.insert(c);
                    } else {
                        consumes_linear_and_peeks.insert(c);
                    }
                }
                Event::Comm(comm) => {
                    consumes_produced.insert(comm.consume);

                    if comm.peeks.is_empty() {
                        for p in comm.produces {
                            produces_consumed.insert(p);
                        }
                    } else {
                        for p in comm.produces {
                            produces_peeked.insert(p);
                        }
                    }
                }
            }
        }

        // Calculate mergeable channels
        let all_produces: HashSet<Produce> = produces_linear
            .iter()
            .chain(produces_persistent.iter())
            .chain(produces_consumed.iter())
            .chain(produces_peeked.iter())
            .cloned()
            .collect();

        let produces_mergeable = all_produces
            .into_iter()
            .filter(|p| mergeable_chs.contains_key(&p.channel_hash))
            .collect();

        let all_consumes: HashSet<Consume> = consumes_linear_and_peeks
            .iter()
            .chain(consumes_persistent.iter())
            .chain(consumes_produced.iter())
            .cloned()
            .collect();

        let consumes_mergeable = all_consumes
            .into_iter()
            .filter(|c| {
                c.channel_hashes
                    .iter()
                    .any(|hash| mergeable_chs.contains_key(hash))
            })
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
            produces_linear: HashSet::new(),
            produces_persistent: HashSet::new(),
            produces_consumed: HashSet::new(),
            produces_peeked: HashSet::new(),
            produces_copied_by_peek: HashSet::new(),
            produces_touching_base_joins: HashSet::new(),
            consumes_linear_and_peeks: HashSet::new(),
            consumes_persistent: HashSet::new(),
            consumes_produced: HashSet::new(),
            produces_mergeable: HashSet::new(),
            consumes_mergeable: HashSet::new(),
            number_channels_data: NumberChannelsDiff::new(),
        }
    }

    pub fn combine(x: Self, y: Self) -> Self {
        todo!()
    }
}
