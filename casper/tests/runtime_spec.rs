// See casper/src/test/scala/coop/rchain/casper/util/rholang/RuntimeSpec.scala

use std::collections::HashMap;
use std::sync::Arc;

use casper::rust::util::rholang::runtime_manager;
use casper::rust::util::rholang::tools::Tools;
use casper::rust::{genesis::genesis::Genesis, rholang::runtime_syntax::RuntimeOps};
use rholang::rust::interpreter::accounting::costs::Cost;
use rholang::rust::interpreter::rho_runtime::RhoRuntime;
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

#[tokio::test]
async fn state_hash_after_fixed_rholang_term_execution_should_be_hash_fixed_without_hard_fork() {
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

    let contract = r#"
         new a in {
           @"2"!(10)|
           @2!("test")|
           @"3"!!(3)|
           @42!!("1")|
           for (@t <- a){Nil}|
           for (@num <- @"3"&@num2 <- @1){10}|
           for (@_ <= @"4"){"3"}|
           for (@_ <= @"5"& @num3 <= @5){Nil}|
           for (@3 <- @44){new g in {Nil}}|
           for (@_ <- @"55"& @num3 <- @55){Nil}
         }
        "#
    .trim_start();

    let random = Tools::rng(&Blake2b256Hash::from_bytes(vec![0; 1]).bytes());
    let mut runtime_lock = runtime.lock().unwrap();
    let r = runtime_lock
        .evaluate(
            contract.to_string(),
            Cost::unsafe_max(),
            HashMap::new(),
            random,
        )
        .await;

    println!("r: {:?}", r);
    assert!(r.is_ok());
    assert!(r.unwrap().errors.is_empty());

    let checkpoint = runtime_lock.create_checkpoint();
    let expected_hash = Blake2b256Hash::from_hex(
        "18e91cdc71e51e08a1a0f3f8aebf7d58b9768e05b7539da02cc953fc9d548fc4",
    );

    assert_eq!(expected_hash, checkpoint.root);
}
