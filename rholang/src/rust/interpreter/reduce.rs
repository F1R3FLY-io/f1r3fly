// See See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::g_unforgeable::UnfInstance;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::var::VarInstance;
use models::rhoapi::{
    BindPattern, Bundle, Expr, GPrivate, GUnforgeable, Match, MatchCase, New, ParWithRandom,
    Receive, ReceiveBind, Send, Var,
};
use models::rhoapi::{ETuple, ListParWithRandom, Par, TaggedContinuation};
use models::rust::rholang::implicits::{concatenate_pars, single_bundle};
use models::ByteString;
use rspace_plus_plus::rspace::history::Either;
use rspace_plus_plus::rspace::util::unpack_option_with_peek;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use crate::rust::interpreter::accounting::costs::match_eval_cost;
use crate::rust::interpreter::matcher::spatial_matcher::SpatialMatcherContext;

use super::accounting::_cost;
use super::accounting::costs::{
    method_call_cost, new_bindings_cost, receive_eval_cost, send_eval_cost, var_eval_cost,
};
use super::errors::InterpreterError;
use super::rho_type::{RhoExpression, RhoUnforgeable};
use super::substitue::Substitute;
use super::util::GeneratedMessage;
use super::{dispatch::RholangAndRustDispatcher, env::Env, rho_runtime::RhoTuplespace};

/**
 * Reduce is the interface for evaluating Rholang expressions.
 */
pub trait Reduce {
    async fn eval(
        &self,
        par: Par,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError>;

    async fn inj(&self, par: Par, rand: Blake2b512Random) -> Result<(), InterpreterError> {
        self.eval(par, &Env::new(), rand).await
    }
}

#[derive(Clone)]
pub struct DebruijnInterpreter {
    pub space: RhoTuplespace,
    pub dispatcher: RholangAndRustDispatcher,
    pub urn_map: HashMap<String, Par>,
    pub merge_chs: Arc<RwLock<HashSet<Par>>>,
    pub mergeable_tag_name: Par,
    pub cost: _cost,
    pub substitute: Substitute,
}

impl Reduce for DebruijnInterpreter {
    async fn eval(
        &self,
        par: Par,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        let terms: Vec<GeneratedMessage> = vec![
            par.sends
                .into_iter()
                .map(GeneratedMessage::Send)
                .collect::<Vec<_>>(),
            par.receives
                .into_iter()
                .map(GeneratedMessage::Receive)
                .collect(),
            par.news.into_iter().map(GeneratedMessage::New).collect(),
            par.matches
                .into_iter()
                .map(GeneratedMessage::Match)
                .collect(),
            par.bundles
                .into_iter()
                .map(GeneratedMessage::Bundle)
                .collect(),
            par.exprs
                .into_iter()
                .filter(|expr| match &expr.expr_instance {
                    Some(expr_instance) => match expr_instance {
                        ExprInstance::EVarBody(_) => true,
                        ExprInstance::EMethodBody(_) => true,
                        _ => false,
                    },
                    None => false,
                })
                .collect::<Vec<Expr>>()
                .into_iter()
                .map(GeneratedMessage::Expr)
                .collect(),
        ]
        .into_iter()
        .filter(|vec| !vec.is_empty())
        .flatten()
        .collect();

        fn split(
            id: i32,
            terms: &Vec<GeneratedMessage>,
            rand: Blake2b512Random,
        ) -> Blake2b512Random {
            if terms.len() == 1 {
                rand
            } else if terms.len() > 256 {
                rand.split_short(id.try_into().unwrap())
            } else {
                rand.split_byte(id.try_into().unwrap())
            }
        }

        let term_split_limit = i16::MAX;
        if terms.len() > term_split_limit.try_into().unwrap() {
            Err(InterpreterError::ReduceError(format!(
                "The number of terms in the Par is {}, which exceeds the limit of {}",
                terms.len(),
                term_split_limit
            )))
        } else {
            // Collect errors from all parallel execution paths (pars)
            // parTraverseSafe
            let futures: Vec<Pin<Box<dyn futures::Future<Output = Result<(), InterpreterError>>>>> =
                terms
                    .iter()
                    .enumerate()
                    .map(|(index, term)| {
                        Box::pin(self.generated_message_eval(
                            term,
                            env,
                            split(index.try_into().unwrap(), &terms, rand.clone()),
                        ))
                            as Pin<Box<dyn futures::Future<Output = Result<(), InterpreterError>>>>
                    })
                    .collect();

            let results: Vec<Result<(), InterpreterError>> =
                futures::future::join_all(futures).await;
            let flattened_results: Vec<InterpreterError> = results
                .into_iter()
                .filter_map(|result| result.err())
                .collect();

            self.aggregate_evaluator_errors(flattened_results)
        }
    }
}

type Application = Option<(
    TaggedContinuation,
    Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
    bool,
)>;

/**
 * Materialize a send in the store, optionally returning the matched continuation.
 *
 * @param chan  The channel on which data is being sent.
 * @param data  The par objects holding the processes being sent.
 * @param persistent  True if the write should remain in the tuplespace indefinitely.
 */
impl DebruijnInterpreter {
    /**
     * Materialize a send in the store, optionally returning the matched continuation.
     *
     * @param chan  The channel on which data is being sent.
     * @param data  The par objects holding the processes being sent.
     * @param persistent  True if the write should remain in the tuplespace indefinitely.
     */
    async fn produce(
        &self,
        chan: Par,
        data: ListParWithRandom,
        persistent: bool,
    ) -> Result<(), InterpreterError> {
        self.update_mergeable_channels(&chan);

        let produce_result =
            self.space
                .lock()
                .unwrap()
                .produce(chan.clone(), data.clone(), persistent);

        self.continue_produce_process(
            unpack_option_with_peek(produce_result),
            chan,
            data,
            persistent,
        )
        .await
    }

