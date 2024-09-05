// See models/src/main/scala/coop/rchain/models/rholang/sorter/MatchSortMatcher.scala

use crate::{
    rhoapi::{Match, MatchCase},
    rust::rholang::sorter::{
        par_sort_matcher::ParSortMatcher,
        score_tree::{Score, ScoreAtom, Tree},
    },
};

use super::{score_tree::ScoredTerm, sortable::Sortable};

pub struct MatchSortMatcher;

impl Sortable<Match> for MatchSortMatcher {
    fn sort_match(m: &Match) -> ScoredTerm<Match> {
        fn sort_case(match_case: &MatchCase) -> ScoredTerm<MatchCase> {
            let sorted_pattern = ParSortMatcher::sort_match(
                &match_case
                    .pattern
                    .as_ref()
                    .expect("pattern field on MatchCase was None, should be Some"),
            );
            let sorted_body = ParSortMatcher::sort_match(
                &match_case
                    .source
                    .as_ref()
                    .expect("source field on MatchCase was None, should be Some"),
            );
            let free_count_score =
                Tree::<ScoreAtom>::create_leaf_from_i64(match_case.free_count as i64);

            ScoredTerm {
                term: MatchCase {
                    pattern: Some(sorted_pattern.term),
                    source: Some(sorted_body.term),
                    free_count: match_case.free_count,
                },
                score: Tree::Node(vec![
                    sorted_pattern.score,
                    sorted_body.score,
                    free_count_score,
                ]),
            }
        }

        let sorted_value = ParSortMatcher::sort_match(
            &m.target
                .as_ref()
                .expect("target field on Match was None, should be Some"),
        );
        let scored_cases: Vec<ScoredTerm<MatchCase>> =
            m.cases.iter().map(|c| sort_case(c)).collect();
        let connective_used_score = if m.connective_used { 1 } else { 0 };

        ScoredTerm {
            term: Match {
                target: Some(sorted_value.term),
                cases: scored_cases.clone().into_iter().map(|c| c.term).collect(),
                locally_free: m.locally_free.clone(),
                connective_used: m.connective_used,
            },
            score: Tree::<ScoreAtom>::create_node_from_i32(
                Score::MATCH,
                vec![sorted_value.score]
                    .into_iter()
                    .chain(scored_cases.into_iter().map(|c| c.score))
                    .into_iter()
                    .chain(vec![Tree::<ScoreAtom>::create_leaf_from_i64(
                        connective_used_score,
                    )])
                    .collect(),
            ),
        }
    }
}
