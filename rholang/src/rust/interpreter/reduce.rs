use models::rhoapi::Par;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use super::{dispatch::RholangAndRustDispatcher, env::Env, rho_runtime::RhoTuplespace};

/**
 * Reduce is the interface for evaluating Rholang expressions.
 *
 * See rholang/src/main/scala/coop/rchain/rholang/interpreter/Reduce.scala
 */
pub trait Reduce {
    fn eval(&self, par: Par, env: Env<Par>, rand: Blake2b256Hash) -> ();

    fn inj(&self, par: Par, rand: Blake2b256Hash) -> ();
}

pub struct DebruijnInterpreter {
    space: RhoTuplespace,
    dispatcher: Box<dyn Fn() -> RholangAndRustDispatcher>,
    urn_map: HashMap<String, Par>,
    merg_chs: Arc<RwLock<HashSet<Par>>>,
    mergeable_tag_name: Par,
}

impl Reduce for DebruijnInterpreter {
    fn eval(&self, par: Par, env: Env<Par>, rand: Blake2b256Hash) -> () {
        todo!()
    }

    fn inj(&self, par: Par, rand: Blake2b256Hash) -> () {
        self.eval(par, Env::new(), rand)
    }
}