    async fn consume(
        &self,
        binds: Vec<(BindPattern, Par)>,
        body: ParWithRandom,
        persistent: bool,
        peek: bool,
    ) -> Result<(), InterpreterError> {
        let (patterns, sources): (Vec<BindPattern>, Vec<Par>) = binds.clone().into_iter().unzip();

        // Update mergeable channels
        for source in &sources {
            self.update_mergeable_channels(source);
        }

        let consume_result = self.space.lock().unwrap().consume(
            sources.clone(),
            patterns.clone(),
            TaggedContinuation {
                tagged_cont: Some(TaggedCont::ParBody(body.clone())),
            },
            persistent,
            if peek {
                BTreeSet::from_iter((0..sources.len() as i32).collect::<Vec<i32>>())
            } else {
                BTreeSet::new()
            },
        );

        self.continue_consume_process(
            unpack_option_with_peek(consume_result),
            binds,
            body,
            persistent,
            peek,
        )
        .await
    }

    async fn continue_produce_process(
        &self,
        res: Application,
        chan: Par,
        data: ListParWithRandom,
        persistent: bool,
    ) -> Result<(), InterpreterError> {
        match res {
            Some((continuation, data_list, peek)) => {
                if persistent {
                    // dispatchAndRun
                    let mut futures: Vec<
                        Pin<Box<dyn futures::Future<Output = Result<(), InterpreterError>>>>,
                    > = vec![Box::pin(self.dispatch(continuation, data_list.clone()))];
                    futures.push(Box::pin(self.produce(chan, data, persistent)));

                    // parTraverseSafe
                    let results: Vec<Result<(), InterpreterError>> =
                        futures::future::join_all(futures).await;
                    let flattened_results: Vec<InterpreterError> = results
                        .into_iter()
                        .filter_map(|result| result.err())
                        .collect();

                    self.aggregate_evaluator_errors(flattened_results)
                } else if peek {
                    // dispatchAndRun
                    let futures = self.produce_peeks(data_list).await;

                    // parTraverseSafe
                    let results: Vec<Result<(), InterpreterError>> =
                        futures::future::join_all(futures).await;
                    let flattened_results: Vec<InterpreterError> = results
                        .into_iter()
                        .filter_map(|result| result.err())
                        .collect();

                    self.aggregate_evaluator_errors(flattened_results)
                } else {
                    self.dispatch(continuation, data_list).await
                }
            }
            None => Ok(()),
        }
    }

