use crate::rspace::internal::{Datum, WaitingContinuation};

// See rspace/src/main/scala/coop/rchain/rspace/HotStoreTrieAction.scala
pub enum HotStoreTrieAction<C, P, A, K> {
    TrieInsertAction(TrieInsertAction<C, P, A, K>),
    TrieDeleteAction(TrieDeleteAction),
}

pub enum TrieInsertAction<C, P, A, K> {
    TrieInsertProduce(TrieInsertProduce<A>),
    TrieInsertJoins(TrieInsertJoins<C>),
    TrieInsertConsume(TrieInsertConsume<P, K>),
    TrieInsertBinaryProduce(TrieInsertBinaryProduce),
    TrieInsertBinaryJoins(TrieInsertBinaryJoins),
    TrieInsertBinaryConsume(TrieInsertBinaryConsume),
}

pub struct TrieInsertProduce<A> {
    hash: blake3::Hash,
    data: Vec<Datum<A>>,
}

pub struct TrieInsertJoins<C> {
    hash: blake3::Hash,
    joins: Vec<Vec<C>>,
}

pub struct TrieInsertConsume<P, K> {
    hash: blake3::Hash,
    continuations: Vec<WaitingContinuation<P, K>>,
}

pub struct TrieInsertBinaryProduce {
    hash: blake3::Hash,
    data: Vec<Vec<u8>>,
}

pub struct TrieInsertBinaryJoins {
    hash: blake3::Hash,
    joins: Vec<Vec<u8>>,
}

pub struct TrieInsertBinaryConsume {
    hash: blake3::Hash,
    continuations: Vec<Vec<u8>>,
}

pub enum TrieDeleteAction {
    TrieDeleteProduce(TrieDeleteProduce),
    TrieDeleteJoins(TrieDeleteJoins),
    TrieDeleteConsume(TrieDeleteConsume),
}

pub struct TrieDeleteProduce {
    hash: blake3::Hash,
}

pub struct TrieDeleteJoins {
    hash: blake3::Hash,
}

pub struct TrieDeleteConsume {
    hash: blake3::Hash,
}
