use crate::rspace::internal::{Datum, WaitingContinuation};

// See rspace/src/main/scala/coop/rchain/rspace/HotStoreAction.scala
#[derive(Clone)]
pub enum HotStoreAction<C, P, A, K> {
    Insert(InsertAction<C, P, A, K>),
    Delete(DeleteAction<C>),
}

#[derive(Clone)]
pub enum InsertAction<C, P, A, K> {
    InsertData(InsertData<C, A>),
    InsertJoins(InsertJoins<C>),
    InsertContinuations(InsertContinuations<C, P, K>),
}

#[derive(Clone)]
pub struct InsertData<C, A> {
    pub channel: C,
    pub data: Vec<Datum<A>>,
}

#[derive(Clone)]
pub struct InsertJoins<C> {
    pub channel: C,
    pub joins: Vec<Vec<C>>,
}

#[derive(Clone)]
pub struct InsertContinuations<C, P, K> {
    pub channels: Vec<C>,
    pub continuations: Vec<WaitingContinuation<P, K>>,
}

#[derive(Clone)]
pub enum DeleteAction<C> {
    DeleteData(DeleteData<C>),
    DeleteJoins(DeleteJoins<C>),
    DeleteContinuations(DeleteContinuations<C>),
}

#[derive(Clone)]
pub struct DeleteData<C> {
    pub channel: C,
}

#[derive(Clone)]
pub struct DeleteJoins<C> {
    pub channel: C,
}

#[derive(Clone)]
pub struct DeleteContinuations<C> {
    pub channels: Vec<C>,
}
