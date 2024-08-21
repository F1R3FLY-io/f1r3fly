use std::collections::HashMap;

use models::rhoapi::{ListParWithRandom, TaggedContinuation};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/dispatch.scala
pub trait Dispatch<A, K> {
    fn dispatch(continuation: K, data_list: Vec<A>) -> ();
}

pub type RhoDispatch = RholangAndRustDispatcher;

pub struct RholangAndRustDispatcher {
    _dispatch_table: HashMap<i64, Box<dyn Fn(Vec<ListParWithRandom>) -> ()>>,
}

impl RholangAndRustDispatcher {
    pub fn dispatch(
        &self,
        continuation: TaggedContinuation,
        data_list: Vec<ListParWithRandom>,
    ) -> () {
        todo!()
    }
}
