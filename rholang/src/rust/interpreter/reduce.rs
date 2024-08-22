// See See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::Par;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::{dispatch::RholangAndRustDispatcher, env::Env, rho_runtime::RhoTuplespace};

/**
 * Reduce is the interface for evaluating Rholang expressions.
 *
 */
pub trait Reduce {
    fn eval(&self, par: Par, env: Env<Par>, rand: Blake2b512Random) -> ();

    fn inj(&self, par: Par, rand: Blake2b512Random) -> ();
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

    fn inj(&self, par: Par, rand: Blake2b512Random) -> () {
        self.eval(par, Env::new(), rand)
    }
}
