use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock},
};

use models::rhoapi::Par;
use models::rhoapi::{ListParWithRandom, TaggedContinuation};

use super::{errors::InterpreterError, reduce::DebruijnInterpreter, rho_runtime::RhoTuplespace};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/dispatch.scala
pub trait Dispatch<A, K> {
    fn dispatch(continuation: K, data_list: Vec<A>) -> ();
}

pub type RhoDispatch = Arc<Mutex<RholangAndRustDispatcher>>;

#[derive(Clone)]
pub struct RholangAndRustDispatcher {
    _dispatch_table: Arc<Mutex<HashMap<i64, Box<dyn Fn(Vec<ListParWithRandom>) -> ()>>>>,
}

impl RholangAndRustDispatcher {
    pub fn create(
        tuplespace: RhoTuplespace,
        dispatch_table: HashMap<i64, Box<dyn Fn(Vec<ListParWithRandom>) -> ()>>,
        urn_map: HashMap<String, Par>,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
        mergeable_tag_name: Par,
    ) -> (RholangAndRustDispatcher, DebruijnInterpreter) {
        let dispatcher = RholangAndRustDispatcher {
            _dispatch_table: Arc::new(Mutex::new(dispatch_table)),
        };

        let reducer = DebruijnInterpreter {
            space: tuplespace,
            dispatcher: dispatcher.clone(),
            urn_map,
            merge_chs,
            mergeable_tag_name,
        };

        (dispatcher, reducer)
    }

    pub async fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<ListParWithRandom>,
    ) -> Result<(), InterpreterError> {
        todo!()
    }
}
