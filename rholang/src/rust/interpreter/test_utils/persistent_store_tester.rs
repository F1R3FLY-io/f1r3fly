use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use rspace_plus_plus::rspace::{
    rspace::RSpace,
    rspace_interface::ISpace,
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
    matcher::r#match::Matcher,
    reduce::DebruijnInterpreter,
    rho_runtime::RhoISpace,
};

pub async fn create_test_space<T>() -> (
    impl ISpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
    DebruijnInterpreter,
)
where
    T: ISpace<Par, BindPattern, ListParWithRandom, TaggedContinuation>,
{
    let cost = CostAccounting::empty_cost();
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space = RSpace::create(store, Arc::new(Box::new(Matcher))).unwrap();
    let rspace: RhoISpace = Arc::new(Mutex::new(Box::new(space.clone())));

    let reducer = DebruijnInterpreter::new(
        rspace,
        HashMap::new(),
        Arc::new(RwLock::new(HashSet::new())),
        Par::default(),
        cost.clone(),
    );

    cost.set(Cost::create(
        i64::MAX,
        "persistent_store_tester setup".to_string(),
    ));

    (space, reducer)
}
