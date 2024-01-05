use prost::Message;
use rspace_plus_plus::rspace::hot_store::HotStore;
use rspace_plus_plus::rspace::matcher::exports::*;
use rspace_plus_plus::rspace::matcher::fold_match::FoldMatch;
use rspace_plus_plus::rspace::matcher::r#match::Match;
use rspace_plus_plus::rspace::matcher::spatial_matcher::SpatialMatcherContext;
use rspace_plus_plus::rspace::rspace::RSpace;

#[derive(Clone)]
struct SpaceMatcher;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/storage/package.scala - matchListPar
impl Match<BindPattern, ListParWithRandom> for SpaceMatcher {
    fn get(&self, pattern: BindPattern, data: ListParWithRandom) -> Option<ListParWithRandom> {
        let mut spatial_matcher = SpatialMatcherContext::new();

        let fold_match_result =
            spatial_matcher.fold_match(data.pars, pattern.patterns, pattern.remainder.clone());
        let match_result = match fold_match_result {
            Some(pars) => Some((spatial_matcher.free_map, pars)),
            None => None,
        };

        match match_result {
            Some((mut free_map, caught_rem)) => {
                let remainder_map = match pattern.remainder {
                    Some(Var {
                        var_instance: Some(FreeVar(level)),
                    }) => {
                        free_map.insert(
                            level,
                            vector_par(Vec::new(), false).with_exprs(vec![new_elist_expr(
                                caught_rem,
                                Vec::new(),
                                false,
                                None,
                            )]),
                        );
                        free_map
                    }
                    _ => free_map,
                };
                Some(ListParWithRandom {
                    pars: to_vec(remainder_map, pattern.free_count),
                    random_state: data.random_state,
                })
            }
            None => None,
        }
    }
}

/*
 * This library contains predefined types for Channel, Pattern, Data, and Continuation - RhoTypes
 * These types (C, P, A, K) MUST MATCH the corresponding types on the Scala side in 'RSpacePlusPlus_RhoTypes.scala'
 */
#[repr(C)]
pub struct Space {
    rspace: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation, SpaceMatcher>,
}

#[no_mangle]
pub extern "C" fn space_new() -> *mut Space {
    Box::into_raw(Box::new(Space {
        rspace: RSpace::create(SpaceMatcher),
    }))
}

#[no_mangle]
pub extern "C" fn space_print(rspace: *mut Space) -> () {
    unsafe { (*rspace).rspace.store.print() }
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

    let result_option = unsafe { (*rspace).rspace.produce(channel, data, persist) };

    match result_option {
        Some((cont_result, rspace_results)) => {
            let protobuf_cont_result = ContResult {
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

            let maybe_action_result = RhoTypesActionResult {
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
    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let consume_params = ConsumeParams::decode(payload_slice).unwrap();

    let channels = consume_params.channels;
    let patterns = consume_params.patterns;
    let continuation = consume_params.continuation.unwrap();
    let persist = consume_params.persist;
    let peeks = consume_params.peeks.into_iter().map(|e| e.value).collect();

    let result_option = unsafe {
        (*rspace)
            .rspace
            .consume(channels, patterns, continuation, persist, peeks)
    };

    match result_option {
        Some((cont_result, rspace_results)) => {
            let protobuf_cont_result = ContResult {
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

            let maybe_action_result = RhoTypesActionResult {
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

    let result_option = unsafe { (*rspace).rspace.install(channels, patterns, continuation) };

    match result_option {
        None => std::ptr::null(),
        Some(_) => {
            panic!("RUST ERROR: Installing can be done only on startup")
        }
    }
}

#[no_mangle]
pub extern "C" fn to_map(rspace: *mut Space) -> *const u8 {
    let hot_store_mapped = unsafe { (*rspace).rspace.store.to_map() };
    let mut map_entries: Vec<MapEntry> = Vec::new();

    for (key, value) in hot_store_mapped {
        let datums = value
            .data
            .into_iter()
            .map(|datum| DatumProto {
                a: Some(datum.a),
                persist: datum.persist,
                source: Some(ProduceProto {
                    channel_hash: datum.source.channel_hash.as_bytes().to_vec(),
                    hash: datum.source.hash.as_bytes().to_vec(),
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
                        .map(|hash| hash.as_bytes().to_vec())
                        .collect(),
                    hash: wk.source.hash.as_bytes().to_vec(),
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
}

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
