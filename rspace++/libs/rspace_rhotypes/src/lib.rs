use prost::Message;
use rspace_plus_plus::rspace::hashing::blake3_hash::Blake3Hash;
use rspace_plus_plus::rspace::matcher::exports::*;
use rspace_plus_plus::rspace::matcher::r#match::Matcher;
use rspace_plus_plus::rspace::matcher::spatial_matcher::SpatialMatcherContext;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceInstances};
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use rspace_plus_plus::rspace::shared::lmdb_dir_store_manager::GB;
use rspace_plus_plus::rspace::shared::rspace_store_manager::mk_rspace_store_manager;
use rspace_plus_plus::rspace_plus_plus_types::rspace_plus_plus_types::{
    Channels, CheckpointProto, Datums, Join, Joins, WaitingContinuations,
};

/*
 * This library contains predefined types for Channel, Pattern, Data, and Continuation - RhoTypes
 * These types (C, P, A, K) MUST MATCH the corresponding types on the Scala side in 'RSpacePlusPlus_RhoTypes.scala'
 */
#[repr(C)]
pub struct Space {
    rspace: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation, Matcher>,
}

#[no_mangle]
pub extern "C" fn space_new() -> *mut Space {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let rspace = rt
        .block_on(async {
            let mut kvm = mk_rspace_store_manager("../rspace++/lmdb".into(), 1 * GB);
            let store = kvm.r_space_stores().await.unwrap();

            RSpaceInstances::create(store, Matcher).await
        })
        .unwrap();

    Box::into_raw(Box::new(Space { rspace }))
}

#[no_mangle]
pub extern "C" fn space_print(rspace: *mut Space) -> () {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { unsafe { (*rspace).rspace.store.print().await } })
}

#[no_mangle]
pub extern "C" fn space_clear(rspace: *mut Space) -> () {
    unsafe {
        (*rspace).rspace.clear();
    }
}

#[no_mangle]
pub extern "C" fn spatial_match_result(
    payload_pointer: *const u8,
    target_bytes_len: usize,
    pattern_bytes_len: usize,
) -> *const u8 {
    let payload_slice = unsafe {
        std::slice::from_raw_parts(payload_pointer, target_bytes_len + pattern_bytes_len)
    };
    let (target_slice, pattern_slice) = payload_slice.split_at(target_bytes_len);

    let target = Par::decode(target_slice).unwrap();
    let pattern = Par::decode(pattern_slice).unwrap();

    let mut spatial_matcher = SpatialMatcherContext::new();
    let result_option = spatial_matcher.spatial_match_result(target, pattern);

    match result_option {
        Some(free_map) => {
            let free_map_proto = FreeMapProto {
                entries: free_map.clone(),
            };

            let mut bytes = free_map_proto.encode_to_vec();
            let len = bytes.len() as u32;
            let len_bytes = len.to_le_bytes().to_vec();
            let mut result = len_bytes;
            result.append(&mut bytes);
            Box::leak(result.into_boxed_slice()).as_ptr()
        }
        None => std::ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn produce(
    rspace: *mut Space,
    payload_pointer: *const u8,
    channel_bytes_len: usize,
    data_bytes_len: usize,
    persist: bool,
) -> *const u8 {
    let payload_slice =
        unsafe { std::slice::from_raw_parts(payload_pointer, channel_bytes_len + data_bytes_len) };
    let (channel_slice, data_slice) = payload_slice.split_at(channel_bytes_len);

    let channel = Par::decode(channel_slice).unwrap();
    let data = ListParWithRandom::decode(data_slice).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result_option = unsafe { (*rspace).rspace.produce(channel, data, persist).await };

        match result_option {
            Some((cont_result, rspace_results)) => {
                let protobuf_cont_result = ContResultProto {
                    continuation: Some(cont_result.continuation),
                    persistent: cont_result.persistent,
                    channels: cont_result.channels,
                    patterns: cont_result.patterns,
                    peek: cont_result.peek,
                };
                let protobuf_results = rspace_results
                    .into_iter()
                    .map(|result| RSpaceResultProto {
                        channel: Some(result.channel),
                        matched_datum: Some(result.matched_datum),
                        removed_datum: Some(result.removed_datum),
                        persistent: result.persistent,
                    })
                    .collect();

                let maybe_action_result = ActionResult {
                    cont_result: Some(protobuf_cont_result),
                    results: protobuf_results,
                };

                let mut bytes = maybe_action_result.encode_to_vec();
                let len = bytes.len() as u32;
                let len_bytes = len.to_le_bytes().to_vec();
                let mut result = len_bytes;
                result.append(&mut bytes);
                Box::leak(result.into_boxed_slice()).as_ptr()
            }
            None => std::ptr::null(),
        }
    })
}

