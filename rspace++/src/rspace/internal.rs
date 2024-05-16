use crate::rspace::event::{Consume, Produce};
use proptest_derive::Arbitrary;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

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
#[derive(Debug, Clone)]
pub struct ContResult<C, P, K> {
    pub continuation: Arc<Mutex<K>>,
    pub persistent: bool,
    pub channels: Vec<C>,
    pub patterns: Vec<P>,
    pub peek: bool,
}

impl<C: Serialize, P: Serialize, K: Serialize> Serialize for ContResult<C, P, K> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let guard = self.continuation.lock().unwrap();
        let inner_data = &*guard; // Obtain the inner data from the mutex
        (self.persistent, &self.channels, &self.patterns, self.peek, inner_data)
            .serialize(serializer)
    }
}

impl<'de, C, P, K> Deserialize<'de> for ContResult<C, P, K>
where
    C: Deserialize<'de>,
    P: Deserialize<'de>,
    K: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (persistent, channels, patterns, peek, inner_data) =
            Deserialize::deserialize(deserializer)?;
        let continuation = Arc::new(Mutex::new(inner_data));
        Ok(ContResult {
            continuation,
            persistent,
            channels,
            patterns,
            peek,
        })
    }
}

// See rspace/src/main/scala/coop/rchain/rspace/internal.scala
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
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
// The 'Arbitrary' macro is needed here for proptest in hot_store_spec.rs
#[derive(Clone, Debug, Arbitrary)]
pub struct WaitingContinuation<P: Clone, K: Clone> {
    pub patterns: Vec<P>,
    pub continuation: Arc<Mutex<K>>,
    pub persist: bool,
    pub peeks: BTreeSet<i32>,
    pub source: Consume,
}

impl<P: Clone + PartialEq, K: Clone + PartialEq> PartialEq for WaitingContinuation<P, K> {
    fn eq(&self, other: &Self) -> bool {
        self.patterns == other.patterns
            && *self.continuation.lock().unwrap() == *other.continuation.lock().unwrap()
            && self.persist == other.persist
            && self.peeks == other.peeks
            && self.source == other.source
    }
}

impl<P: Clone + Eq, K: Clone + Eq> Eq for WaitingContinuation<P, K> {}

impl<P, K> Serialize for WaitingContinuation<P, K>
where
    P: Serialize + Clone,
    K: Serialize + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let guard = self.continuation.lock().unwrap();
        let inner_data = &*guard; // Obtain the inner data from the mutex
        (self.patterns.clone(), inner_data, self.persist, self.peeks.clone(), self.source.clone())
            .serialize(serializer)
    }
}

impl<'de, P, K> Deserialize<'de> for WaitingContinuation<P, K>
where
    P: Deserialize<'de> + Clone,
    K: Deserialize<'de> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (patterns, inner_data, persist, peeks, source) =
            Deserialize::deserialize(deserializer)?;
        let continuation = Arc::new(Mutex::new(inner_data));
        Ok(WaitingContinuation {
            patterns,
            continuation,
            persist,
            peeks,
            source,
        })
    }
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

#[derive(Clone, Debug)]
pub struct Install<P, K> {
    pub patterns: Vec<P>,
    pub continuation: K,
}