    async fn continue_consume_process(
        &self,
        res: Application,
        binds: Vec<(BindPattern, Par)>,
        body: ParWithRandom,
        persistent: bool,
        _peek: bool,
    ) -> Result<(), InterpreterError> {
        match res {
            Some((continuation, data_list, peek)) => {
                if persistent {
                    // dispatchAndRun
                    let mut futures: Vec<
                        Pin<Box<dyn futures::Future<Output = Result<(), InterpreterError>>>>,
                    > = vec![Box::pin(self.dispatch(continuation, data_list.clone()))];
                    futures.push(Box::pin(self.consume(binds, body, persistent, _peek)));

                    // parTraverseSafe
                    let results: Vec<Result<(), InterpreterError>> =
                        futures::future::join_all(futures).await;
                    let flattened_results: Vec<InterpreterError> = results
                        .into_iter()
                        .filter_map(|result| result.err())
                        .collect();

                    self.aggregate_evaluator_errors(flattened_results)
                } else if peek {
                    // dispatchAndRun
                    let futures = self.produce_peeks(data_list).await;

                    // parTraverseSafe
                    let results: Vec<Result<(), InterpreterError>> =
                        futures::future::join_all(futures).await;
                    let flattened_results: Vec<InterpreterError> = results
                        .into_iter()
                        .filter_map(|result| result.err())
                        .collect();

                    self.aggregate_evaluator_errors(flattened_results)
                } else {
                    self.dispatch(continuation, data_list).await
                }
            }
            None => Ok(()),
        }
    }

    async fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
    ) -> Result<(), InterpreterError> {
        self.dispatcher
            .dispatch(
                continuation,
                data_list.into_iter().map(|tuple| tuple.1).collect(),
            )
            .await
    }