#[no_mangle]
pub extern "C" fn consume(
    rspace: *mut Space,
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> *const u8 {
    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let consume_params = ConsumeParams::decode(payload_slice).unwrap();

    let channels = consume_params.channels;
    let patterns = consume_params.patterns;
    let continuation = consume_params.continuation.unwrap();
    let persist = consume_params.persist;
    let peeks = consume_params.peeks.into_iter().map(|e| e.value).collect();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result_option = unsafe {
            (*rspace)
                .rspace
                .consume(channels, patterns, continuation, persist, peeks)
                .await
        };

        match result_option {
            Some((cont_result, rspace_results)) => {
                let protobuf_cont_result = ContResultProto {
                    continuation: Some(cont_result.continuation),
                    persistent: cont_result.persistent,
                    channels: cont_result.channels,
                    patterns: cont_result.patterns,
                    peek: cont_result.peek,
                };
                let protobuf_results = rspace_results
                    .into_iter()
                    .map(|result| RSpaceResultProto {
                        channel: Some(result.channel),
                        matched_datum: Some(result.matched_datum),
                        removed_datum: Some(result.removed_datum),
                        persistent: result.persistent,
                    })
                    .collect();

                let maybe_action_result = ActionResult {
                    cont_result: Some(protobuf_cont_result),
                    results: protobuf_results,
                };

                let mut bytes = maybe_action_result.encode_to_vec();
                let len = bytes.len() as u32;
                let len_bytes = len.to_le_bytes().to_vec();
                let mut result = len_bytes;
                result.append(&mut bytes);
                Box::leak(result.into_boxed_slice()).as_ptr()
            }
            None => std::ptr::null(),
        }
    })
}

#[no_mangle]
pub extern "C" fn install(
    rspace: *mut Space,
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> *const u8 {
    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let consume_params = InstallParams::decode(payload_slice).unwrap();

    let channels = consume_params.channels;
    let patterns = consume_params.patterns;
    let continuation = consume_params.continuation.unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result_option = unsafe {
            (*rspace)
                .rspace
                .install(channels, patterns, continuation)
                .await
        };

        match result_option {
            None => std::ptr::null(),
            Some(_) => {
                panic!("RUST ERROR: Installing can be done only on startup")
            }
        }
    })
}

