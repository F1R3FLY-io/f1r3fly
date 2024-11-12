use std::sync::{Arc, Mutex};

use models::rholang_scala_rust_types::*;
use prost::Message;
use rspace_plus_plus::rspace::shared::{
    key_value_store_manager::KeyValueStoreManager, lmdb_dir_store_manager::GB,
    rspace_store_manager::mk_rspace_store_manager,
};
use rust::interpreter::{
    matcher::r#match::Matcher,
    rho_runtime::{create_runtime_from_kv_store, RhoRuntimeImpl},
};

pub mod rust {

    pub mod interpreter;
}

#[repr(C)]
struct RhoRuntime {
    runtime: Arc<Mutex<RhoRuntimeImpl>>,
}

// Note: I am defaulting 'additional_system_processes' to 'Vec::new()'
#[no_mangle]
extern "C" fn create_runtime(
    payload_pointer: *const u8,
    payload_bytes_len: usize,
) -> *mut RhoRuntime {
    let payload_slice = unsafe { std::slice::from_raw_parts(payload_pointer, payload_bytes_len) };
    let params = CreateRuntimeParams::decode(payload_slice).unwrap();

    let store_path = params.store_path;
    let mergeable_tag_name = params.mergeable_tag_name.unwrap();
    let init_registry = params.init_registry;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let rspace_store = rt.block_on(async {
        let mut kvm =
            mk_rspace_store_manager((&format!("{}/rspace++/", store_path)).into(), 1 * GB);
        let store = kvm.r_space_stores().await.unwrap();
        store
    });

    let runtime = create_runtime_from_kv_store(
        rspace_store,
        mergeable_tag_name,
        init_registry,
        &mut Vec::new(),
        Arc::new(Box::new(Matcher)),
    );
    Box::into_raw(Box::new(RhoRuntime { runtime }))
}
