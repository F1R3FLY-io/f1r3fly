// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeSpec.scala

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use rholang::rust::interpreter::{
    accounting::costs::Cost,
    matcher::r#match::Matcher,
    rho_runtime::{bootstrap_registry, create_rho_runtime, RhoRuntime, RhoRuntimeImpl},
};
use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    history::instances::radix_history::RadixHistory,
    rspace::RSpace,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
};

fn empty_state_hash_fixed() -> String {
    "Blake2b256Hash(cb75e7f94e8eac21f95c524a07590f2583fbdaba6fb59291cf52fa16a14c784d)".to_string()
}

async fn empty_state_hash_from_runtime(runtime: Arc<Mutex<RhoRuntimeImpl>>) -> Blake2b256Hash {
    let mut runtime_lock = runtime.lock().unwrap();
    let _ = runtime_lock.reset(RadixHistory::empty_root_node_hash());
    drop(runtime_lock);
    let _ = bootstrap_registry(runtime.clone()).await;

    let mut runtime_lock = runtime.lock().unwrap();
    let checkpoint = runtime_lock.create_checkpoint();
    checkpoint.root
}

#[tokio::test]
async fn empty_state_hash_should_be_the_same_as_hard_coded_cached_value() {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> =
        RSpace::create(store, Arc::new(Box::new(Matcher))).unwrap();
    let runtime = create_rho_runtime(space, Par::default(), false, &mut Vec::new()).await;

    assert_eq!(
        empty_state_hash_fixed(),
        empty_state_hash_from_runtime(runtime).await.to_string()
    );
}

#[tokio::test]
async fn runtime_should_evaluate_successfully() {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> =
        RSpace::create(store, Arc::new(Box::new(Matcher))).unwrap();
    let runtime = create_rho_runtime(space, Par::default(), true, &mut Vec::new()).await;

    let rholang_code = r#"
new x, y, stdout(`rho:io:stdout`), keccak256Hash(`rho:crypto:keccak256Hash`) in {
    x!(@"name"!("Joe") | @"age"!(40)) |  // (1)
        for (@r <- x) {
            //hashing channel expect byte array as input, this is true for all 3 channels:
            //keccak256Hash, sha256Hash and blake2b256Hash
            keccak256Hash!(r.toByteArray(), *y) // hash the program from (1)
        } |
        for (@h <- y) {
            // the h here is hash of the rholang term we sent to the hash channel
            // we can do anything we want with it but we choose to just print it
            stdout!(h)  // print out the keccak256 hash
        }
}
    "#;

    let eval_result = runtime
        .try_lock()
        .unwrap()
        .evaluate(
            rholang_code.to_string(),
            Cost::unsafe_max(),
            HashMap::new(),
            Blake2b512Random::create_from_bytes(&[]),
        )
        .await;
    assert!(eval_result.is_ok());
    assert!(eval_result.clone().unwrap().errors.is_empty());
}
