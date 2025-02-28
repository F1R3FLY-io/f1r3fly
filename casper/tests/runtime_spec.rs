// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeSpec.scala

use std::sync::Arc;

use casper::rust::util::rholang::runtime_manager;
use casper::rust::{genesis::genesis::Genesis, rholang::runtime_syntax::RuntimeOps};
use rholang::rust::interpreter::{
    matcher::r#match::Matcher, rho_runtime::create_runtime_from_kv_store,
};
use rspace_plus_plus::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
};

#[tokio::test]
async fn empty_state_hash_should_be_the_same_as_hard_coded_cached_value() {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let runtime = create_runtime_from_kv_store(
        store,
        Genesis::non_negative_mergeable_tag_name(),
        false,
        &mut Vec::new(),
        Arc::new(Box::new(Matcher)),
    )
    .await;
    let runtime_ops = RuntimeOps::new(runtime);

    let hard_coded_hash = runtime_manager::empty_state_hash_fixed();
    let empty_root_hash = runtime_ops.empty_state_hash().await.unwrap();

    let empty_hash_hard_coded = Blake2b256Hash::from_bytes(hard_coded_hash.to_vec());
    let empty_hash = Blake2b256Hash::from_bytes(empty_root_hash.to_vec());

    assert_eq!(empty_hash_hard_coded, empty_hash);
}
