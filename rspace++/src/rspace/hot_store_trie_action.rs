use super::hashing::blake3_hash::Blake3Hash;
use crate::rspace::internal::{Datum, WaitingContinuation};

// See rspace/src/main/scala/coop/rchain/rspace/HotStoreTrieAction.scala
pub enum HotStoreTrieAction<C: Clone, P: Clone, A: Clone, K: Clone> {
    TrieInsertAction(TrieInsertAction<C, P, A, K>),
    TrieDeleteAction(TrieDeleteAction),
}

pub enum TrieInsertAction<C: Clone, P: Clone, A: Clone, K: Clone> {
    TrieInsertProduce(TrieInsertProduce<A>),
    TrieInsertJoins(TrieInsertJoins<C>),
    TrieInsertConsume(TrieInsertConsume<P, K>),
    TrieInsertBinaryProduce(TrieInsertBinaryProduce),
    TrieInsertBinaryJoins(TrieInsertBinaryJoins),
    TrieInsertBinaryConsume(TrieInsertBinaryConsume),
}

pub struct TrieInsertProduce<A: Clone> {
    hash: Blake3Hash,
    data: Vec<Datum<A>>,
}

impl<A: Clone> TrieInsertProduce<A> {
    pub fn new(hash: Blake3Hash, data: Vec<Datum<A>>) -> Self {
        TrieInsertProduce { hash, data }
    }
}

pub struct TrieInsertJoins<C: Clone> {
    hash: Blake3Hash,
    joins: Vec<Vec<C>>,
}

impl<C: Clone> TrieInsertJoins<C> {
    pub fn new(hash: Blake3Hash, joins: Vec<Vec<C>>) -> Self {
        TrieInsertJoins { hash, joins }
    }
}

pub struct TrieInsertConsume<P: Clone, K: Clone> {
    hash: Blake3Hash,
    continuations: Vec<WaitingContinuation<P, K>>,
}

impl<P: Clone, K: Clone> TrieInsertConsume<P, K> {
    pub fn new(hash: Blake3Hash, continuations: Vec<WaitingContinuation<P, K>>) -> Self {
        TrieInsertConsume {
            hash,
            continuations,
        }
    }
}

pub struct TrieInsertBinaryProduce {
    hash: Blake3Hash,
    data: Vec<Vec<u8>>,
}

pub struct TrieInsertBinaryJoins {
    hash: Blake3Hash,
    joins: Vec<Vec<u8>>,
}

pub struct TrieInsertBinaryConsume {
    hash: Blake3Hash,
    continuations: Vec<Vec<u8>>,
}

pub enum TrieDeleteAction {
    TrieDeleteProduce(TrieDeleteProduce),
    TrieDeleteJoins(TrieDeleteJoins),
    TrieDeleteConsume(TrieDeleteConsume),
}

pub struct TrieDeleteProduce {
    hash: Blake3Hash,
}

impl TrieDeleteProduce {
    pub fn new(hash: Blake3Hash) -> Self {
        TrieDeleteProduce { hash }
    }
}

pub struct TrieDeleteJoins {
    hash: Blake3Hash,
}

impl TrieDeleteJoins {
    pub fn new(hash: Blake3Hash) -> Self {
        TrieDeleteJoins { hash }
    }
}

pub struct TrieDeleteConsume {
    hash: Blake3Hash,
}

impl TrieDeleteConsume {
    pub fn new(hash: Blake3Hash) -> Self {
        TrieDeleteConsume { hash }
    }
}
