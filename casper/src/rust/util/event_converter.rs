// See casper/src/main/scala/coop/rchain/casper/util/EventConverter.scala

use models::rust::casper::protocol::casper_message::CommEvent;
use models::rust::casper::protocol::casper_message::ConsumeEvent;
use models::rust::casper::protocol::casper_message::Event;
use models::rust::casper::protocol::casper_message::Peek;
use models::rust::casper::protocol::casper_message::ProduceEvent;
use rspace_plus_plus::rspace::trace::event::Event as RspaceEvent;
use rspace_plus_plus::rspace::trace::event::IOEvent;
use rspace_plus_plus::rspace::trace::event::COMM as RspaceComm;

pub fn to_casper_event(event: RspaceEvent) -> Event {
    match event {
        RspaceEvent::Comm(RspaceComm {
            consume,
            produces,
            peeks,
            times_repeated,
        }) => Event::Comm(CommEvent {
            consume: ConsumeEvent {
                channels_hashes: consume
                    .channel_hashes
                    .iter()
                    .map(|h| h.to_bytes_prost())
                    .collect(),
                hash: consume.hash.to_bytes_prost(),
                persistent: consume.persistent,
            },
            produces: produces
                .into_iter()
                .map(|p| ProduceEvent {
                    channels_hash: p.channel_hash.to_bytes_prost(),
                    hash: p.hash.to_bytes_prost(),
                    persistent: p.persistent,
                    times_repeated: *times_repeated.get(&p).unwrap_or(&0),
                    is_deterministic: p.is_deterministic,
                    output_value: p.output_value.into_iter().map(|v| v.into()).collect(),
                })
                .collect(),
            peeks: peeks.iter().map(|p| Peek { channel_index: *p }).collect(),
        }),

        RspaceEvent::IoEvent(ioevent) => match ioevent {
            IOEvent::Produce(produce) => Event::Produce(ProduceEvent {
                channels_hash: produce.channel_hash.to_bytes_prost(),
                hash: produce.hash.to_bytes_prost(),
                persistent: produce.persistent,
                times_repeated: 0,
                is_deterministic: produce.is_deterministic,
                output_value: produce.output_value.into_iter().map(|v| v.into()).collect(),
            }),

            IOEvent::Consume(consume) => Event::Consume(ConsumeEvent {
                channels_hashes: consume
                    .channel_hashes
                    .iter()
                    .map(|h| h.to_bytes_prost())
                    .collect(),
                hash: consume.hash.to_bytes_prost(),
                persistent: consume.persistent,
            }),
        },
    }
}
