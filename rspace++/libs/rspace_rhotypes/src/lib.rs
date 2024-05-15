use dashmap::DashMap;
use prost::Message;
use rand::Rng;
use rspace_plus_plus::rspace::checkpoint::SoftCheckpoint;
use rspace_plus_plus::rspace::event::{Consume, Produce};
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use rspace_plus_plus::rspace::hot_store::HotStoreState;
use rspace_plus_plus::rspace::internal::{Datum, WaitingContinuation};
use rspace_plus_plus::rspace::matcher::exports::*;
use rspace_plus_plus::rspace::matcher::r#match::Matcher;
use rspace_plus_plus::rspace::matcher::spatial_matcher::SpatialMatcherContext;
use rspace_plus_plus::rspace::rspace::{RSpace, RSpaceInstances};
use rspace_plus_plus::rspace::shared::key_value_store_manager::KeyValueStoreManager;
use rspace_plus_plus::rspace::shared::lmdb_dir_store_manager::GB;
use rspace_plus_plus::rspace::shared::rspace_store_manager::{
    get_or_create_rspace_store, mk_rspace_store_manager,
};
use rspace_plus_plus::rspace_plus_plus_types::rspace_plus_plus_types::{
    ChannelsProto, CheckpointProto, DatumsProto, HotStoreStateProto, JoinProto, JoinsProto,
    ProduceCounterMapEntry, SoftCheckpointProto, StoreStateContMapEntry, StoreStateDataMapEntry,
    StoreStateInstalledContMapEntry, StoreStateInstalledJoinsMapEntry, StoreStateJoinsMapEntry,
    StoreToMapValue, WaitingContinuationsProto,
};
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::Mutex;

/*
 * This library contains predefined types for Channel, Pattern, Data, and Continuation - RhoTypes
 * These types (C, P, A, K) MUST MATCH the corresponding types on the Scala side in 'RSpacePlusPlus_RhoTypes.scala'
 */
#[repr(C)]
pub struct Space {
    rspace: Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation, Matcher>>,
}

#[no_mangle]
pub extern "C" fn space_new(path: *const c_char) -> *mut Space {
    let c_str = unsafe { CStr::from_ptr(path) };
    let data_dir = c_str.to_str().unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let rspace = rt
        .block_on(async {
            println!("\nHit space_new");

            // let mut kvm = mk_rspace_store_manager(lmdb_path.into(), 1 * GB);
            // let store = kvm.r_space_stores().await.unwrap();

            let mut rng = rand::thread_rng();
            let unique_id: u32 = rng.gen();
            let mut kvm =
                mk_rspace_store_manager((&format!("{}/rspace++/", data_dir)).into(), 1 * GB);
            let store = kvm.r_space_stores().await.unwrap();

            // println!("\nhistory store: {:?}", store.history.lock().unwrap().to_map().unwrap());
            // println!("\nroots store: {:?}", store.roots.lock().unwrap().to_map().unwrap());
            // roots_store.print_store();
            // println!("\ncold store: {:?}", store.cold.lock().unwrap().to_map().unwrap().len());
            // cold_store.print_store();

            // let store =
            //     get_or_create_rspace_store(&format!("{}/rspace++_{}/", data_dir, unique_id), 1 * GB)
            //         .expect("Error getting RSpaceStore: ");
            // let store = get_or_create_rspace_store(&format!("{}/rspace++/", data_dir), 1 * GB)
            //     .expect("Error getting RSpaceStore: ");

            RSpaceInstances::create(store, Matcher)
        })
        .unwrap();

    // let store = get_or_create_rspace_store(&format!("{}/rspace++/", data_dir), 1 * GB)
    //     .expect("Error getting RSpaceStore: ");

    // let rspace = RSpaceInstances::create(store, Matcher).unwrap();

    Box::into_raw(Box::new(Space {
        rspace: Mutex::new(rspace),
    }))
}

#[no_mangle]
pub extern "C" fn space_print(rspace: *mut Space) -> () {
    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    rspace_lock.store.print()
}

#[no_mangle]
pub extern "C" fn space_clear(rspace: *mut Space) -> () {
    let mut rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };

    rspace_lock
        .clear()
        .expect("Rust RSpacePlusPlus Library: Failed to clear")
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
    // println!("\nHit produce");

    let payload_slice =
        unsafe { std::slice::from_raw_parts(payload_pointer, channel_bytes_len + data_bytes_len) };
    let (channel_slice, data_slice) = payload_slice.split_at(channel_bytes_len);

    let channel = Par::decode(channel_slice).unwrap();
    let data = ListParWithRandom::decode(data_slice).unwrap();

    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let result_option = rspace_lock.produce(channel, data, persist);

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
}

