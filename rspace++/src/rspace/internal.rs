use crate::rspace::event::{Consume, Produce};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// See rspace/src/main/scala/coop/rchain/rspace/ISpace.scala
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RSpaceResult<C, A> {
    pub channel: C,
    pub matched_datum: A,
    pub removed_datum: A,
    pub persistent: bool,
}

// See rspace/src/main/scala/coop/rchain/rspace/ISpace.scala
// NOTE: On Scala side, they are defaulting "peek" to false
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContResult<C, P, K> {
    pub continuation: K,
    pub persistent: bool,
    pub channels: Vec<C>,
    pub patterns: Vec<P>,
    pub peek: bool,
}

// See rspace/src/main/scala/coop/rchain/rspace/internal.scala
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Datum<A: Clone> {
    pub a: A,
    pub persist: bool,
    pub source: Produce,
}

impl<A> Datum<A>
where
    A: Clone + Serialize,
{
    pub fn create<C: Serialize>(channel: C, a: A, persist: bool) -> Datum<A> {
        Datum {
            a: a.clone(),
            persist,
            source: Produce::create(channel, a, persist),
        }
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/internal.scala
#[derive(Clone, Debug)]
pub struct WaitingContinuation<P: Clone, K: Clone> {
    pub patterns: Vec<P>,
    pub continuation: K,
    pub persist: bool,
    pub peeks: BTreeSet<i32>,
    pub source: Consume,
}

// See rspace/src/main/scala/coop/rchain/rspace/internal.scala
#[derive(Clone, Debug)]
pub struct ConsumeCandidate<C, A: Clone> {
    pub channel: C,
    pub datum: Datum<A>,
    pub removed_datum: A,
    pub datum_index: i32,
}

// See rspace/src/main/scala/coop/rchain/rspace/internal.scala
#[derive(Debug)]
pub struct ProduceCandidate<C, P: Clone, A: Clone, K: Clone> {
    pub channels: Vec<C>,
    pub continuation: WaitingContinuation<P, K>,
    pub continuation_index: i32,
    pub data_candidates: Vec<ConsumeCandidate<C, A>>,
}

// See rspace/src/main/scala/coop/rchain/rspace/internal.scala
#[derive(Debug)]
pub struct Row<P: Clone, A: Clone, K: Clone> {
    pub data: Vec<Datum<A>>,
    pub wks: Vec<WaitingContinuation<P, K>>,
}
