use models::rhoapi::{ListParWithRandom, Par, TaggedContinuation};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

use crate::rust::interpreter::{
    accounting::_cost,
    dispatch::{Dispatch, RhoDispatch},
    errors::InterpreterError,
    reduce::DebruijnInterpreter,
    rho_runtime::RhoTuplespace,
    substitute::Substitute,
};

pub struct RholangOnlyDispatcher {
    // reducer: DebruijnInterpreter,
}

impl RholangOnlyDispatcher {
    pub fn create_with_merge_chs(
        tuplespace: RhoTuplespace,
        urn_map: HashMap<String, Par>,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
        cost: _cost,
    ) -> (RhoDispatch, DebruijnInterpreter) {
        let dispatcher = Arc::new(Mutex::new(Box::new(RholangOnlyDispatcher {})
            as Box<dyn Dispatch<ListParWithRandom, TaggedContinuation>>));

        let reducer = DebruijnInterpreter {
            space: tuplespace,
            dispatcher: dispatcher.clone(),
            urn_map,
            merge_chs,
            mergeable_tag_name: Par::default(),
            cost: cost.clone(),
            substitute: Substitute { cost },
        };

        (dispatcher, reducer)
    }

    pub fn create(
        tuplespace: RhoTuplespace,
        urn_map: HashMap<String, Par>,
        cost: _cost,
    ) -> (RhoDispatch, DebruijnInterpreter) {
        let init_merge_channels_mutex = Arc::new(RwLock::new(HashSet::new()));

        RholangOnlyDispatcher::create_with_merge_chs(
            tuplespace,
            urn_map,
            init_merge_channels_mutex,
            cost,
        )
    }
}

impl Dispatch<ListParWithRandom, TaggedContinuation> for RholangOnlyDispatcher {
    fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<ListParWithRandom>,
    ) -> Result<(), InterpreterError> {
        todo!()
    }
}
