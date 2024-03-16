use super::hashing::blake2b256_hash::Blake2b256Hash;
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
    pub hash: Blake2b256Hash,
    pub data: Vec<Datum<A>>,
}

impl<A: Clone> TrieInsertProduce<A> {
    pub fn new(hash: Blake2b256Hash, data: Vec<Datum<A>>) -> Self {
        TrieInsertProduce { hash, data }
    }
}

pub struct TrieInsertJoins<C: Clone> {
    pub hash: Blake2b256Hash,
    pub joins: Vec<Vec<C>>,
}

impl<C: Clone> TrieInsertJoins<C> {
    pub fn new(hash: Blake2b256Hash, joins: Vec<Vec<C>>) -> Self {
        TrieInsertJoins { hash, joins }
    }
}

pub struct TrieInsertConsume<P: Clone, K: Clone> {
    pub hash: Blake2b256Hash,
    pub continuations: Vec<WaitingContinuation<P, K>>,
}

impl<P: Clone, K: Clone> TrieInsertConsume<P, K> {
    pub fn new(hash: Blake2b256Hash, continuations: Vec<WaitingContinuation<P, K>>) -> Self {
        TrieInsertConsume {
            hash,
            continuations,
        }
    }
}

pub struct TrieInsertBinaryProduce {
    pub hash: Blake2b256Hash,
    pub data: Vec<Vec<u8>>,
}

pub struct TrieInsertBinaryJoins {
    pub hash: Blake2b256Hash,
    pub joins: Vec<Vec<u8>>,
}

pub struct TrieInsertBinaryConsume {
    pub hash: Blake2b256Hash,
    pub continuations: Vec<Vec<u8>>,
}

pub enum TrieDeleteAction {
    TrieDeleteProduce(TrieDeleteProduce),
    TrieDeleteJoins(TrieDeleteJoins),
    TrieDeleteConsume(TrieDeleteConsume),
}

pub struct TrieDeleteProduce {
    pub hash: Blake2b256Hash,
}

impl TrieDeleteProduce {
    pub fn new(hash: Blake2b256Hash) -> Self {
        TrieDeleteProduce { hash }
    }
}

pub struct TrieDeleteJoins {
    pub hash: Blake2b256Hash,
}

impl TrieDeleteJoins {
    pub fn new(hash: Blake2b256Hash) -> Self {
        TrieDeleteJoins { hash }
    }
}

pub struct TrieDeleteConsume {
    pub hash: Blake2b256Hash,
}

impl TrieDeleteConsume {
    pub fn new(hash: Blake2b256Hash) -> Self {
        TrieDeleteConsume { hash }
    }
}
