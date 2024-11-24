// See rholang/src/main/scala/coop/rchain/rholang/interpreter/dispatch.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{tagged_continuation::TaggedCont, Par};
use models::rhoapi::{BindPattern, ListParWithRandom, TaggedContinuation};
use rspace_plus_plus::rspace::rspace_interface::ISpace;
use rspace_plus_plus::rspace::tuplespace_interface::Tuplespace;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use super::{
    accounting::_cost, env::Env, errors::InterpreterError, reduce::DebruijnInterpreter,
    substitute::Substitute, unwrap_option_safe,
};

pub fn build_env(data_list: Vec<ListParWithRandom>) -> Env<Par> {
    let pars: Vec<Par> = data_list.into_iter().flat_map(|list| list.pars).collect();
    let mut env = Env::new();

    for par in pars {
        env = env.put(par);
    }

    env
}

#[derive(Clone)]
pub struct RholangAndScalaDispatcher {
    pub _dispatch_table:
        Arc<std::sync::Mutex<HashMap<i64, Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>>>>,
    pub reducer: Option<DebruijnInterpreter>,
}

impl RholangAndScalaDispatcher {
    pub async fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<ListParWithRandom>,
    ) -> Result<(), InterpreterError> {
        // println!("\ndispatch");
        match continuation.tagged_cont {
            Some(cont) => match cont {
                TaggedCont::ParBody(par_with_rand) => {
                    let env = build_env(data_list.clone());
                    let mut randoms = vec![Blake2b512Random::new(&par_with_rand.random_state)];
                    randoms.extend(
                        data_list
                            .iter()
                            .map(|p| Blake2b512Random::new(&p.random_state)),
                    );

                    self.reducer
                        .clone()
                        .unwrap()
                        .eval(
                            unwrap_option_safe(par_with_rand.body)?,
                            &env,
                            Blake2b512Random::merge(randoms),
                        )
                        .await
                }
                TaggedCont::ScalaBodyRef(_ref) => {
                    match self._dispatch_table.try_lock().unwrap().get_mut(&_ref) {
                        Some(f) => Ok(f(data_list)),
                        None => Err(InterpreterError::BugFoundError(format!(
                            "dispatch: no function for {}",
                            _ref,
                        ))),
                    }
                }
            },
            None => Ok(()),
        }
    }
}

pub type RhoDispatch = Arc<RwLock<RholangAndScalaDispatcher>>;

impl RholangAndScalaDispatcher {
    pub fn create<T>(
        tuplespace: T,
        dispatch_table: HashMap<i64, Box<dyn FnMut(Vec<ListParWithRandom>) -> ()>>,
        urn_map: HashMap<String, Par>,
        merge_chs: Arc<RwLock<HashSet<Par>>>,
        mergeable_tag_name: Par,
        cost: _cost,
    ) -> (RholangAndScalaDispatcher, DebruijnInterpreter)
    where
        T: ISpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> + 'static,
    {
        // println!("\ncreate");
        let mut dispatcher = RholangAndScalaDispatcher {
            _dispatch_table: Arc::new(std::sync::Mutex::new(dispatch_table)),
            reducer: None,
        };

        let mut reducer = DebruijnInterpreter {
            space: Arc::new(std::sync::Mutex::new(Box::new(tuplespace))),
            dispatcher: Arc::new(RwLock::new(dispatcher.clone())),
            urn_map,
            merge_chs,
            mergeable_tag_name,
            cost: cost.clone(),
            substitute: Substitute { cost },
        };

        dispatcher.reducer = Some(reducer.clone());
        let dispatcher_arc = Arc::new(RwLock::new(dispatcher.clone()));
        reducer.dispatcher = dispatcher_arc.clone();
        let mut dispatcher_locked = dispatcher_arc.try_write().unwrap();
        dispatcher_locked.reducer = Some(reducer.clone());
        drop(dispatcher_locked);

        (dispatcher, reducer)
    }

    pub fn create_dispatcher<T>(tuplespace: T, cost: _cost) -> RhoDispatch
    where
        T: ISpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> + 'static,
    {
        // println!("\ncreate_dispatcher");
        let mut dispatcher = RholangAndScalaDispatcher {
            _dispatch_table: Arc::new(std::sync::Mutex::new(HashMap::new())),
            reducer: None,
        };

        let mut reducer = DebruijnInterpreter {
            space: Arc::new(std::sync::Mutex::new(Box::new(tuplespace))),
            dispatcher: Arc::new(RwLock::new(dispatcher.clone())),
            urn_map: HashMap::new(),
            merge_chs: Arc::new(RwLock::new(HashSet::new())),
            mergeable_tag_name: Par::default(),
            cost: cost.clone(),
            substitute: Substitute { cost },
        };

        dispatcher.reducer = Some(reducer.clone());
        let dispatcher_arc = Arc::new(RwLock::new(dispatcher.clone()));
        reducer.dispatcher = dispatcher_arc.clone();
        let mut dispatcher_locked = dispatcher_arc.try_write().unwrap();
        dispatcher_locked.reducer = Some(reducer.clone());
        drop(dispatcher_locked);

        Arc::new(RwLock::new(dispatcher))
    }
}
