// See See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use futures::future;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::tagged_continuation::TaggedCont;
use models::rhoapi::BindPattern;
use models::rhoapi::ParWithRandom;
use models::rhoapi::{ETuple, ListParWithRandom, Par, TaggedContinuation};
use rspace_plus_plus::rspace::util::unpack_option_with_peek;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::errors::InterpreterError;
use super::{dispatch::RholangAndRustDispatcher, env::Env, rho_runtime::RhoTuplespace};

/**
 * Reduce is the interface for evaluating Rholang expressions.
 */
pub trait Reduce {
    fn eval(&self, par: Par, env: Env<Par>, rand: Blake2b512Random) -> ();

    fn inj(&self, par: Par, rand: Blake2b512Random) -> () {
        self.eval(par, Env::new(), rand)
    }
}

#[derive(Clone)]
pub struct DebruijnInterpreter {
    pub space: RhoTuplespace,
    pub dispatcher: RholangAndRustDispatcher,
    pub urn_map: HashMap<String, Par>,
    pub merge_chs: Arc<RwLock<HashSet<Par>>>,
    pub mergeable_tag_name: Par,
}

impl Reduce for DebruijnInterpreter {
    fn eval(&self, par: Par, env: Env<Par>, rand: Blake2b512Random) -> () {
        todo!()
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
        self.proceed(
            unpack_option_with_peek(produce_result),
            Box::pin(self.produce(chan, data, persistent)).await,
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

        self.proceed(
            unpack_option_with_peek(consume_result),
            Box::pin(self.consume(binds, body, persistent, peek)).await,
            persistent,
        )
        .await
    }

    // proceed is alias for 'continue'
    async fn proceed(
        &self,
        res: Application,
        repeat_op: Result<(), InterpreterError>,
        persistent: bool,
    ) -> Result<(), InterpreterError> {
        match res {
            Some((continuation, data_list, peek)) => {
                if persistent {
                    self.dispatch_and_run(continuation, data_list, vec![repeat_op])
                        .await
                } else if peek {
                    let produce_peeks_result = self.produce_peeks(data_list.clone()).await;
                    self.dispatch_and_run(continuation, data_list, produce_peeks_result)
                        .await
                } else {
                    self.dispatch(continuation, data_list)
                }
            }
            None => Ok(()),
        }
    }

    // TODO: Review this method to make sure it follows Scala logic
    async fn dispatch_and_run(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
        ops: Vec<Result<(), InterpreterError>>,
    ) -> Result<(), InterpreterError> {
        let mut tasks: Vec<Result<(), InterpreterError>> =
            vec![self.dispatch(continuation.clone(), data_list.clone())];

        tasks.extend(ops);

        self.par_traverse_safe(tasks, |op| op).await
    }

    fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
    ) -> Result<(), InterpreterError> {
        self.dispatcher.dispatch(
            continuation,
            data_list.into_iter().map(|tuple| tuple.1).collect(),
        )
    }

    async fn produce_peeks(
        &self,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
    ) -> Vec<Result<(), InterpreterError>> {
        let results: Vec<_> = data_list
            .into_iter()
            .filter(|(_, _, _, persist)| !persist)
            .map(|(chan, _, removed_data, _)| async {
                self.produce(chan, removed_data, false).await
            })
            .collect();

        future::join_all(results).await
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

    // TODO: Review this method to make sure it follows Scala logic
    async fn par_traverse_safe<A>(
        &self,
        xs: Vec<A>,
        op: impl Fn(A) -> Result<(), InterpreterError> + Clone,
    ) -> Result<(), InterpreterError> {
        let futures: Vec<_> = xs
            .into_iter()
            .map(|x| {
                let op_clone = op.clone();
                async move { op_clone(x).map_err(|e| e) }
            })
            .collect();

        let results: Vec<Result<(), InterpreterError>> = future::join_all(futures).await;

        let flattened_results: Vec<InterpreterError> = results
            .into_iter()
            .filter_map(|result| result.err())
            .collect();

        self.aggregate_evaluator_errors(flattened_results)
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
}