#[no_mangle]
pub extern "C" fn create_checkpoint(rspace: *mut Space) -> *const u8 {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let checkpoint = unsafe { (*rspace).rspace.create_checkpoint().await };

        let checkpoint_proto = CheckpointProto {
            root: checkpoint.root.bytes(),
        };

        let mut bytes = checkpoint_proto.encode_to_vec();
        let len = bytes.len() as u32;
        let len_bytes = len.to_le_bytes().to_vec();
        let mut result = len_bytes;
        result.append(&mut bytes);
        Box::leak(result.into_boxed_slice()).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn reset(rspace: *mut Space, root_pointer: *const u8, root_bytes_len: usize) -> () {
    let root_slice = unsafe { std::slice::from_raw_parts(root_pointer, root_bytes_len) };
    let root = Blake3Hash::new(root_slice);

    let _ = unsafe { (*rspace).rspace.reset(root) };
}

#[no_mangle]
pub extern "C" fn get_data(
    rspace: *mut Space,
    channel_pointer: *const u8,
    channel_bytes_len: usize,
) -> *const u8 {
    let channel_slice = unsafe { std::slice::from_raw_parts(channel_pointer, channel_bytes_len) };
    let channel = Par::decode(channel_slice).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let datums = unsafe { (*rspace).rspace.get_data(channel).await };

        let datums_protos: Vec<DatumProto> = datums
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

        let datums_proto = Datums {
            datums: datums_protos,
        };

        let mut bytes = datums_proto.encode_to_vec();
        let len = bytes.len() as u32;
        let len_bytes = len.to_le_bytes().to_vec();
        let mut result = len_bytes;
        result.append(&mut bytes);
        Box::leak(result.into_boxed_slice()).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn get_waiting_continuations(
    rspace: *mut Space,
    channels_pointer: *const u8,
    channels_bytes_len: usize,
) -> *const u8 {
    let channels_slice =
        unsafe { std::slice::from_raw_parts(channels_pointer, channels_bytes_len) };
    let channels_proto = Channels::decode(channels_slice).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let wks = unsafe {
            (*rspace)
                .rspace
                .get_waiting_continuations(channels_proto.channels)
                .await
        };

        let wks_protos: Vec<WaitingContinuationProto> = wks
            .into_iter()
            .map(|wk| WaitingContinuationProto {
                patterns: wk.patterns,
                continuation: Some(wk.continuation),
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
            })
            .collect();

        let wks_proto = WaitingContinuations { wks: wks_protos };

        let mut bytes = wks_proto.encode_to_vec();
        let len = bytes.len() as u32;
        let len_bytes = len.to_le_bytes().to_vec();
        let mut result = len_bytes;
        result.append(&mut bytes);
        Box::leak(result.into_boxed_slice()).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn get_joins(
    rspace: *mut Space,
    channel_pointer: *const u8,
    channel_bytes_len: usize,
) -> *const u8 {
    let channel_slice = unsafe { std::slice::from_raw_parts(channel_pointer, channel_bytes_len) };
    let channel = Par::decode(channel_slice).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let joins = unsafe { (*rspace).rspace.get_joins(channel).await };
        let vec_join: Vec<Join> = joins.into_iter().map(|join| Join { join }).collect();
        let joins_proto = Joins { joins: vec_join };

        let mut bytes = joins_proto.encode_to_vec();
        let len = bytes.len() as u32;
        let len_bytes = len.to_le_bytes().to_vec();
        let mut result = len_bytes;
        result.append(&mut bytes);
        Box::leak(result.into_boxed_slice()).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn to_map(rspace: *mut Space) -> *const u8 {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let hot_store_mapped = unsafe { (*rspace).rspace.store.to_map().await };
        let mut map_entries: Vec<MapEntry> = Vec::new();

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
                .map(|wk| WaitingContinuationProto {
                    patterns: wk.patterns,
                    continuation: Some(wk.continuation),
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
                })
                .collect();

            let row = ProtobufRow { data: datums, wks };
            map_entries.push(MapEntry {
                key,
                value: Some(row),
            });
        }

        let to_map_result = ToMapResult { map_entries };

        let mut bytes = to_map_result.encode_to_vec();
        let len = bytes.len() as u32;
        let len_bytes = len.to_le_bytes().to_vec();
        let mut result = len_bytes;
        result.append(&mut bytes);
        Box::leak(result.into_boxed_slice()).as_ptr()
    })
}

#[no_mangle]
pub extern "C" fn spawn(rspace: *mut Space) -> *mut Space {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let rspace = rt.block_on(async { unsafe { (*rspace).rspace.spawn().await } });

    Box::into_raw(Box::new(Space { rspace }))
}

// #[no_mangle]
// pub extern "C" fn create_soft_checkpoint(rspace: *mut Space) -> *const u8 {
//     let rt = tokio::runtime::Runtime::new().unwrap();
//     let soft_checkpoint =
//         rt.block_on(async { unsafe { (*rspace).rspace.create_soft_checkpoint().await } });

//     Box::into_raw(Box::new(Space { rspace }))
// }

#[no_mangle]
pub extern "C" fn deallocate_memory(ptr: *mut u8, len: usize) {
    // SAFETY: The caller must guarantee that `ptr` is a valid pointer to a memory block
    // allocated by Rust, and that `len` is the correct size of the block.
    unsafe {
        // Convert the raw pointer back to a Box to allow Rust to deallocate the memory.
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        // The Box goes out of scope here, and Rust automatically deallocates the memory.
    }
}
