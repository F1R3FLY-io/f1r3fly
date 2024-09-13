use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

use models::rhoapi::Par;
use models::rhoapi::{ListParWithRandom, TaggedContinuation};

use super::{
    accounting::_cost, env::Env, errors::InterpreterError, reduce::DebruijnInterpreter,
    rho_runtime::RhoTuplespace, substitute::Substitute,
};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/dispatch.scala
pub trait Dispatch<A, K> {
    fn dispatch(&self, continuation: K, data_list: Vec<A>) -> Result<(), InterpreterError>;
}

pub fn build_env(data_list: Vec<ListParWithRandom>) -> Env<Par> {
    let pars: Vec<Par> = data_list.into_iter().flat_map(|list| list.pars).collect();
    let mut env = Env::new();

    for par in pars {
        env = env.put(par);
    }

    env
}

#[derive(Clone)]
pub struct RholangAndRustDispatcher {
    pub _dispatch_table: Arc<Mutex<HashMap<i64, Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>>>>,
}

impl Dispatch<ListParWithRandom, TaggedContinuation> for RholangAndRustDispatcher {
    fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<ListParWithRandom>,
    ) -> Result<(), InterpreterError> {
        todo!()
    }
}

pub type RhoDispatch = Arc<Mutex<Box<dyn Dispatch<ListParWithRandom, TaggedContinuation>>>>;

impl RholangAndRustDispatcher {
    pub fn create(
        tuplespace: RhoTuplespace,
        dispatch_table: HashMap<i64, Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>>,
        urn_map: HashMap<String, Par>,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
        mergeable_tag_name: Par,
        cost: _cost,
    ) -> (RholangAndRustDispatcher, DebruijnInterpreter) {
        let dispatcher = RholangAndRustDispatcher {
            _dispatch_table: Arc::new(Mutex::new(dispatch_table)),
        };

        let reducer = DebruijnInterpreter {
            space: tuplespace,
            dispatcher: Arc::new(Mutex::new(Box::new(dispatcher.clone()))),
            urn_map,
            merge_chs,
            mergeable_tag_name,
            cost: cost.clone(),
            substitute: Substitute { cost },
        };

        (dispatcher, reducer)
    }
}
