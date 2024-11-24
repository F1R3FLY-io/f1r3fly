pub mod rust {

    pub mod interpreter;
}

use std::sync::{Arc, Mutex};

use crypto::rust::{hash::blake2b512_random::Blake2b512Random, public_key::PublicKey};
use models::rspace_plus_plus_types::*;
use models::{
    rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation},
    rholang_scala_rust_types::*,
};
use prost::Message;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::{
    rspace::RSpace,
    trace::event::{Event, IOEvent},
};
use rust::interpreter::{
    accounting::costs::Cost,
    rho_runtime::{
        bootstrap_registry as bootstrap_registry_internal, create_rho_runtime,
        RhoRuntime as RhoRuntimeTrait, RhoRuntimeImpl,
    },
    system_processes::BlockData,
};

#[repr(C)]
struct RhoRuntime {
    runtime: Arc<Mutex<RhoRuntimeImpl>>,
}

#[repr(C)]
struct Space {
    rspace: Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>,
}

#[no_mangle]
extern "C" fn get_hot_changes(runtime_ptr: *mut RhoRuntime) -> *const u8 {
    let runtime = unsafe { (*runtime_ptr).runtime.clone() };
    let hot_store_mapped = runtime.try_lock().unwrap().get_hot_changes();

    let mut map_entries: Vec<StoreToMapEntry> = Vec::new();

    for (key, value) in hot_store_mapped {
        let datums = value
            .data
            .into_iter()
            .map(|datum| DatumProto {
                a: Some(datum.a),
                persist: datum.persist,
                source: Some(ProduceProto {
                    channel_hash: datum.source.channel_hash.bytes(),
                    hash: datum.source.hash.bytes(),
                    persistent: datum.source.persistent,
                }),
            })
            .collect();

        let wks = value
            .wks
            .into_iter()
            .map(|wk| {
                let res = WaitingContinuationProto {
                    patterns: wk.patterns,
                    continuation: Some(wk.continuation.clone()),
                    persist: wk.persist,
                    peeks: wk
                        .peeks
                        .into_iter()
                        .map(|peek| SortedSetElement { value: peek as i32 })
                        .collect(),
                    source: Some(ConsumeProto {
                        channel_hashes: wk
                            .source
                            .channel_hashes
                            .iter()
                            .map(|hash| hash.bytes())
                            .collect(),
                        hash: wk.source.hash.bytes(),
                        persistent: wk.source.persistent,
                    }),
                };

                res
            })
            .collect();

        let value = StoreToMapValue { data: datums, wks };
        map_entries.push(StoreToMapEntry {
            key,
            value: Some(value),
        });
    }

    let to_map_result = StoreToMapResult { map_entries };

    let mut bytes = to_map_result.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
extern "C" fn evaluate(
    runtime_ptr: *mut RhoRuntime,
    params_ptr: *const u8,
    params_bytes_len: usize,
) -> *const u8 {
    // println!("\nhit rust lib evaluate");

    let params_slice = unsafe { std::slice::from_raw_parts(params_ptr, params_bytes_len) };
    let params = EvaluateParams::decode(params_slice).unwrap();

    let term = params.term;
    let cost_proto = params.initial_phlo.unwrap();
    let initial_phlo = Cost::create(cost_proto.value.into(), cost_proto.operation);
    let normalizer_env = params.normalizer_env;
    let rand = Blake2b512Random::new(&params.rand);
    // println!("\nrand in rust lib: {:?}", rand.to_vec());

    let mut rho_runtime = unsafe { (*runtime_ptr).runtime.try_lock().unwrap() };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let eval_result = rt.block_on(async {
        rho_runtime
            .evaluate(
                term,
                initial_phlo,
                normalizer_env.into_iter().collect(),
                rand,
            )
            .await
            .unwrap()
    });

    // println!("\neval_result: {:?}", eval_result);

    let eval_result_proto = EvaluateResultProto {
        cost: Some(CostProto {
            value: eval_result.cost.value as i32,
            operation: eval_result.cost.operation,
        }),
        errors: eval_result
            .errors
            .into_iter()
            .map(|err| err.to_string())
            .collect(),
        mergeable: eval_result.mergeable.into_iter().collect(),
    };

    let mut bytes = eval_result_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
extern "C" fn create_checkpoint(runtime_ptr: *mut RhoRuntime) -> *const u8 {
    let runtime = unsafe { (*runtime_ptr).runtime.clone() };
    let checkpoint = runtime.try_lock().unwrap().create_checkpoint();

    let log = checkpoint.log;
    let log_proto: Vec<EventProto> = log
        .into_iter()
        .map(|event| match event {
            Event::Comm(comm) => {
                let comm_proto = CommProto {
                    consume: {
                        Some(ConsumeProto {
                            channel_hashes: comm
                                .consume
                                .channel_hashes
                                .iter()
                                .map(|hash| hash.bytes())
                                .collect(),
                            hash: comm.consume.hash.bytes(),
                            persistent: comm.consume.persistent,
                        })
                    },
                    produces: {
                        comm.produces
                            .into_iter()
                            .map(|produce| ProduceProto {
                                channel_hash: produce.channel_hash.bytes(),
                                hash: produce.hash.bytes(),
                                persistent: produce.persistent,
                            })
                            .collect()
                    },
                    peeks: {
                        comm.peeks
                            .into_iter()
                            .map(|peek| SortedSetElement { value: peek as i32 })
                            .collect()
                    },
                    times_repeated: {
                        let mut produce_counter_map_entries: Vec<ProduceCounterMapEntry> =
                            Vec::new();
                        for (key, value) in comm.times_repeated {
                            let produce = ProduceProto {
                                channel_hash: key.channel_hash.bytes(),
                                hash: key.hash.bytes(),
                                persistent: key.persistent,
                            };

                            produce_counter_map_entries.push(ProduceCounterMapEntry {
                                key: Some(produce),
                                value,
                            });
                        }
                        produce_counter_map_entries
                    },
                };

                EventProto {
                    event_type: Some(event_proto::EventType::Comm(comm_proto)),
                }
            }
            Event::IoEvent(io_event) => match io_event {
                IOEvent::Produce(produce) => {
                    let produce_proto = ProduceProto {
                        channel_hash: produce.channel_hash.bytes(),
                        hash: produce.hash.bytes(),
                        persistent: produce.persistent,
                    };
                    EventProto {
                        event_type: Some(event_proto::EventType::IoEvent(IoEventProto {
                            io_event_type: Some(io_event_proto::IoEventType::Produce(
                                produce_proto,
                            )),
                        })),
                    }
                }
                IOEvent::Consume(consume) => {
                    let consume_proto = ConsumeProto {
                        channel_hashes: consume
                            .channel_hashes
                            .iter()
                            .map(|hash| hash.bytes())
                            .collect(),
                        hash: consume.hash.bytes(),
                        persistent: consume.persistent,
                    };
                    EventProto {
                        event_type: Some(event_proto::EventType::IoEvent(IoEventProto {
                            io_event_type: Some(io_event_proto::IoEventType::Consume(
                                consume_proto,
                            )),
                        })),
                    }
                }
            },
        })
        .collect();

    let checkpoint_proto = CheckpointProto {
        root: checkpoint.root.bytes(),
        log: log_proto,
    };

    let mut bytes = checkpoint_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
extern "C" fn reset(
    runtime_ptr: *mut RhoRuntime,
    root_pointer: *const u8,
    root_bytes_len: usize,
) -> () {
    // println!("\nHit reset");

    let root_slice = unsafe { std::slice::from_raw_parts(root_pointer, root_bytes_len) };
    let root = Blake2b256Hash::from_bytes(root_slice.to_vec());

    let runtime = unsafe { (*runtime_ptr).runtime.clone() };
    runtime.try_lock().unwrap().reset(root);
}

#[no_mangle]
extern "C" fn set_block_data(
    runtime_ptr: *mut RhoRuntime,
    params_ptr: *const u8,
    params_bytes_len: usize,
) -> () {
    let params_slice = unsafe { std::slice::from_raw_parts(params_ptr, params_bytes_len) };
    let params = BlockDataProto::decode(params_slice).unwrap();
    let block_data = BlockData {
        time_stamp: params.time_stamp as i64,
        block_number: params.block_number as i64,
        sender: PublicKey::from_bytes(&params.public_key),
        seq_num: params.seq_num,
    };

    unsafe {
        (*runtime_ptr)
            .runtime
            .try_lock()
            .unwrap()
            .set_block_data(block_data);
    }
}

#[no_mangle]
extern "C" fn bootstrap_registry(runtime_ptr: *mut RhoRuntime) -> () {
    let runtime = unsafe { (*runtime_ptr).runtime.clone() };
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    tokio_runtime.block_on(async {
        bootstrap_registry_internal(runtime).await;
    });
}

// Note: I am defaulting 'additional_system_processes' to 'Vec::new()'
#[no_mangle]
extern "C" fn create_runtime(
    rspace_ptr: *mut Space,
    params_ptr: *const u8,
    params_bytes_len: usize,
) -> *mut RhoRuntime {
    let rspace = unsafe { (*rspace_ptr).rspace.try_lock().unwrap().clone() };

    let params_slice = unsafe { std::slice::from_raw_parts(params_ptr, params_bytes_len) };
    let params = CreateRuntimeParams::decode(params_slice).unwrap();

    let mergeable_tag_name = params.mergeable_tag_name.unwrap();
    let init_registry = params.init_registry;

    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let rho_runtime = tokio_runtime.block_on(async {
        create_rho_runtime(rspace, mergeable_tag_name, init_registry, &mut Vec::new()).await
    });

    Box::into_raw(Box::new(RhoRuntime {
        runtime: rho_runtime,
    }))
}
