// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/ReceiveBindsSortMatcher.scala

use models::{
    rhoapi::{Par, ReceiveBind, Var},
    rust::rholang::sorter::{receive_sort_matcher::ReceiveSortMatcher, score_tree::ScoredTerm},
};

use crate::rust::interpreter::errors::InterpreterError;

use super::exports::FreeMap;

pub fn pre_sort_binds<T: Clone>(
    binds: Vec<(Vec<Par>, Option<Var>, Par, FreeMap<T>)>,
) -> Result<Vec<(ReceiveBind, FreeMap<T>)>, InterpreterError> {
    let mut bind_sortings: Vec<ScoredTerm<(ReceiveBind, FreeMap<T>)>> = binds
        .into_iter()
        .map(|(patterns, remainder, channel, known_free)| {
            let sorted_bind = ReceiveSortMatcher::sort_bind(ReceiveBind {
                patterns,
                source: Some(channel),
                remainder,
                free_count: known_free.count_no_wildcards() as i32,
            });

            ScoredTerm {
                term: (sorted_bind.term, known_free),
                score: sorted_bind.score,
            }
        })
        .collect();

    ScoredTerm::sort_vec(&mut bind_sortings);
    Ok(bind_sortings
        .into_iter()
        .map(|scored_term| scored_term.term)
        .collect())
}
