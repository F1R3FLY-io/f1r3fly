use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::TaggedContinuation;
use std::sync::{Arc, RwLock};

use models::rhoapi::{ListParWithRandom, Par};

use super::system_processes::RhoDispatchMap;
use super::{env::Env, errors::InterpreterError, reduce::DebruijnInterpreter};

pub fn build_env(data_list: Vec<ListParWithRandom>) -> Env<Par> {
    data_list
        .into_iter()
        .flat_map(|v| v.pars)
        .fold(Env::default(), |acc, value| acc.put(value))
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
        data_list: Vec<models::rhoapi::ListParWithRandom>,
    ) -> Result<(), InterpreterError> {
        match continuation.tagged_cont {
            Some(cont) => match cont {
                TaggedCont::ParBody(par_with_rand) => {
                    let mut randoms =
                        vec![Blake2b512Random::from_bytes(&par_with_rand.random_state)];
                    randoms.extend(
                        data_list
                            .iter()
                            .map(|p| Blake2b512Random::from_bytes(&p.random_state)),
                    );

                    let par = par_with_rand
                        .body
                        .map(Into::into)
                        .ok_or(InterpreterError::UndefinedRequiredProtobufFieldError)?;

                    self.reducer
                        .clone()
                        .unwrap()
                        .evaluate(par, Env::default(), Blake2b512Random::merge(randoms))
                        .await
                }
                TaggedCont::ScalaBodyRef(_ref) => {
                    let dispatch_table = &self._dispatch_table.try_read().unwrap();
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