    async fn produce_peeks(
        &self,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
    ) -> Vec<Pin<Box<dyn futures::Future<Output = Result<(), InterpreterError>> + '_>>> {
        data_list
            .into_iter()
            .filter(|(_, _, _, persist)| !persist)
            .map(|(chan, _, removed_data, _)| {
                Box::pin(self.produce(chan, removed_data, false))
                    as Pin<Box<dyn futures::Future<Output = Result<(), InterpreterError>>>>
            })
            .collect()
    }

    /* Collect mergeable channels */

    fn update_mergeable_channels(&self, chan: &Par) -> () {
        let is_mergeable = self.is_mergeable_channel(chan);

        if is_mergeable {
            let mut merge_chs_write = self.merge_chs.write().unwrap();
            merge_chs_write.insert(chan.clone());
        }
    }

    fn is_mergeable_channel(&self, chan: &Par) -> bool {
        let tuple_elms: Vec<Par> = chan
            .exprs
            .iter()
            .flat_map(|y| match &y.expr_instance {
                Some(expr_instance) => match expr_instance {
                    ExprInstance::ETupleBody(etuple) => etuple.ps.clone(),
                    _ => ETuple::default().ps,
                },
                None => ETuple::default().ps,
            })
            .collect();

        tuple_elms
            .first()
            .map_or(false, |head| head == &self.mergeable_tag_name)
    }

    fn aggregate_evaluator_errors(
        &self,
        errors: Vec<InterpreterError>,
    ) -> Result<(), InterpreterError> {
        match errors.as_slice() {
            // No errors
            [] => Ok(()),

            // Out Of Phlogiston error is always single
            // - if one execution path is out of phlo, the whole evaluation is also
            err_list
                if err_list
                    .iter()
                    .any(|e| matches!(e, InterpreterError::OutOfPhlogistonsError)) =>
            {
                Err(InterpreterError::OutOfPhlogistonsError)
            }

            // Rethrow single error
            [ex] => Err(ex.clone()),

            // Collect errors from parallel execution
            err_list => Err(InterpreterError::AggregateError {
                interpreter_errors: err_list.to_vec(),
            }),
        }
    }

    async fn generated_message_eval(
        &self,
        term: &GeneratedMessage,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        match term {
            GeneratedMessage::Send(term) => self.eval_send(term, env, rand).await,
            GeneratedMessage::Receive(term) => self.eval_receive(term, env, rand).await,
            GeneratedMessage::New(term) => self.eval_new(term, env.clone(), rand).await,
            GeneratedMessage::Match(term) => self.eval_match(term, env, rand).await,
            GeneratedMessage::Bundle(term) => self.eval_bundle(term, env, rand).await,
            GeneratedMessage::Expr(term) => match &term.expr_instance {
                Some(expr_instance) => match expr_instance {
                    ExprInstance::EVarBody(e) => {
                        let res = self.eval_var(&e.clone().v.unwrap(), &env)?;
                        self.eval(res, env, rand).await
                    }
                    ExprInstance::EMethodBody(e) => {
                        let res = self.eval_expr_to_par(
                            &Expr {
                                expr_instance: Some(ExprInstance::EMethodBody(e.clone())),
                            },
                            env,
                        )?;
                        self.eval(res, env, rand).await
                    }
                    other => Err(InterpreterError::BugFoundError(format!(
                        "Undefined term: {:?}",
                        other
                    ))),
                },
                None => Err(InterpreterError::BugFoundError(format!(
                    "Undefined term, expr_instance was None"
                ))),
            },
        }
    }

    /** Algorithm as follows:
     *
     * 1. Fully evaluate the channel in given environment.
     * 2. Substitute any variable references in the channel so that it can be
     *    correctly used as a key in the tuple space.
     * 3. Evaluate any top level expressions in the data being sent.
     * 4. Call produce
     *
     * @param send An output process
     * @param env An execution context
     *
     */
    async fn eval_send(
        &self,
        send: &Send,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        self.cost.charge(send_eval_cost())?;
        let eval_chan = self.eval_expr(&send.chan.as_ref().unwrap(), env)?;
        let sub_chan = self.substitute.substitute_and_charge(&eval_chan, 0, env)?;
        let unbundled = match single_bundle(&sub_chan) {
            Some(value) => {
                if !value.write_flag {
                    return Err(InterpreterError::ReduceError(
                        "Trying to send on non-writeable channel.".to_string(),
                    ));
                } else {
                    value.body.unwrap()
                }
            }
            None => sub_chan,
        };

        let data = send
            .data
            .iter()
            .map(|expr| self.eval_expr(expr, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let subst_data = data
            .into_iter()
            .map(|p| self.substitute.substitute_and_charge(&p, 0, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        self.produce(
            unbundled,
            ListParWithRandom {
                pars: subst_data,
                random_state: rand.to_vec(),
            },
            send.persistent,
        )
        .await
    }

    async fn eval_receive(
        &self,
        receive: &Receive,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        self.cost.charge(receive_eval_cost())?;
        let binds = receive
            .binds
            .clone()
            .into_iter()
            .map(|rb| {
                let q = self.unbundle_receive(&rb, env)?;
                let subst_patterns = rb
                    .patterns
                    .into_iter()
                    .map(|pattern| self.substitute.substitute_and_charge(&pattern, 1, env))
                    .collect::<Result<Vec<_>, InterpreterError>>()?;

                Ok((
                    BindPattern {
                        patterns: subst_patterns,
                        remainder: rb.remainder,
                        free_count: rb.free_count,
                    },
                    q,
                ))
            })
            .collect::<Result<Vec<_>, InterpreterError>>()?;

        let subst_body = self.substitute.substitute_no_sort_and_charge(
            receive.body.as_ref().unwrap(),
            0,
            &env.shift(receive.bind_count),
        )?;

        self.consume(
            binds,
            ParWithRandom {
                body: Some(subst_body),
                random_state: rand.to_vec(),
            },
            receive.persistent,
            receive.peek,
        )
        .await
    }

    /**
     * Variable "evaluation" is an environment lookup, but
     * lookup of an unbound variable should be an error.
     *
     * @param valproc The variable to be evaluated
     * @param env  provides the environment (possibly) containing a binding for the given variable.
     * @return If the variable has a binding (par), lift the
     *                  binding into the monadic context, else signal
     *                  an exception.
     */
    fn eval_var(&self, valproc: &Var, env: &Env<Par>) -> Result<Par, InterpreterError> {
        self.cost.charge(var_eval_cost())?;
        match valproc.var_instance {
            Some(VarInstance::BoundVar(level)) => Err(InterpreterError::ReduceError(format!(
                "Unbound variable: {} in {:#?}",
                level, env.env_map
            ))),
            Some(VarInstance::Wildcard(_)) => Err(InterpreterError::ReduceError(
                "Unbound variable: attempting to evaluate a pattern".to_string(),
            )),
            Some(VarInstance::FreeVar(_)) => Err(InterpreterError::ReduceError(
                "Unbound variable: attempting to evaluate a pattern".to_string(),
            )),
            None => Err(InterpreterError::ReduceError(
                "Impossible var instance EMPTY".to_string(),
            )),
        }
    }

    // TODO: review 'loop' matches 'tailRecM'
    async fn eval_match(
        &self,
        mat: &Match,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        fn add_to_env(env: &Env<Par>, free_map: BTreeMap<i32, Par>, free_count: i32) -> Env<Par> {
            (0..free_count).fold(env.clone(), |mut acc, e| {
                let value = free_map.get(&e).unwrap_or(&Par::default()).clone();
                acc.put(value)
            })
        }

        let first_match = Box::new(
            |target: Par, cases: Vec<MatchCase>, rand: Blake2b512Random| async {
                let mut state = (target, cases);

                loop {
                    let (_target, _cases) = state;

                    match _cases.as_slice() {
                        [] => return Ok(()),

                        [single_case, case_rem @ ..] => {
                            let pattern = self.substitute.substitute_and_charge(
                                single_case.pattern.as_ref().unwrap(),
                                1,
                                env,
                            )?;

                            let mut spatial_matcher = SpatialMatcherContext::new();

                            let match_result =
                                spatial_matcher.spatial_match_result(_target.clone(), pattern);

                            match match_result {
                                None => {
                                    state = (_target, case_rem.to_vec());
                                }

                                Some(free_map) => {
                                    let eval_result = self
                                        .eval(
                                            single_case.source.clone().unwrap(),
                                            &add_to_env(
                                                env,
                                                free_map.clone(),
                                                single_case.free_count,
                                            ),
                                            rand,
                                        )
                                        .await?;

                                    return Ok(eval_result);
                                }
                            }
                        }
                    }
                }
            },
        );

        self.cost.charge(match_eval_cost())?;
        let evaled_target = self.eval_expr(&mat.target.as_ref().unwrap(), env)?;
        let subst_target = self
            .substitute
            .substitute_and_charge(&evaled_target, 0, env)?;

        first_match(subst_target, mat.cases.clone(), rand).await
    }

    /**
     * Adds neu.bindCount new GPrivate from UUID's to the environment and then
     * proceeds to evaluate the body.
     */
    async fn eval_new(
        &self,
        new: &New,
        env: Env<Par>,
        mut rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        let mut alloc = |count: usize, urns: Vec<String>| {
            let simple_news =
                (0..(count - urns.len()))
                    .into_iter()
                    .fold(env.clone(), |mut _env: Env<Par>, _| {
                        let addr: Par = Par::default().with_unforgeables(vec![GUnforgeable {
                            unf_instance: Some(UnfInstance::GPrivateBody(GPrivate {
                                id: ByteString::from(rand.next()),
                            })),
                        }]);
                        _env.put(addr)
                    });

            let add_urn = |new_env: &mut Env<Par>, urn: String| {
                if !self.urn_map.contains_key(&urn) {
                    /** TODO: Injections (from normalizer) are not used currently, see [[NormalizerEnv]]. */
                    // If `urn` can't be found in `urnMap`, it must be referencing an injection
                    match new.injections.get(&urn) {
                      Some(p) => {
                        if let Some(gunf) = RhoUnforgeable::unapply(p) {
                          if let Some(instance) = gunf.unf_instance {
                              Either::Right(new_env.put(Par::default().with_unforgeables(vec![GUnforgeable {unf_instance: Some(instance)}])))
                          } else {
                               Either::Left(InterpreterError::BugFoundError("unf_instance field is None".to_string()))
                          }
                      } else if let Some(expr) = RhoExpression::unapply(p) {
                          if let Some(instance) = expr.expr_instance {
                              Either::Right(new_env.put(Par::default().with_exprs(vec![Expr {expr_instance: Some(instance)}])))
                          } else {
                               Either::Left(InterpreterError::BugFoundError("expr_instance field is None".to_string()))
                          }
                      } else {
                          Either::Left(InterpreterError::BugFoundError("invalid injection".to_string()))
                      }
                      },
                      None => {
                        Either::Left(InterpreterError::BugFoundError(format!("No value set for {}. This is a bug in the normalizer or on the path from it.", urn)))
                      },
                    }
                } else {
                    match self.urn_map.get(&urn) {
                        Some(p) => Either::Right(new_env.put(p.clone())),
                        None => Either::Left(InterpreterError::ReduceError(format!(
                            "Unknown urn for new: {}",
                            urn
                        ))),
                    }
                }
            };

            urns.into_iter()
                .fold(Ok(simple_news), |acc, urn| match acc {
                    Ok(mut news) => match add_urn(&mut news, urn) {
                        Either::Left(err) => Err(err),
                        Either::Right(env) => Ok(env),
                    },

                    Err(e) => Err(e),
                })
        };

        self.cost.charge(new_bindings_cost(new.bind_count as i64))?;
        match alloc(new.bind_count as usize, new.uri.clone()) {
            Ok(env) => self.eval(new.p.clone().unwrap(), &env, rand).await,
            Err(e) => Err(e),
        }
    }

    fn unbundle_receive(&self, rb: &ReceiveBind, env: &Env<Par>) -> Result<Par, InterpreterError> {
        let eval_src = self.eval_expr(&rb.source.as_ref().unwrap(), env)?;
        let subst = self.substitute.substitute_and_charge(&eval_src, 0, env)?;
        // Check if we try to read from bundled channel
        let unbndl = match single_bundle(&subst) {
            Some(value) => {
                if !value.write_flag {
                    return Err(InterpreterError::ReduceError(
                        "Trying to read on non-readable channel.".to_string(),
                    ));
                } else {
                    value.body.unwrap()
                }
            }
            None => subst,
        };

        Ok(unbndl)
    }

    async fn eval_bundle(
        &self,
        bundle: &Bundle,
        env: &Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        self.eval(bundle.body.clone().unwrap(), env, rand).await
    }

    fn eval_expr_to_par(&self, expr: &Expr, env: &Env<Par>) -> Result<Par, InterpreterError> {
        match expr.expr_instance.clone().unwrap() {
            ExprInstance::EVarBody(evar) => {
                let p = self.eval_var(&evar.v.unwrap(), env)?;
                let evaled_p = self.eval_expr(&p, env)?;
                Ok(evaled_p)
            }
            ExprInstance::EMethodBody(emethod) => {
                self.cost.charge(method_call_cost())?;
                let evaled_target = self.eval_expr(&emethod.target.unwrap(), env)?;
                let evaled_args: Vec<Par> = emethod
                    .arguments
                    .iter()
                    .map(|arg| self.eval_expr(arg, env))
                    .collect::<Result<Vec<_>, InterpreterError>>()?;

                let result_par = match self.method_table().get(&emethod.method_name) {
                    Some(_method) => _method.apply(evaled_target, evaled_args)?,
                    None => {
                        return Err(InterpreterError::ReduceError(format!(
                            "Unimplemented method: {}",
                            emethod.method_name
                        )))
                    }
                };

                Ok(result_par)
            }
            _ => Ok(Par::default().with_exprs(vec![self.eval_expr_to_expr(expr, env)?])),
        }
    }

    fn eval_expr_to_expr(&self, expr: &Expr, env: &Env<Par>) -> Result<Expr, InterpreterError> {
        let relop = |p1: &Par,
                     p2: &Par,
                     relopb: fn(bool, bool) -> bool,
                     relopi: fn(i64, i64) -> bool,
                     relops: fn(String, String) -> bool| {
            let v1 = self.eval_single_expr(p1, env)?;
            let v2 = self.eval_single_expr(p2, env)?;

            match (
                v1.expr_instance.clone().unwrap(),
                v2.expr_instance.clone().unwrap(),
            ) {
                (ExprInstance::GBool(b1), ExprInstance::GBool(b2)) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GBool(relopb(b1, b2))),
                }),

                (ExprInstance::GInt(i1), ExprInstance::GInt(i2)) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GBool(relopi(i1, i2))),
                }),

                (ExprInstance::GString(s1), ExprInstance::GString(s2)) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GBool(relops(s1, s2))),
                }),

                _ => Err(InterpreterError::ReduceError(format!(
                    "Unexpected compare: {:?} vs. {:?}",
                    v1, v2
                ))),
            }
        };

        match &expr.expr_instance {
            Some(expr_instance) => match expr_instance {
                ExprInstance::GBool(x) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GBool(*x)),
                }),
                ExprInstance::GInt(x) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GInt(*x)),
                }),
                ExprInstance::GString(x) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GString(x.clone())),
                }),
                ExprInstance::GUri(x) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GUri(x.clone())),
                }),
                ExprInstance::GByteArray(x) => Ok(Expr {
                    expr_instance: Some(ExprInstance::GByteArray(x.clone())),
                }),

                ExprInstance::ENotBody(enot) => {
                    let b = self.eval_to_bool(&enot.p.as_ref().unwrap(), env)?;
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GBool(!b)),
                    })
                }
                ExprInstance::ENegBody(eneg) => {
                    let v = self.eval_to_i64(&eneg.p.as_ref().unwrap(), env)?;
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GInt(-v)),
                    })
                }

                ExprInstance::EMultBody(_) => todo!(),
                ExprInstance::EDivBody(_) => todo!(),
                ExprInstance::EPlusBody(_) => todo!(),
                ExprInstance::EMinusBody(_) => todo!(),
                ExprInstance::ELtBody(_) => todo!(),
                ExprInstance::ELteBody(_) => todo!(),
                ExprInstance::EGtBody(_) => todo!(),
                ExprInstance::EGteBody(_) => todo!(),
                ExprInstance::EEqBody(_) => todo!(),
                ExprInstance::ENeqBody(_) => todo!(),
                ExprInstance::EAndBody(_) => todo!(),
                ExprInstance::EOrBody(_) => todo!(),
                ExprInstance::EVarBody(_) => todo!(),
                ExprInstance::EListBody(_) => todo!(),
                ExprInstance::ETupleBody(_) => todo!(),
                ExprInstance::ESetBody(_) => todo!(),
                ExprInstance::EMapBody(_) => todo!(),
                ExprInstance::EMethodBody(_) => todo!(),
                ExprInstance::EMatchesBody(_) => todo!(),
                ExprInstance::EPercentPercentBody(_) => todo!(),
                ExprInstance::EPlusPlusBody(_) => todo!(),
                ExprInstance::EMinusMinusBody(_) => todo!(),
                ExprInstance::EModBody(_) => todo!(),
            },
            None => Err(InterpreterError::ReduceError(format!(
                "Unimplemented expression: {:?}",
                expr
            ))),
        }
    }

    fn method_table(&self) -> HashMap<String, impl Method> {
        let mut table = HashMap::new();
        table.insert("nth".to_string(), NthMethod);
        table
    }

    fn eval_single_expr(&self, p: &Par, env: &Env<Par>) -> Result<Expr, InterpreterError> {
        todo!()
    }

    fn eval_to_i64(&self, p: &Par, env: &Env<Par>) -> Result<i64, InterpreterError> {
        todo!()
    }

    fn eval_to_bool(&self, p: &Par, env: &Env<Par>) -> Result<bool, InterpreterError> {
        todo!()
    }

    /**
     * Evaluate any top level expressions in @param Par .
     */
    fn eval_expr(&self, par: &Par, env: &Env<Par>) -> Result<Par, InterpreterError> {
        let evaled_exprs = par
            .exprs
            .iter()
            .map(|expr| self.eval_expr_to_par(expr, env))
            .collect::<Result<Vec<_>, InterpreterError>>()?;
        // Note: the locallyFree cache in par could now be invalid, but given
        // that locallyFree is for use in the matcher, and the matcher uses
        // substitution, it will resolve in that case. AlwaysEqual makes sure
        // that this isn't an issue in the rest of cases.
        let result = evaled_exprs
            .into_iter()
            .fold(par.with_exprs(Vec::new()), |acc, expr| {
                // acc.exprs.iter().chain(expr.exprs.iter()).cloned().collect()
                concatenate_pars(acc, expr)
            });

        Ok(result)
    }
}

trait Method {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError>;
}

struct NthMethod;

impl Method for NthMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct ToByteArrayMethod;

impl Method for ToByteArrayMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct HexToBytesMethod;

impl Method for HexToBytesMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct BytesToHexMethod;

impl Method for BytesToHexMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct ToUtf8BytesMethod;

impl Method for ToUtf8BytesMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct UnionMethod;

impl Method for UnionMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct DiffMethod;

impl Method for DiffMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct AddMethod;

impl Method for AddMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct DeleteMethod;

impl Method for DeleteMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct ContainsMethod;

impl Method for ContainsMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct GetMethod;

impl Method for GetMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct GetOrElseMethod;

impl Method for GetOrElseMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct SetMethod;

impl Method for SetMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct KeysMethod;

impl Method for KeysMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct SizeMethod;

impl Method for SizeMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct LengthMethod;

impl Method for LengthMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct SliceMethod;

impl Method for SliceMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct TakeMethod;

impl Method for TakeMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct ToListMethod;

impl Method for ToListMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct ToSetMethod;

impl Method for ToSetMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}

struct ToMapMethod;

impl Method for ToMapMethod {
    fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
        todo!()
    }
}
