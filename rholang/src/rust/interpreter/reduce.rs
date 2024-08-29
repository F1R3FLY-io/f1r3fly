// See See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::g_unforgeable::UnfInstance;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::var::VarInstance;
use models::rhoapi::{
    BindPattern, Bundle, EAnd, EDiv, EEq, EGt, EGte, EList, ELt, ELte, EMap, EMatches, EMinus,
    EMinusMinus, EMod, EMult, ENeq, EOr, EPercentPercent, EPlus, EPlusPlus, EVar, Expr, GPrivate,
    GUnforgeable, Match, MatchCase, New, ParWithRandom, Receive, ReceiveBind, Send, Var,
};
use models::rhoapi::{ETuple, ListParWithRandom, Par, TaggedContinuation};
use models::rust::rholang::implicits::{concatenate_pars, single_bundle};
use models::rust::utils::union;
use models::ByteString;
use rspace_plus_plus::rspace::history::Either;
use rspace_plus_plus::rspace::util::unpack_option_with_peek;
use std::any::{type_name, Any};
use std::collections::{BTreeMap, BTreeSet};
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use crate::rust::interpreter::accounting::costs::{interpolate_cost, match_eval_cost};
use crate::rust::interpreter::matcher::spatial_matcher::SpatialMatcherContext;

