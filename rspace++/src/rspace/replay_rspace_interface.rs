// See rspace/src/main/scala/coop/rchain/rspace/IReplaySpace.scala

use std::collections::HashSet;

use super::{
    errors::RSpaceError, hashing::blake2b256_hash::Blake2b256Hash, internal::MultisetMultiMap, rspace_interface::ISpace, trace::{
        event::{Event, IOEvent, COMM},
        Log,
    }
};

pub trait IReplayRSpace<C: Eq + std::hash::Hash, P: Clone, A: Clone, K: Clone>:
    ISpace<C, P, A, K>
{
    fn rig_and_reset(
        &mut self,
        start_root: Blake2b256Hash,
        log: Log,
        replay_data: MultisetMultiMap<IOEvent, COMM>,
    ) -> Result<(), RSpaceError> {
        self.rig(log, replay_data)?;
        self.reset(start_root)
    }

    fn rig(
        &self,
        log: Log,
        replay_data: MultisetMultiMap<IOEvent, COMM>,
    ) -> Result<(), RSpaceError> {
        // println!("\nlog len in rust rig: {:?}", log.len());
        let (io_events, comm_events): (Vec<_>, Vec<_>) =
            log.iter().partition(|event| match event {
                Event::IoEvent(IOEvent::Produce(_)) => true,
                Event::IoEvent(IOEvent::Consume(_)) => true,
                Event::Comm(_) => false,
            });

        // Create a set of the "new" IOEvents
        let new_stuff: HashSet<_> = io_events.into_iter().collect();

        // Create and prepare the ReplayData table
        replay_data.clear();

        for event in comm_events {
            match event {
                Event::Comm(comm) => {
                    let comm_cloned = comm.clone();
                    let (consume, produces) = (comm_cloned.consume, comm_cloned.produces);
                    let produce_io_events: Vec<IOEvent> = produces
                        .into_iter()
                        .map(|produce| IOEvent::Produce(produce))
                        .collect();

                    let mut io_events = produce_io_events.clone();
                    io_events.insert(0, IOEvent::Consume(consume));

                    for io_event in io_events {
                        let io_event_converted: Event = match io_event {
                            IOEvent::Produce(ref p) => Event::IoEvent(IOEvent::Produce(p.clone())),
                            IOEvent::Consume(ref c) => Event::IoEvent(IOEvent::Consume(c.clone())),
                        };

                        if new_stuff.contains(&io_event_converted) {
                            // println!("\nadd_binding in rig");
                            replay_data.add_binding(io_event, comm.clone());
                        }
                    }
                    Ok(())
                }
                _ => Err(RSpaceError::BugFoundError(
                    "BUG FOUND: only COMM events are expected here".to_string(),
                )),
            }?
        }

        Ok(())
    }

    fn check_replay_data(
        &self,
        replay_data: MultisetMultiMap<IOEvent, COMM>,
    ) -> Result<(), RSpaceError> {
        if replay_data.is_empty() {
            Ok(())
        } else {
            Err(RSpaceError::BugFoundError(format!(
                "Unused COMM event: replayData multimap has {} elements left",
                replay_data.map.len()
            )))
        }
    }
}
