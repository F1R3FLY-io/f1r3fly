pub mod rust {

    pub mod interpreter;
}

use std::sync::{Arc, Mutex};

use crypto::rust::public_key::PublicKey;
use models::{
    rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation},
    rholang_scala_rust_types::*,
};
use prost::Message;
use rspace_plus_plus::rspace::rspace::RSpace;
use rust::interpreter::{
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
            .lock()
            .unwrap()
            .set_block_data(block_data);
    }
}

#[no_mangle]
extern "C" fn bootstrap_registry(runtime_ptr: *mut RhoRuntime) -> () {
    let runtime = unsafe { (*runtime_ptr).runtime.clone() };
    bootstrap_registry_internal(runtime);
}

// Note: I am defaulting 'additional_system_processes' to 'Vec::new()'
#[no_mangle]
extern "C" fn create_runtime(
    rspace_ptr: *mut Space,
    params_ptr: *const u8,
    params_bytes_len: usize,
) -> *mut RhoRuntime {
    let rspace = unsafe { (*rspace_ptr).rspace.lock().unwrap().clone() };

    let params_slice = unsafe { std::slice::from_raw_parts(params_ptr, params_bytes_len) };
    let params = CreateRuntimeParams::decode(params_slice).unwrap();

    let mergeable_tag_name = params.mergeable_tag_name.unwrap();
    let init_registry = params.init_registry;

    let runtime = create_rho_runtime(rspace, mergeable_tag_name, init_registry, &mut Vec::new());
    Box::into_raw(Box::new(RhoRuntime { runtime }))
}