use super::accounting::_cost;
use super::accounting::costs::{
    boolean_and_cost, boolean_or_cost, byte_array_append_cost, comparison_cost, division_cost,
    equality_check_cost, list_append_cost, method_call_cost, modulo_cost, multiplication_cost,
    new_bindings_cost, op_call_cost, receive_eval_cost, send_eval_cost, string_append_cost,
    subtraction_cost, sum_cost, var_eval_cost,
};
use super::errors::InterpreterError;
use super::matcher::has_locally_free::HasLocallyFree;
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

                ExprInstance::EMultBody(EMult { p1, p2 }) => {
                    let v1 = self.eval_to_i64(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_to_i64(&p2.clone().unwrap(), env)?;
                    self.cost.charge(multiplication_cost())?;
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GInt(v1 * v2)),
                    })
                }

                ExprInstance::EDivBody(EDiv { p1, p2 }) => {
                    let v1 = self.eval_to_i64(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_to_i64(&p2.clone().unwrap(), env)?;
                    self.cost.charge(division_cost())?;
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GInt(v1 / v2)),
                    })
                }

                ExprInstance::EModBody(EMod { p1, p2 }) => {
                    let v1 = self.eval_to_i64(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_to_i64(&p2.clone().unwrap(), env)?;
                    self.cost.charge(modulo_cost())?;
                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GInt(v1 % v2)),
                    })
                }

                ExprInstance::EPlusBody(EPlus { p1, p2 }) => {
                    let v1 = self.eval_single_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_single_expr(&p2.clone().unwrap(), env)?;

                    match (v1.expr_instance.unwrap(), v2.expr_instance.unwrap()) {
                        (ExprInstance::GInt(lhs), ExprInstance::GInt(rhs)) => {
                            self.cost.charge(sum_cost())?;
                            Ok(Expr {
                                expr_instance: Some(ExprInstance::GInt(lhs + rhs)),
                            })
                        }

                        (ExprInstance::ESetBody(lhs), rhs) => {
                            self.cost.charge(op_call_cost())?;
                            let result_par = self.add_method().apply(
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::ESetBody(lhs)),
                                }]),
                                vec![Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(rhs),
                                }])],
                            )?;

                            let result_expr = self.eval_single_expr(&result_par, env)?;
                            Ok(result_expr)
                        }

                        (ExprInstance::GInt(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "+".to_string(),
                                expected: "Int".to_string(),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (other, _) => Err(InterpreterError::OperatorNotDefined {
                            op: "+".to_string(),
                            other_type: format!("{:?}", other.type_id()),
                        }),
                    }
                }

                ExprInstance::EMinusBody(EMinus { p1, p2 }) => {
                    let v1 = self.eval_single_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_single_expr(&p2.clone().unwrap(), env)?;

                    match (v1.expr_instance.unwrap(), v2.expr_instance.unwrap()) {
                        (ExprInstance::GInt(lhs), ExprInstance::GInt(rhs)) => {
                            self.cost.charge(subtraction_cost())?;
                            Ok(Expr {
                                expr_instance: Some(ExprInstance::GInt(lhs - rhs)),
                            })
                        }

                        (ExprInstance::EMapBody(lhs), rhs) => {
                            self.cost.charge(op_call_cost())?;
                            let result_par = self.delete_method().apply(
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::EMapBody(lhs)),
                                }]),
                                vec![Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(rhs),
                                }])],
                            )?;

                            let result_expr = self.eval_single_expr(&result_par, env)?;
                            Ok(result_expr)
                        }

                        (ExprInstance::ESetBody(lhs), rhs) => {
                            self.cost.charge(op_call_cost())?;
                            let result_par = self.delete_method().apply(
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::ESetBody(lhs)),
                                }]),
                                vec![Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(rhs),
                                }])],
                            )?;

                            let result_expr = self.eval_single_expr(&result_par, env)?;
                            Ok(result_expr)
                        }

                        (ExprInstance::GInt(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "-".to_string(),
                                expected: "Int".to_string(),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (other, _) => Err(InterpreterError::OperatorNotDefined {
                            op: "-".to_string(),
                            other_type: format!("{:?}", other.type_id()),
                        }),
                    }
                }

                ExprInstance::ELtBody(ELt { p1, p2 }) => {
                    self.cost.charge(comparison_cost())?;
                    relop(
                        &p1.clone().unwrap(),
                        &p2.clone().unwrap(),
                        |b1: bool, b2: bool| b1 < b2,
                        |i1: i64, i2: i64| i1 < i2,
                        |s1: String, s2: String| s1 < s2,
                    )
                }

                ExprInstance::ELteBody(ELte { p1, p2 }) => {
                    self.cost.charge(comparison_cost())?;
                    relop(
                        &p1.clone().unwrap(),
                        &p2.clone().unwrap(),
                        |b1: bool, b2: bool| b1 <= b2,
                        |i1: i64, i2: i64| i1 <= i2,
                        |s1: String, s2: String| s1 <= s2,
                    )
                }

                ExprInstance::EGtBody(EGt { p1, p2 }) => {
                    self.cost.charge(comparison_cost())?;
                    relop(
                        &p1.clone().unwrap(),
                        &p2.clone().unwrap(),
                        |b1: bool, b2: bool| b1 > b2,
                        |i1: i64, i2: i64| i1 > i2,
                        |s1: String, s2: String| s1 > s2,
                    )
                }

                ExprInstance::EGteBody(EGte { p1, p2 }) => {
                    self.cost.charge(comparison_cost())?;
                    relop(
                        &p1.clone().unwrap(),
                        &p2.clone().unwrap(),
                        |b1: bool, b2: bool| b1 >= b2,
                        |i1: i64, i2: i64| i1 >= i2,
                        |s1: String, s2: String| s1 >= s2,
                    )
                }

                ExprInstance::EEqBody(EEq { p1, p2 }) => {
                    let v1 = self.eval_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_expr(&p2.clone().unwrap(), env)?;
                    // TODO: build an equality operator that takes in an environment. - OLD
                    let sv1 = self.substitute.substitute_and_charge(&v1, 0, env)?;
                    let sv2 = self.substitute.substitute_and_charge(&v2, 0, env)?;
                    self.cost.charge(equality_check_cost(&sv1, &sv2))?;

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GBool(sv1 == sv2)),
                    })
                }

                ExprInstance::ENeqBody(ENeq { p1, p2 }) => {
                    let v1 = self.eval_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_expr(&p2.clone().unwrap(), env)?;
                    let sv1 = self.substitute.substitute_and_charge(&v1, 0, env)?;
                    let sv2 = self.substitute.substitute_and_charge(&v2, 0, env)?;
                    self.cost.charge(equality_check_cost(&sv1, &sv2))?;

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GBool(sv1 != sv2)),
                    })
                }

                ExprInstance::EAndBody(EAnd { p1, p2 }) => {
                    let b1 = self.eval_to_bool(&p1.clone().unwrap(), env)?;
                    let b2 = self.eval_to_bool(&p2.clone().unwrap(), env)?;
                    self.cost.charge(boolean_and_cost())?;

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GBool(b1 && b2)),
                    })
                }

                ExprInstance::EOrBody(EOr { p1, p2 }) => {
                    let b1 = self.eval_to_bool(&p1.clone().unwrap(), env)?;
                    let b2 = self.eval_to_bool(&p2.clone().unwrap(), env)?;
                    self.cost.charge(boolean_or_cost())?;

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GBool(b1 || b2)),
                    })
                }

                ExprInstance::EMatchesBody(EMatches { target, pattern }) => {
                    let evaled_target = self.eval_expr(&target.clone().unwrap(), env)?;
                    let subst_target =
                        self.substitute
                            .substitute_and_charge(&evaled_target, 0, env)?;
                    let subst_pattern =
                        self.substitute
                            .substitute_and_charge(&pattern.clone().unwrap(), 1, env)?;

                    let mut spatial_matcher = SpatialMatcherContext::new();
                    let match_result =
                        spatial_matcher.spatial_match_result(subst_target, subst_pattern);

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::GBool(match_result.is_some())),
                    })
                }

                ExprInstance::EPercentPercentBody(EPercentPercent { p1, p2 }) => {
                    fn eval_to_string_pair(
                        key_expr: Expr,
                        value_expr: Expr,
                    ) -> Result<(String, String), InterpreterError> {
                        match (
                            key_expr.expr_instance.unwrap(),
                            value_expr.expr_instance.unwrap(),
                        ) {
                            (
                                ExprInstance::GString(key_string),
                                ExprInstance::GString(value_string),
                            ) => Ok((key_string, value_string)),

                            (ExprInstance::GString(key_string), ExprInstance::GInt(value_int)) => {
                                Ok((key_string, value_int.to_string()))
                            }

                            (
                                ExprInstance::GString(key_string),
                                ExprInstance::GBool(value_bool),
                            ) => Ok((key_string, value_bool.to_string())),

                            (ExprInstance::GString(key_string), ExprInstance::GUri(uri)) => {
                                Ok((key_string, uri))
                            }

                            // TODO: Add cases for other ground terms as well? Maybe it would be better
                            // to implement cats.Show for all ground terms. - OLD
                            (ExprInstance::GString(_), value) => {
                                Err(InterpreterError::ReduceError(format!(
                                    "Error: interpolation doesn't support {:?}",
                                    value.type_id()
                                )))
                            }

                            _ => Err(InterpreterError::ReduceError(format!(
                                "Error: interpolation Map should only contain String keys"
                            ))),
                        }
                    }

                    fn interpolate(string: &str, key_value_pairs: &[(String, String)]) -> String {
                        let mut result = String::new();
                        let mut current = string.to_string();

                        while !current.is_empty() {
                            let mut found = false;

                            for (k, v) in key_value_pairs {
                                if current.starts_with(&format!("${{{}}}", k)) {
                                    result.push_str(v);
                                    current = current.split_at(k.len() + 3).1.to_string();
                                    found = true;

                                    break;
                                }
                            }

                            if !found {
                                result.push(current.chars().next().unwrap());
                                current.remove(0);
                            }
                        }

                        result
                    }

                    self.cost.charge(op_call_cost())?;
                    let v1 = self.eval_single_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_single_expr(&p2.clone().unwrap(), env)?;

                    match (v1.expr_instance.unwrap(), v2.expr_instance.unwrap()) {
                        (ExprInstance::GString(lhs), ExprInstance::EMapBody(EMap { kvs, .. })) => {
                            if !lhs.is_empty() || !kvs.is_empty() {
                                let key_value_pairs = kvs
                                    .clone()
                                    .into_iter()
                                    .map(|kv| {
                                        let key_expr =
                                            self.eval_single_expr(&kv.key.unwrap(), env)?;
                                        let value_expr =
                                            self.eval_single_expr(&kv.value.unwrap(), env)?;
                                        let result = eval_to_string_pair(key_expr, value_expr)?;
                                        Ok(result)
                                    })
                                    .collect::<Result<Vec<_>, InterpreterError>>()?;

                                self.cost
                                    .charge(interpolate_cost(lhs.len() as i64, kvs.len() as i64))?;
                                Ok(Expr {
                                    expr_instance: Some(ExprInstance::GString(interpolate(
                                        &lhs,
                                        &key_value_pairs,
                                    ))),
                                })
                            } else {
                                Ok(Expr {
                                    expr_instance: Some(ExprInstance::GString(lhs)),
                                })
                            }
                        }

                        (ExprInstance::GString(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "%%".to_string(),
                                expected: String::from("Map"),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (other, _) => Err(InterpreterError::OperatorNotDefined {
                            op: String::from("%%"),
                            other_type: format!("{:?}", other.type_id()),
                        }),
                    }
                }

                ExprInstance::EPlusPlusBody(EPlusPlus { p1, p2 }) => {
                    self.cost.charge(op_call_cost())?;
                    let v1 = self.eval_single_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_single_expr(&p2.clone().unwrap(), env)?;

                    match (v1.expr_instance.unwrap(), v2.expr_instance.unwrap()) {
                        (ExprInstance::GString(lhs), ExprInstance::GString(rhs)) => {
                            self.cost
                                .charge(string_append_cost(lhs.len() as i64, rhs.len() as i64))?;
                            Ok(Expr {
                                expr_instance: Some(ExprInstance::GString(lhs + &rhs)),
                            })
                        }

                        (ExprInstance::GByteArray(lhs), ExprInstance::GByteArray(rhs)) => {
                            self.cost.charge(byte_array_append_cost(lhs.clone()))?;
                            Ok(Expr {
                                expr_instance: Some(ExprInstance::GByteArray(
                                    lhs.into_iter().chain(rhs.into_iter()).collect(),
                                )),
                            })
                        }

                        (ExprInstance::EListBody(lhs), ExprInstance::EListBody(rhs)) => {
                            self.cost.charge(list_append_cost(lhs.clone().ps))?;
                            Ok(Expr {
                                expr_instance: Some(ExprInstance::EListBody(EList {
                                    ps: lhs.ps.into_iter().chain(rhs.ps.into_iter()).collect(),
                                    locally_free: union(lhs.locally_free, rhs.locally_free),
                                    connective_used: lhs.connective_used || rhs.connective_used,
                                    remainder: None,
                                })),
                            })
                        }

                        (ExprInstance::EMapBody(lhs), ExprInstance::EMapBody(rhs)) => {
                            let result_par = self.union_method().apply(
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::EMapBody(lhs)),
                                }]),
                                vec![Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::EMapBody(rhs)),
                                }])],
                            )?;
                            let result_expr = self.eval_single_expr(&result_par, env)?;
                            Ok(result_expr)
                        }

                        (ExprInstance::ESetBody(lhs), ExprInstance::ESetBody(rhs)) => {
                            let result_par = self.union_method().apply(
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::ESetBody(lhs)),
                                }]),
                                vec![Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::ESetBody(rhs)),
                                }])],
                            )?;
                            let result_expr = self.eval_single_expr(&result_par, env)?;
                            Ok(result_expr)
                        }

                        (ExprInstance::GString(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "++".to_string(),
                                expected: String::from("String"),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (ExprInstance::EListBody(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "++".to_string(),
                                expected: String::from("List"),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (ExprInstance::EMapBody(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "++".to_string(),
                                expected: String::from("Map"),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (ExprInstance::ESetBody(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "++".to_string(),
                                expected: String::from("Set"),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (other, _) => Err(InterpreterError::OperatorNotDefined {
                            op: String::from("++"),
                            other_type: format!("{:?}", other.type_id()),
                        }),
                    }
                }

                ExprInstance::EMinusMinusBody(EMinusMinus { p1, p2 }) => {
                    self.cost.charge(op_call_cost())?;
                    let v1 = self.eval_single_expr(&p1.clone().unwrap(), env)?;
                    let v2 = self.eval_single_expr(&p2.clone().unwrap(), env)?;

                    match (v1.expr_instance.unwrap(), v2.expr_instance.unwrap()) {
                        (ExprInstance::ESetBody(lhs), ExprInstance::ESetBody(rhs)) => {
                            let result_par = self.diff_method().apply(
                                Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::ESetBody(lhs)),
                                }]),
                                vec![Par::default().with_exprs(vec![Expr {
                                    expr_instance: Some(ExprInstance::ESetBody(rhs)),
                                }])],
                            )?;
                            let result_expr = self.eval_single_expr(&result_par, env)?;
                            Ok(result_expr)
                        }

                        (ExprInstance::ESetBody(_), other) => {
                            Err(InterpreterError::OperatorExpectedError {
                                op: "--".to_string(),
                                expected: String::from("Set"),
                                other_type: format!("{:?}", other.type_id()),
                            })
                        }

                        (other, _) => Err(InterpreterError::OperatorNotDefined {
                            op: String::from("--"),
                            other_type: format!("{:?}", other.type_id()),
                        }),
                    }
                }

                ExprInstance::EVarBody(EVar { v }) => {
                    let p = self.eval_var(&v.clone().unwrap(), env)?;
                    let expr_val = self.eval_single_expr(&p, env)?;
                    Ok(expr_val)
                }

                ExprInstance::EListBody(e1) => {
                    let evaled_ps = e1
                        .ps
                        .iter()
                        .map(|p| self.eval_expr(p, env))
                        .collect::<Result<Vec<_>, InterpreterError>>()?;

                    let updated_ps: Vec<Par> = evaled_ps
                        .iter()
                        .map(|p| self.update_locally_free_par(p.clone()))
                        .collect();

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::EListBody(
                            self.update_locally_free_elist(EList {
                                ps: updated_ps,
                                locally_free: e1.locally_free.clone(),
                                connective_used: e1.connective_used,
                                remainder: None,
                            }),
                        )),
                    })
                }

                ExprInstance::ETupleBody(e1) => {
                    let evaled_ps = e1
                        .ps
                        .iter()
                        .map(|p| self.eval_expr(p, env))
                        .collect::<Result<Vec<_>, InterpreterError>>()?;

                    let updated_ps: Vec<Par> = evaled_ps
                        .iter()
                        .map(|p| self.update_locally_free_par(p.clone()))
                        .collect();

                    Ok(Expr {
                        expr_instance: Some(ExprInstance::ETupleBody(
                            self.update_locally_free_etuple(ETuple {
                                ps: updated_ps,
                                locally_free: e1.locally_free.clone(),
                                connective_used: e1.connective_used,
                            }),
                        )),
                    })
                }

                ExprInstance::ESetBody(set) => {
                    let evaled_ps = set
                        .ps
                        .iter()
                        .map(|p| self.eval_expr(p, env))
                        .collect::<Result<Vec<_>, InterpreterError>>()?;

                    let updated_ps: Vec<Par> = evaled_ps
                        .iter()
                        .map(|p| self.update_locally_free_par(p.clone()))
                        .collect();

                    todo!()
                }

                ExprInstance::EMapBody(_) => todo!(),
                ExprInstance::EMethodBody(_) => todo!(),
            },
            None => Err(InterpreterError::ReduceError(format!(
                "Unimplemented expression: {:?}",
                expr
            ))),
        }
    }

    fn nth_method(&self) -> impl Method {
        struct NthMethod;

        impl Method for NthMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        NthMethod
    }

    fn to_byte_array_method(&self) -> impl Method {
        struct ToByteArrayMethod;

        impl Method for ToByteArrayMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        ToByteArrayMethod
    }

    fn hex_to_bytes_method(&self) -> impl Method {
        struct HexToBytesMethod;

        impl Method for HexToBytesMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        HexToBytesMethod
    }

    fn bytes_to_hex_method(&self) -> impl Method {
        struct BytesToHexMethod;

        impl Method for BytesToHexMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        BytesToHexMethod
    }

    fn to_utf8_bytes_method(&self) -> impl Method {
        struct ToUtf8BytesMethod;

        impl Method for ToUtf8BytesMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        ToUtf8BytesMethod
    }

    fn union_method(&self) -> impl Method {
        struct UnionMethod;

        impl Method for UnionMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        UnionMethod
    }

    fn diff_method(&self) -> impl Method {
        struct DiffMethod;

        impl Method for DiffMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        DiffMethod
    }

    fn add_method(&self) -> impl Method {
        struct AddMethod;

        impl Method for AddMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        AddMethod
    }

    fn delete_method(&self) -> impl Method {
        struct NthMethod;

        impl Method for NthMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        NthMethod
    }

    fn contains_method(&self) -> impl Method {
        struct ContainsMethod;

        impl Method for ContainsMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        ContainsMethod
    }

    fn get_method(&self) -> impl Method {
        struct GetMethod;

        impl Method for GetMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        GetMethod
    }

    fn get_or_else_method(&self) -> impl Method {
        struct GetOrElseMethod;

        impl Method for GetOrElseMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        GetOrElseMethod
    }

    fn set_method(&self) -> impl Method {
        struct SetMethod;

        impl Method for SetMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        SetMethod
    }

    fn keys_method(&self) -> impl Method {
        struct KeysMethod;

        impl Method for KeysMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        KeysMethod
    }

    fn size_method(&self) -> impl Method {
        struct SizeMethod;

        impl Method for SizeMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        SizeMethod
    }

    fn length_method(&self) -> impl Method {
        struct LengthMethod;

        impl Method for LengthMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        LengthMethod
    }

    fn slice_method(&self) -> impl Method {
        struct SliceMethod;

        impl Method for SliceMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        SliceMethod
    }

    fn take_method(&self) -> impl Method {
        struct TakeMethod;

        impl Method for TakeMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        TakeMethod
    }

    fn to_list_method(&self) -> impl Method {
        struct ToListMethod;

        impl Method for ToListMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        ToListMethod
    }

    fn to_set_method(&self) -> impl Method {
        struct ToSetMethod;

        impl Method for ToSetMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        ToSetMethod
    }

    fn to_map_method(&self) -> impl Method {
        struct ToMapMethod;

        impl Method for ToMapMethod {
            fn apply(&self, p: Par, args: Vec<Par>) -> Result<Par, InterpreterError> {
                todo!()
            }
        }

        ToMapMethod
    }

    fn method_table(&self) -> HashMap<String, impl Method> {
        let mut table = HashMap::new();
        table.insert("nth".to_string(), self.nth_method());
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

    fn update_locally_free_par(&self, mut par: Par) -> Par {
        let mut locally_free = Vec::new();

        locally_free = union(
            locally_free,
            par.sends
                .iter()
                .flat_map(|send| send.locally_free.clone())
                .collect(),
        );

        locally_free = union(
            locally_free,
            par.receives
                .iter()
                .flat_map(|receive| receive.locally_free.clone())
                .collect(),
        );

        locally_free = union(
            locally_free,
            par.news
                .iter()
                .flat_map(|new_proc| new_proc.locally_free.clone())
                .collect(),
        );

        locally_free = union(
            locally_free,
            par.exprs
                .iter()
                .flat_map(|expr| expr.locally_free(expr.clone(), 0))
                .collect(),
        );

        locally_free = union(
            locally_free,
            par.matches
                .iter()
                .flat_map(|match_proc| match_proc.locally_free.clone())
                .collect(),
        );

        locally_free = union(
            locally_free,
            par.bundles
                .iter()
                .flat_map(|bundle_proc| bundle_proc.body.clone().unwrap().locally_free.clone())
                .collect(),
        );

        par.locally_free = locally_free;
        par
    }

    fn update_locally_free_elist(&self, mut elist: EList) -> EList {
        elist.locally_free = elist
            .ps
            .iter()
            .map(|p| p.locally_free.clone())
            .fold(Vec::new(), |acc, locally_free| union(acc, locally_free));

        elist
    }

    fn update_locally_free_etuple(&self, mut etuple: ETuple) -> ETuple {
        etuple.locally_free = etuple
            .ps
            .iter()
            .map(|p| p.locally_free.clone())
            .fold(Vec::new(), |acc, locally_free| union(acc, locally_free));

        etuple
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
