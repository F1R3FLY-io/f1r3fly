use crate::rspace::internal::{Datum, WaitingContinuation};

// See rspace/src/main/scala/coop/rchain/rspace/HotStoreAction.scala
#[derive(Clone, Debug)]
pub enum HotStoreAction<C: Clone, P: Clone, A: Clone, K: Clone> {
    Insert(InsertAction<C, P, A, K>),
    Delete(DeleteAction<C>),
}

#[derive(Clone, Debug)]
pub enum InsertAction<C: Clone, P: Clone, A: Clone, K: Clone> {
    InsertData(InsertData<C, A>),
    InsertJoins(InsertJoins<C>),
    InsertContinuations(InsertContinuations<C, P, K>),
}

#[derive(Clone, Debug)]
pub struct InsertData<C: Clone, A: Clone> {
    pub channel: C,
    pub data: Vec<Datum<A>>,
}

#[derive(Clone, Debug)]
pub struct InsertJoins<C: Clone> {
    pub channel: C,
    pub joins: Vec<Vec<C>>,
}

#[derive(Clone, Debug)]
pub struct InsertContinuations<C: Clone, P: Clone, K: Clone> {
    pub channels: Vec<C>,
    pub continuations: Vec<WaitingContinuation<P, K>>,
}

#[derive(Clone, Debug)]
pub enum DeleteAction<C: Clone> {
    DeleteData(DeleteData<C>),
    DeleteJoins(DeleteJoins<C>),
    DeleteContinuations(DeleteContinuations<C>),
}

#[derive(Clone, Debug)]
pub struct DeleteData<C: Clone> {
    pub channel: C,
}

#[derive(Clone, Debug)]
pub struct DeleteJoins<C: Clone> {
    pub channel: C,
}

#[derive(Clone, Debug)]
pub struct DeleteContinuations<C: Clone> {
    pub channels: Vec<C>,
}