use std::sync::{Arc, Mutex};

use models::{
    rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation},
    rholang_scala_rust_types::*,
};
use prost::Message;
use rspace_plus_plus::rspace::rspace::RSpace;
use rust::interpreter::rho_runtime::{create_rho_runtime, RhoRuntimeImpl};

pub mod rust {

    pub mod interpreter;
}

#[repr(C)]
struct RhoRuntime {
    runtime: Arc<Mutex<RhoRuntimeImpl>>,
}

#[repr(C)]
pub struct Space {
    rspace: Mutex<RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>>,
}

// Note: I am defaulting 'additional_system_processes' to 'Vec::new()'
#[no_mangle]
extern "C" fn create_runtime(
    rspace_ptr: *mut Space,
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> *mut RhoRuntime {
    let rspace = unsafe { (*rspace_ptr).rspace.lock().unwrap().clone() };

    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let params = CreateRuntimeParams::decode(payload_slice).unwrap();

    let mergeable_tag_name = params.mergeable_tag_name.unwrap();
    let init_registry = params.init_registry;

    let runtime = create_rho_runtime(rspace, mergeable_tag_name, init_registry, &mut Vec::new());
    Box::into_raw(Box::new(RhoRuntime { runtime }))
}
