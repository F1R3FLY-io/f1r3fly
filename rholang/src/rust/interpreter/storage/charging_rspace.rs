// See rholang/src/main/scala/coop/rchain/rholang/interpreter/storage/ChargingRSpace.scala

use std::collections::BTreeSet;

use crate::rust::interpreter::{
    accounting::{
        _cost,
        costs::{
            comm_event_storage_cost, event_storage_cost, storage_cost_consume,
            storage_cost_produce, Cost,
        },
    },
    errors::InterpreterError,
    unwrap_option_safe,
};
use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{
    tagged_continuation::TaggedCont, BindPattern, ListParWithRandom, Par, TaggedContinuation,
};
use rspace_plus_plus::rspace::{
    errors::RSpaceError,
    rspace_interface::{ContResult, MaybeActionResult, RSpaceResult},
    tuplespace_interface::Tuplespace,
};

pub struct ChargingRSpace;

#[derive(Clone)]
pub enum TriggeredBy {
    Consume {
        id: Blake2b512Random,
        persistent: bool,
        channels_count: i64,
    },
    Produce {
        id: Blake2b512Random,
        persistent: bool,
        channels_count: i64,
    },
}

fn consume_id(continuation: TaggedContinuation) -> Result<Blake2b512Random, InterpreterError> {
    //TODO: Make ScalaBodyRef-s have their own random state and merge it during its COMMs - OLD
    match unwrap_option_safe(continuation.tagged_cont)? {
        TaggedCont::ParBody(par_with_random) => {
            Ok(Blake2b512Random::new(&par_with_random.random_state))
        }
        TaggedCont::ScalaBodyRef(value) => Ok(Blake2b512Random::new(&value.to_be_bytes())),
    }
}

impl ChargingRSpace {
    pub fn charging_rspace<T>(
        space: T,
        cost: _cost,
    ) -> impl Tuplespace<Par, BindPattern, ListParWithRandom, TaggedContinuation> + Clone
    where
        T: Tuplespace<Par, BindPattern, ListParWithRandom, TaggedContinuation> + Clone,
    {
        #[derive(Clone)]
        struct ChargingRSpace<T> {
            space: T,
            cost: _cost,
        }

        impl<T: Tuplespace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>
            Tuplespace<Par, BindPattern, ListParWithRandom, TaggedContinuation>
            for ChargingRSpace<T>
        {
            fn consume(
                &mut self,
                channels: Vec<Par>,
                patterns: Vec<BindPattern>,
                continuation: TaggedContinuation,
                persist: bool,
                peeks: BTreeSet<i32>,
            ) -> Result<
                MaybeActionResult<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
                RSpaceError,
            > {
                self.cost.charge(storage_cost_consume(
                    channels.clone(),
                    patterns.clone(),
                    continuation.clone(),
                ))?;

                let consume_res = self.space.consume(
                    channels.clone(),
                    patterns,
                    continuation.clone(),
                    persist,
                    peeks,
                )?;

                let id = consume_id(continuation)?;
                handle_result(
                    consume_res.clone(),
                    TriggeredBy::Consume {
                        id,
                        persistent: persist,
                        channels_count: channels.len() as i64,
                    },
                    self.cost.clone(),
                )?;
                Ok(consume_res)
            }

            fn produce(
                &mut self,
                channel: Par,
                data: ListParWithRandom,
                persist: bool,
            ) -> Result<
                MaybeActionResult<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
                RSpaceError,
            > {
                self.cost
                    .charge(storage_cost_produce(channel.clone(), data.clone()))?;
                let produce_res = self.space.produce(channel, data.clone(), persist)?;
                handle_result(
                    produce_res.clone(),
                    TriggeredBy::Produce {
                        id: Blake2b512Random::new(&data.random_state),
                        persistent: persist,
                        channels_count: 1,
                    },
                    self.cost.clone(),
                )?;
                Ok(produce_res)
            }

            fn install(
                &mut self,
                channels: Vec<Par>,
                patterns: Vec<BindPattern>,
                continuation: TaggedContinuation,
            ) -> Result<Option<(TaggedContinuation, Vec<ListParWithRandom>)>, RSpaceError>
            {
                self.space.install(channels, patterns, continuation)
            }
        }

        ChargingRSpace { space, cost }
    }
}

fn handle_result(
    result: MaybeActionResult<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    triggered_by: TriggeredBy,
    cost: _cost,
) -> Result<(), InterpreterError> {
    let triggered_by_id = match triggered_by.clone() {
        TriggeredBy::Consume { id, .. } => id,
        TriggeredBy::Produce { id, .. } => id,
    };
    let triggered_by_channels_count = match triggered_by {
        TriggeredBy::Consume { channels_count, .. } => channels_count,
        TriggeredBy::Produce { .. } => 1,
    };
    let triggered_by_persistent = match triggered_by {
        TriggeredBy::Consume { persistent, .. } => persistent,
        TriggeredBy::Produce { persistent, .. } => persistent,
    };

    match result {
        Some((cont, data_list)) => {
            let consume_id = consume_id(cont.continuation.clone())?;

            // We refund for non-persistent continuations, and for the persistent continuation triggering the comm.
            // That persistent continuation is going to be charged for (without refund) once it has no matches in TS.
            let refund_for_consume =
                if !cont.persistent || consume_id.to_vec() == triggered_by_id.to_vec() {
                    storage_cost_consume(
                        cont.channels.clone(),
                        cont.patterns.clone(),
                        cont.continuation.clone(),
                    )
                } else {
                    Cost::create(0, "refund_for_consume".to_string())
                };

            let refund_for_produces =
                refund_for_removing_produces(data_list, cont.clone(), triggered_by);

            cost.charge(Cost::create(
                -refund_for_consume.value,
                "consume storage refund".to_string(),
            ))?;
            cost.charge(Cost::create(
                -refund_for_produces.value,
                "produces storage refund".to_string(),
            ))?;

            let last_iteration = !triggered_by_persistent;

            if last_iteration {
                cost.charge(event_storage_cost(triggered_by_channels_count))?;
            }

            cost.charge(comm_event_storage_cost(cont.channels.len() as i64))
        }
        None => cost.charge(event_storage_cost(triggered_by_channels_count)),
    }
}

fn refund_for_removing_produces(
    data_list: Vec<RSpaceResult<Par, ListParWithRandom>>,
    cont: ContResult<Par, BindPattern, TaggedContinuation>,
    triggered_by: TriggeredBy,
) -> Cost {
    let triggered_id = match triggered_by {
        TriggeredBy::Consume { id, .. } => id,
        TriggeredBy::Produce { id, .. } => id,
    };

    let removed_data: Vec<(RSpaceResult<Par, ListParWithRandom>, Par)> = data_list
        .into_iter()
        .zip(cont.channels.into_iter())
        // A persistent produce is charged for upfront before reaching the TS, and needs to be refunded
        // after each iteration it matches an existing consume. We treat it as 'removed' on each such iteration.
        // It is going to be 'not removed' and charged for on the last iteration, where it doesn't match anything.
        .filter(|(data, _)| {
            !data.persistent || data.removed_datum.random_state == triggered_id.to_vec()
        })
        .collect();

    removed_data
        .into_iter()
        .map(|(data, channel)| storage_cost_produce(channel, data.removed_datum))
        .fold(
            Cost::create(0, "refund_for_removing_produces init".to_string()),
            |acc, cost| {
                Cost::create(
                    acc.value + cost.value,
                    "refund_for_removing_produces operation".to_string(),
                )
            },
        )
}