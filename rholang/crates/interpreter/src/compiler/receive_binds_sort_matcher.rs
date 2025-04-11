// See rholang/src/main/scala/coop/rchain/rholang/interpreter/compiler/ReceiveBindsSortMatcher.scala

use models::{
    rhoapi::{Par, ReceiveBind, Var},
    rust::rholang::sorter::{receive_sort_matcher::ReceiveSortMatcher, score_tree::ScoredTerm},
};

use crate::errors::InterpreterError;

use super::exports::FreeMap;

pub fn pre_sort_binds(
    binds: Vec<(Vec<Par>, Option<Var>, Par, FreeMap)>,
) -> Result<Vec<(ReceiveBind, FreeMap)>, InterpreterError> {
    // println!("\nbinds in pre_sort_binds: {:?}", binds);

    let mut bind_sortings: Vec<ScoredTerm<(ReceiveBind, FreeMap)>> = binds
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

#[cfg(test)]
mod tests {
    use models::rust::utils::{new_freevar_var, new_gint_par};

    use super::*;

    #[test]
    fn binds_should_pre_sort_based_on_their_channel_and_then_patterns() {
        let empty_map = FreeMap::new();

        let binds: Vec<(Vec<Par>, Option<Var>, Par, FreeMap)> = vec![
            (
                vec![new_gint_par(2, Vec::new(), false)],
                None,
                new_gint_par(3, Vec::new(), false),
                empty_map.clone(),
            ),
            (
                vec![new_gint_par(3, Vec::new(), false)],
                None,
                new_gint_par(2, Vec::new(), false),
                empty_map.clone(),
            ),
            (
                vec![new_gint_par(3, Vec::new(), false)],
                Some(new_freevar_var(0)),
                new_gint_par(2, Vec::new(), false),
                empty_map.clone(),
            ),
            (
                vec![new_gint_par(1, Vec::new(), false)],
                None,
                new_gint_par(3, Vec::new(), false),
                empty_map.clone(),
            ),
        ];

        let sorted_binds: Vec<(ReceiveBind, FreeMap)> = vec![
            (
                ReceiveBind {
                    patterns: vec![new_gint_par(3, Vec::new(), false)],
                    source: Some(new_gint_par(2, Vec::new(), false)),
                    remainder: None,
                    free_count: 0,
                },
                empty_map.clone(),
            ),
            (
                ReceiveBind {
                    patterns: vec![new_gint_par(3, Vec::new(), false)],
                    source: Some(new_gint_par(2, Vec::new(), false)),
                    remainder: Some(new_freevar_var(0)),
                    free_count: 0,
                },
                empty_map.clone(),
            ),
            (
                ReceiveBind {
                    patterns: vec![new_gint_par(1, Vec::new(), false)],
                    source: Some(new_gint_par(3, Vec::new(), false)),
                    remainder: None,
                    free_count: 0,
                },
                empty_map.clone(),
            ),
            (
                ReceiveBind {
                    patterns: vec![new_gint_par(2, Vec::new(), false)],
                    source: Some(new_gint_par(3, Vec::new(), false)),
                    remainder: None,
                    free_count: 0,
                },
                empty_map,
            ),
        ];

        let result = pre_sort_binds(binds);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), sorted_binds);
    }
}