#[no_mangle]
pub extern "C" fn consume(
    rspace: *mut Space,
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> *const u8 {
    // println!("\nHit consume in rust");

    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let consume_params = ConsumeParams::decode(payload_slice).unwrap();

    let channels = consume_params.channels;
    let patterns = consume_params.patterns;
    let continuation = consume_params.continuation.unwrap();
    let persist = consume_params.persist;
    let peeks = consume_params.peeks.into_iter().map(|e| e.value).collect();

    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let result_option = rspace_lock.consume(channels, patterns, continuation, persist, peeks);

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
                .map(|result| {
                    let r = RSpaceResultProto {
                        channel: Some(result.channel),
                        matched_datum: Some(result.matched_datum),
                        removed_datum: Some(result.removed_datum),
                        persistent: result.persistent,
                    };
                    r
                })
                .collect();

            let maybe_action_result = ActionResult {
                cont_result: Some(protobuf_cont_result),
                results: protobuf_results,
            };

            // println!("\nmaybe_action_result: {:?}", maybe_action_result);

            // let bytes_test = bincode::serialize(&maybe_action_result).unwrap();
            // println!("\ndecoded: {:?}", bincode::deserialize::<ActionResult>(&bytes_test).is_ok());

            let bytes_test = maybe_action_result.encode_length_delimited_to_vec();
            // Read the length prefix from the encoded bytes
            let mut length_bytes = &bytes_test[..];
            let length_prefix = match prost::encoding::decode_varint(&mut length_bytes) {
                Ok(len) => len,
                Err(e) => {
                    panic!("\nError decoding length prefix: {:?}", e);
                }
            };
            // Compare the length prefix to the length of the remaining bytes
            let remaining_bytes = length_bytes.len();
            if length_prefix as usize != remaining_bytes {
                eprintln!("\nMismatch between length prefix and actual data length. Prefix: {}, Actual: {}", length_prefix, remaining_bytes);
            }

            match ActionResult::decode_length_delimited(&*bytes_test) {
                Ok(decoded) => {
                    println!("\nSucessfully decoded")
                }
                Err(e) => eprintln!("\nError decoding ActionResult: {:?}", e),
            }

            let mut bytes = maybe_action_result.encode_to_vec();
            // println!("\ndecoded actual bytes: {:?}", ActionResult::decode(&*bytes));
            let len = bytes.len() as u32;
            let len_bytes = len.to_le_bytes().to_vec();
            let mut result = len_bytes;
            result.append(&mut bytes);

            // println!("\nlen: {:?}", len);

            Box::leak(result.into_boxed_slice()).as_ptr()
        }
        None => {
            // println!("\nnone in rust consume");
            std::ptr::null()
        }
    }
}

// #[no_mangle]
// pub extern "C" fn consume(
//     rspace: *mut Space,
//     payload_pointer: *const u8,
//     payload_bytes_len: usize,
// ) -> *mut c_void {
//     println!("\nHit consume in rust");

//     let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
//     let consume_params = ConsumeParams::decode(payload_slice).unwrap();

//     let channels = consume_params.channels;
//     let patterns = consume_params.patterns;
//     let continuation = consume_params.continuation.unwrap();
//     let persist = consume_params.persist;
//     let peeks = consume_params.peeks.into_iter().map(|e| e.value).collect();

//     let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
//     let result_option = rspace_lock.consume(channels, patterns, continuation, persist, peeks);

//     match result_option {
//         Some((cont_result, rspace_results)) => {
//             let protobuf_cont_result = ContResultProto {
//                 continuation: Some(cont_result.continuation),
//                 persistent: cont_result.persistent,
//                 channels: cont_result.channels,
//                 patterns: cont_result.patterns,
//                 peek: cont_result.peek,
//             };
//             let protobuf_results = rspace_results
//                 .into_iter()
//                 .map(|result| {
//                     let r = RSpaceResultProto {
//                         channel: Some(result.channel),
//                         matched_datum: Some(result.matched_datum),
//                         removed_datum: Some(result.removed_datum),
//                         persistent: result.persistent,
//                     };
//                     r
//                 })
//                 .collect();

//             let maybe_action_result = ActionResult {
//                 cont_result: Some(protobuf_cont_result),
//                 results: protobuf_results,
//             };

