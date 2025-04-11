use std::{borrow::Borrow, hash::Hash, ops::Deref};

use indextree::NodeId;

use super::score_tree::{ArenaBuilder, ScoreBuilder, ScoreTree};

#[derive(Debug, Clone)]
pub struct ScoredTerm<T> {
    pub term: Sorted<T>,
    score: ScoreTree,
}

impl<T> ScoredTerm<T> {
    pub(crate) fn graft_into<Builder: ScoreBuilder>(&self, score: &mut Builder) -> Option<NodeId> {
        score.graft(&self.score)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Sorted<T>(T);

impl<T> Sorted<T> {
    pub fn take(self) -> T {
        self.0
    }
}

pub trait Sortable: Sized {
    type Sorter<'a>: SortMatcher + 'a
    where
        Self: 'a;
    fn sorter(&mut self) -> Self::Sorter<'_>;

    fn sort_match(mut self) -> ScoredTerm<Self> {
        let sorter = self.sorter();
        let mut score = ArenaBuilder::new();
        sorter.sort_match(&mut score);
        ScoredTerm {
            term: Sorted(self),
            score: score.to_tree(),
        }
    }
}

pub trait SortMatcher: Sized {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder);
}

impl<T> Borrow<Sorted<T>> for ScoredTerm<T> {
    fn borrow(&self) -> &Sorted<T> {
        &self.term
    }
}

impl<T> Deref for Sorted<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> PartialEq for ScoredTerm<T> {
    fn eq(&self, other: &ScoredTerm<T>) -> bool {
        self.score == other.score
    }
}

impl<T: PartialEq> PartialEq<T> for ScoredTerm<T> {
    fn eq(&self, other: &T) -> bool {
        self.term.0 == *other
    }
}

impl<T> Eq for ScoredTerm<T> {}

impl<T> PartialOrd for ScoredTerm<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl<T> Ord for ScoredTerm<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

// Borrow needs this

impl<T: Hash> Hash for ScoredTerm<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.term.hash(state);
    }
}
