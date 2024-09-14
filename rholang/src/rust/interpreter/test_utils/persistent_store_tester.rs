use models::rhoapi::Par;
use rspace_plus_plus::rspace::{
    rspace::RSpaceInstances,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

use crate::rust::interpreter::{
    accounting::{cost_accounting::CostAccounting, costs::Cost},
    dispatch::RholangAndScalaDispatcher,
    matcher::r#match::Matcher,
    reduce::DebruijnInterpreter,
    rho_runtime::RhoTuplespace,
};

pub async fn create_test_space() -> (RhoTuplespace, DebruijnInterpreter) {
    let cost = CostAccounting::empty_cost();
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space = Arc::new(Mutex::new(
        RSpaceInstances::create(store, Arc::new(Box::new(Matcher))).unwrap(),
    ));
    let reducer = RholangAndScalaDispatcher::create(
        space.clone(),
        HashMap::new(),
        HashMap::new(),
        Arc::new(RwLock::new(HashSet::new())),
        Par::default(),
        cost.clone(),
    )
    .1;

    cost.set(Cost::create(
        i64::MAX,
        "persistent_store_tester setup".to_string(),
    ));

    (space, reducer)
}