//             let bytes_test = serde_json::to_string(&maybe_action_result).unwrap();
//             println!("\nbytes_test: {:?}", bytes_test);
//             // println!("\ndecoded: {:?}", serde_json::from_str::<ActionResult>(&bytes_test));

//             let c_str = CString::new(bytes_test).unwrap();
//             c_str.into_raw() as *mut c_void
//         }
//         None => {
//             let c_str = CString::new("").unwrap();
//             c_str.into_raw() as *mut c_void
//         }
//     }
// }

#[no_mangle]
pub extern "C" fn install(
    rspace: *mut Space,
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> *const u8 {
    // println!("\nHit install");

    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let consume_params = InstallParams::decode(payload_slice).unwrap();

    let channels = consume_params.channels;
    let patterns = consume_params.patterns;
    let continuation = consume_params.continuation.unwrap();

    let mut rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let result_option = rspace_lock.install(channels, patterns, continuation);

    match result_option {
        None => std::ptr::null(),
        Some(_) => {
            panic!("RUST ERROR: Installing can be done only on startup")
        }
    }
}

#[no_mangle]
pub extern "C" fn create_checkpoint(rspace: *mut Space) -> *const u8 {
    // println!("\nHit create_checkpoint");

    let mut rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let checkpoint = rspace_lock
        .create_checkpoint()
        .expect("Rust RSpacePlusPlus Library: Failed to create checkpoint");

    // println!("create_checkpoint root hash: {}", checkpoint.root);

    let checkpoint_proto = CheckpointProto {
        root: checkpoint.root.bytes(),
    };

    let mut bytes = checkpoint_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);

    // println!("\nAAA. after: {:?}", ActionResult::decode(&*bytes).unwrap());

    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
pub extern "C" fn reset(rspace: *mut Space, root_pointer: *const u8, root_bytes_len: usize) -> () {
    // println!("\nHit reset");

    let root_slice = unsafe { std::slice::from_raw_parts(root_pointer, root_bytes_len) };
    let root = Blake2b256Hash::from_bytes(root_slice.to_vec());

    let mut rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    rspace_lock
        .reset(root)
        .expect("Rust RSpacePlusPlus Library: Failed to reset")
}

#[no_mangle]
pub extern "C" fn get_data(
    rspace: *mut Space,
    channel_pointer: *const u8,
    channel_bytes_len: usize,
) -> *const u8 {
    let channel_slice = unsafe { std::slice::from_raw_parts(channel_pointer, channel_bytes_len) };
    let channel = Par::decode(channel_slice).unwrap();

    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let datums = rspace_lock.get_data(channel);

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

    let datums_proto = DatumsProto {
        datums: datums_protos,
    };

    let mut bytes = datums_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
pub extern "C" fn get_waiting_continuations(
    rspace: *mut Space,
    channels_pointer: *const u8,
    channels_bytes_len: usize,
) -> *const u8 {
    let channels_slice =
        unsafe { std::slice::from_raw_parts(channels_pointer, channels_bytes_len) };
    let channels_proto = ChannelsProto::decode(channels_slice).unwrap();

    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let wks = rspace_lock.get_waiting_continuations(channels_proto.channels);

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

    let wks_proto = WaitingContinuationsProto { wks: wks_protos };

    let mut bytes = wks_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
pub extern "C" fn get_joins(
    rspace: *mut Space,
    channel_pointer: *const u8,
    channel_bytes_len: usize,
) -> *const u8 {
    let channel_slice = unsafe { std::slice::from_raw_parts(channel_pointer, channel_bytes_len) };
    let channel = Par::decode(channel_slice).unwrap();

    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let joins = rspace_lock.get_joins(channel);
    let vec_join: Vec<JoinProto> = joins.into_iter().map(|join| JoinProto { join }).collect();
    let joins_proto = JoinsProto { joins: vec_join };

    let mut bytes = joins_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
pub extern "C" fn to_map(rspace: *mut Space) -> *const u8 {
    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let hot_store_mapped = rspace_lock.store.to_map();
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
pub extern "C" fn spawn(rspace: *mut Space) -> *mut Space {
    let rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let rspace = rspace_lock
        .spawn()
        .expect("Rust RSpacePlusPlus Library: Failed to spawn");

    Box::into_raw(Box::new(Space {
        rspace: Mutex::new(rspace),
    }))
}

#[no_mangle]
pub extern "C" fn create_soft_checkpoint(rspace: *mut Space) -> *const u8 {
    let mut rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };
    let soft_checkpoint = rspace_lock.create_soft_checkpoint();

    let mut conts_map_entries: Vec<StoreStateContMapEntry> = Vec::new();
    let mut installed_conts_map_entries: Vec<StoreStateInstalledContMapEntry> = Vec::new();
    let mut data_map_entries: Vec<StoreStateDataMapEntry> = Vec::new();
    let mut joins_map_entries: Vec<StoreStateJoinsMapEntry> = Vec::new();
    let mut installed_joins_map_entries: Vec<StoreStateInstalledJoinsMapEntry> = Vec::new();

    let hot_store_state = soft_checkpoint.cache_snapshot;

    for (key, value) in hot_store_state.continuations.clone().into_iter() {
        let wks: Vec<WaitingContinuationProto> = value
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

        conts_map_entries.push(StoreStateContMapEntry { key, value: wks });
    }

    for (key, value) in hot_store_state.installed_continuations.clone().into_iter() {
        let wk = WaitingContinuationProto {
            patterns: value.patterns,
            continuation: Some(value.continuation),
            persist: value.persist,
            peeks: value
                .peeks
                .into_iter()
                .map(|peek| SortedSetElement { value: peek as i32 })
                .collect(),
            source: Some(ConsumeProto {
                channel_hashes: value
                    .source
                    .channel_hashes
                    .iter()
                    .map(|hash| hash.bytes())
                    .collect(),
                hash: value.source.hash.bytes(),
                persistent: value.source.persistent,
            }),
        };

        installed_conts_map_entries.push(StoreStateInstalledContMapEntry {
            key,
            value: Some(wk),
        });
    }

    for (key, value) in hot_store_state.data.clone().into_iter() {
        let datums = value
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

        data_map_entries.push(StoreStateDataMapEntry {
            key: Some(key),
            value: datums,
        });
    }

    for (key, value) in hot_store_state.joins.clone().into_iter() {
        let joins = value.into_iter().map(|join| JoinProto { join }).collect();

        joins_map_entries.push(StoreStateJoinsMapEntry {
            key: Some(key),
            value: joins,
        });
    }

    for (key, value) in hot_store_state.installed_joins.clone().into_iter() {
        let joins = value.into_iter().map(|join| JoinProto { join }).collect();

        installed_joins_map_entries.push(StoreStateInstalledJoinsMapEntry {
            key: Some(key),
            value: joins,
        });
    }

    let hot_store_state_proto = HotStoreStateProto {
        continuations: conts_map_entries,
        installed_continuations: installed_conts_map_entries,
        data: data_map_entries,
        joins: joins_map_entries,
        installed_joins: installed_joins_map_entries,
    };

    let mut produce_counter_map_entries: Vec<ProduceCounterMapEntry> = Vec::new();
    let produce_counter_map = soft_checkpoint.produce_counter;

    for (key, value) in produce_counter_map {
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

    let soft_checkpoint_proto = SoftCheckpointProto {
        cache_snapshot: Some(hot_store_state_proto),
        produce_counter: produce_counter_map_entries,
    };

    let mut bytes = soft_checkpoint_proto.encode_to_vec();
    let len = bytes.len() as u32;
    let len_bytes = len.to_le_bytes().to_vec();
    let mut result = len_bytes;
    result.append(&mut bytes);
    Box::leak(result.into_boxed_slice()).as_ptr()
}

#[no_mangle]
pub extern "C" fn revert_to_soft_checkpoint(
    rspace: *mut Space,
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> () {
    // println!("\nHit revert_to_soft_checkpoint");

    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let soft_checkpoint_proto = SoftCheckpointProto::decode(payload_slice).unwrap();
    let cache_snapshot_proto = soft_checkpoint_proto.cache_snapshot.unwrap();

    let conts_map: DashMap<Vec<Par>, Vec<WaitingContinuation<BindPattern, TaggedContinuation>>> =
        cache_snapshot_proto
            .continuations
            .into_iter()
            .map(|map_entry| {
                let key = map_entry.key;
                let value = map_entry
                    .value
                    .into_iter()
                    .map(|cont_proto| WaitingContinuation {
                        patterns: cont_proto.patterns,
                        continuation: cont_proto.continuation.unwrap(),
                        persist: cont_proto.persist,
                        peeks: cont_proto
                            .peeks
                            .iter()
                            .map(|element| element.value)
                            .collect(),
                        source: {
                            let consume_proto = cont_proto.source.unwrap();
                            Consume {
                                channel_hashes: consume_proto
                                    .channel_hashes
                                    .iter()
                                    .map(|hash_bytes| {
                                        Blake2b256Hash::from_bytes(hash_bytes.to_vec())
                                    })
                                    .collect(),
                                hash: Blake2b256Hash::from_bytes(consume_proto.hash),
                                persistent: consume_proto.persistent,
                            }
                        },
                    })
                    .collect();

                (key, value)
            })
            .collect();

    let installed_conts_map: DashMap<
        Vec<Par>,
        WaitingContinuation<BindPattern, TaggedContinuation>,
    > = cache_snapshot_proto
        .installed_continuations
        .into_iter()
        .map(|map_entry| {
            let key = map_entry.key;
            let wk_proto = map_entry.value.unwrap();
            let value = WaitingContinuation {
                patterns: wk_proto.patterns,
                continuation: wk_proto.continuation.unwrap(),
                persist: wk_proto.persist,
                peeks: wk_proto.peeks.iter().map(|element| element.value).collect(),
                source: {
                    let consume_proto = wk_proto.source.unwrap();
                    Consume {
                        channel_hashes: consume_proto
                            .channel_hashes
                            .iter()
                            .map(|hash_bytes| Blake2b256Hash::from_bytes(hash_bytes.to_vec()))
                            .collect(),
                        hash: Blake2b256Hash::from_bytes(consume_proto.hash),
                        persistent: consume_proto.persistent,
                    }
                },
            };

            (key, value)
        })
        .collect();

    let datums_map: DashMap<Par, Vec<Datum<ListParWithRandom>>> = cache_snapshot_proto
        .data
        .into_iter()
        .map(|map_entry| {
            let key = map_entry.key.unwrap();
            let value = map_entry
                .value
                .into_iter()
                .map(|datum_proto| Datum {
                    a: datum_proto.a.unwrap(),
                    persist: datum_proto.persist,
                    source: {
                        let produce_proto = datum_proto.source.unwrap();
                        Produce {
                            channel_hash: Blake2b256Hash::from_bytes(produce_proto.channel_hash),
                            hash: Blake2b256Hash::from_bytes(produce_proto.hash),
                            persistent: produce_proto.persistent,
                        }
                    },
                })
                .collect();

            (key, value)
        })
        .collect();

    let joins_map: DashMap<Par, Vec<Vec<Par>>> = cache_snapshot_proto
        .joins
        .into_iter()
        .map(|map_entry| {
            let key = map_entry.key.unwrap();
            let value = map_entry
                .value
                .into_iter()
                .map(|join_proto| join_proto.join)
                .collect();

            (key, value)
        })
        .collect();

    let installed_joins_map: DashMap<Par, Vec<Vec<Par>>> = cache_snapshot_proto
        .installed_joins
        .into_iter()
        .map(|map_entry| {
            let key = map_entry.key.unwrap();
            let value = map_entry
                .value
                .into_iter()
                .map(|join_proto| join_proto.join)
                .collect();

            (key, value)
        })
        .collect();

    let produce_counter_map: HashMap<Produce, i32> = soft_checkpoint_proto
        .produce_counter
        .into_iter()
        .map(|map_entry| {
            let key_proto = map_entry.key.unwrap();
            let produce = Produce {
                channel_hash: Blake2b256Hash::from_bytes(key_proto.channel_hash),
                hash: Blake2b256Hash::from_bytes(key_proto.hash),
                persistent: key_proto.persistent,
            };

            let value = map_entry.value;

            (produce, value)
        })
        .collect();

    let cache_snapshot = HotStoreState {
        continuations: conts_map,
        installed_continuations: installed_conts_map,
        data: datums_map,
        joins: joins_map,
        installed_joins: installed_joins_map,
    };

    let soft_checkpoint = SoftCheckpoint {
        cache_snapshot,
        produce_counter: produce_counter_map,
    };

    let mut rspace_lock = unsafe { (*rspace).rspace.lock().unwrap() };

    rspace_lock
        .revert_to_soft_checkpoint(soft_checkpoint)
        .expect("Rust RSpacePlusPlus Library: Failed to revert to soft checkpoint");
}

#[no_mangle]
pub extern "C" fn deallocate_memory(ptr: *mut u8, len: usize) {
    // SAFETY: The caller must guarantee that `ptr` is a valid pointer to a memory block
    // allocated by Rust, and that `len` is the correct size of the block.
    // println!("\nhit deallocate_memory");
    unsafe {
        // Convert the raw pointer back to a Box to allow Rust to deallocate the memory.
        let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        // The Box goes out of scope here, and Rust automatically deallocates the memory.
    }
}
