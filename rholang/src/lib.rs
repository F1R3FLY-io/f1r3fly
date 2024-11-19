pub mod rust {

    pub mod interpreter;
}

use std::sync::{Arc, Mutex};

use crypto::rust::{hash::blake2b512_random::Blake2b512Random, public_key::PublicKey};
use models::{
    rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation},
    rholang_scala_rust_types::*,
};
use prost::Message;
use rspace_plus_plus::rspace::rspace::RSpace;
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
    bootstrap_registry_internal(runtime);
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

    let runtime = create_rho_runtime(rspace, mergeable_tag_name, init_registry, &mut Vec::new());
    Box::into_raw(Box::new(RhoRuntime { runtime }))
}
