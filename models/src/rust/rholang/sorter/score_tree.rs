// See models/src/main/scala/coop/rchain/models/rholang/sorter/ScoreTree.scala

use crate::ByteString;

pub struct ScoreTree;

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Tree<T> {
    Leaf(T),
    Node(Vec<Tree<T>>),
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum TaggedAtom {
    IntAtom(i64),
    StringAtom(String),
    ByteAtom(ByteString),
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct ScoreAtom {
    value: TaggedAtom,
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct ScoredTerm<T> {
    pub term: T,
    pub score: Tree<ScoreAtom>,
}
