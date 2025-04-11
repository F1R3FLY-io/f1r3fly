use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{tagged_continuation::TaggedCont, Par};
use models::rhoapi::{ListParWithRandom, TaggedContinuation};
use std::sync::{Arc, RwLock};

use super::system_processes::RhoDispatchMap;
use super::{env::Env, errors::InterpreterError, reduce::DebruijnInterpreter, unwrap_option_safe};

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
    pub _dispatch_table: RhoDispatchMap,
    pub reducer: Option<DebruijnInterpreter>,
}

pub type RhoDispatch = Arc<RwLock<RholangAndScalaDispatcher>>;

impl RholangAndScalaDispatcher {
    pub async fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<ListParWithRandom>,
    ) -> Result<(), InterpreterError> {
        // println!("\ndispatcher dispatch");
        // println!("continuation: {:?}", continuation);
        match continuation.tagged_cont {
            Some(cont) => match cont {
                TaggedCont::ParBody(par_with_rand) => {
                    let env = build_env(data_list.clone());
                    let mut randoms =
                        vec![Blake2b512Random::from_bytes(&par_with_rand.random_state)];
                    randoms.extend(
                        data_list
                            .iter()
                            .map(|p| Blake2b512Random::from_bytes(&p.random_state)),
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
                    // println!("self {:p}", self);
                    let dispatch_table = &self._dispatch_table.try_read().unwrap();
                    // println!(
                    //     "dispatch_table at ScalaBodyRef: {:?}",
                    //     dispatch_table.keys()
                    // );
                    match dispatch_table.get(&_ref) {
                        Some(f) => Ok(f(data_list).await),
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
