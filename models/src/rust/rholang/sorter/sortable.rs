// See models/src/main/scala/coop/rchain/models/rholang/sorter/Sortable.scala

use super::score_tree::ScoredTerm;

pub trait Sortable<T> {
    fn sort_match(term: &T) -> ScoredTerm<T>;
}
