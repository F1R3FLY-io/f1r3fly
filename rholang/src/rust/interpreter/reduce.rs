// See See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::{ETuple, ListParWithRandom, Par, TaggedContinuation};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

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
    fn produce(&self, chan: Par, data: ListParWithRandom, persistent: bool) -> () {
        self.update_mergeable_channels(&chan);
        let produce_result = self.space.lock().unwrap().produce(chan, data, persistent);
    }

    // proceed is alias for 'continue'
    fn proceed(&self, res: Application, repeat_op: (), persistent: bool) -> () {
        match res {
            Some((continuation, data_list, _)) => {
                if persistent {
                    self.dispatch_and_run(continuation, data_list, vec![repeat_op])
                }
            }
            None => todo!(),
        }
    }

    fn dispatch_and_run(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
        mut ops: Vec<()>,
    ) -> () {
        // Collect errors from all parallel execution paths (pars)
        let unit = self.dispatch(continuation, data_list);
        ops.insert(0, unit);
        // self.par_traverse_safe(ops, op)
        todo!();
    }

    fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<(Par, ListParWithRandom, ListParWithRandom, bool)>,
    ) -> () {
        self.dispatcher.dispatch(
            continuation,
            data_list.into_iter().map(|tuple| tuple.1).collect(),
        )
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

    fn par_traverse_safe<A>(&self, xs: Vec<A>, op: Box<dyn Fn(A) -> ()>) -> () {
        todo!()
    }
}
