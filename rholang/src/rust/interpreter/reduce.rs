// See See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::{BindPattern, Bundle, Expr, Match, New, ParWithRandom, Receive, Send, Var};
use models::rhoapi::{ETuple, ListParWithRandom, Par, TaggedContinuation};
use rspace_plus_plus::rspace::util::unpack_option_with_peek;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use super::accounting::_cost;
use super::errors::InterpreterError;
use super::util::GeneratedMessage;
use super::{dispatch::RholangAndRustDispatcher, env::Env, rho_runtime::RhoTuplespace};

/**
 * Reduce is the interface for evaluating Rholang expressions.
 */
pub trait Reduce {
    async fn eval(
        &self,
        par: Par,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError>;

    async fn inj(&self, par: Par, rand: Blake2b512Random) -> Result<(), InterpreterError> {
        self.eval(par, Env::new(), rand).await
    }
}

#[derive(Clone)]
pub struct DebruijnInterpreter {
    pub space: RhoTuplespace,
    pub dispatcher: RholangAndRustDispatcher,
    pub urn_map: HashMap<String, Par>,
    pub merge_chs: Arc<RwLock<HashSet<Par>>>,
    pub mergeable_tag_name: Par,
    pub cost: _cost
}

impl Reduce for DebruijnInterpreter {
    async fn eval(
        &self,
        par: Par,
        env: Env<Par>,
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
                            env.clone(),
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
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        match term {
            GeneratedMessage::Send(term) => self.send_eval(term, env, rand),
            GeneratedMessage::Receive(term) => self.receive_eval(term, env, rand),
            GeneratedMessage::New(term) => self.new_eval(term, env, rand),
            GeneratedMessage::Match(term) => self.match_eval(term, env, rand),
            GeneratedMessage::Bundle(term) => self.bundle_eval(term, env, rand),
            GeneratedMessage::Expr(term) => match &term.expr_instance {
                Some(expr_instance) => match expr_instance {
                    ExprInstance::EVarBody(e) => {
                        let res = self.var_eval(&e.clone().v.unwrap(), &env, &rand)?;
                        self.eval(res, env, rand).await
                    }
                    ExprInstance::EMethodBody(e) => {
                        let res = self.eval_expr_to_par(
                            &Expr {
                                expr_instance: Some(ExprInstance::EMethodBody(e.clone())),
                            },
                            &env,
                            &rand,
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
    fn send_eval(
        &self,
        send: &Send,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        todo!()
    }

    fn receive_eval(
        &self,
        send: &Receive,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        todo!()
    }

    fn new_eval(
        &self,
        send: &New,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        todo!()
    }

    fn match_eval(
        &self,
        send: &Match,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        todo!()
    }

    fn bundle_eval(
        &self,
        send: &Bundle,
        env: Env<Par>,
        rand: Blake2b512Random,
    ) -> Result<(), InterpreterError> {
        todo!()
    }

    fn var_eval(
        &self,
        send: &Var,
        env: &Env<Par>,
        rand: &Blake2b512Random,
    ) -> Result<Par, InterpreterError> {
        todo!()
    }

    fn eval_expr_to_par(
        &self,
        send: &Expr,
        env: &Env<Par>,
        rand: &Blake2b512Random,
    ) -> Result<Par, InterpreterError> {
        todo!()
    }
}
