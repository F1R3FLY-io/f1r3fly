// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeSpec.scala

use std::sync::{Arc, Mutex};

use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use rholang::rust::interpreter::{
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
    "Blake2b256Hash(5144948f445bdb22d59ecd2fe012d53dcff9474f2d965197ad947cfc87c70c23)".to_string()
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
    let runtime = create_rho_runtime(
        Arc::new(Mutex::new(Box::new(space))),
        Par::default(),
        false,
        &mut Vec::new(),
    )
    .await;

    assert_eq!(
        empty_state_hash_fixed(),
        empty_state_hash_from_runtime(runtime).await.to_string()
    );
}
