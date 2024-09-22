// See models/src/test/scala/coop/rchain/models/rholang/SortTest.scala - ScoredTermSpec

use models::{
    rhoapi::{expr::ExprInstance, Expr, Var},
    rust::{
        par_set::ParSet,
        par_set_type_mapper::ParSetTypeMapper,
        rholang::sorter::{
            expr_sort_matcher::ExprSortMatcher,
            score_tree::{ScoreAtom, ScoredTerm, Tree},
            sortable::Sortable,
        },
        sorted_par_hash_set::SortedParHashSet,
    },
};

#[test]
fn scored_term_should_sort_so_that_shorter_nodes_come_first() {
    let mut unsorted_terms = vec![
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 2, 2, 3]),
        },
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 2, 2]),
        },
    ];

    let sorted_terms = vec![
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 2, 2]),
        },
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 2, 2, 3]),
        },
    ];

    ScoredTerm::sort_vec(&mut unsorted_terms);
    assert_eq!(unsorted_terms, sorted_terms);
}

#[test]
fn scored_term_should_sort_so_that_smaller_leaves_stay_first() {
    let mut unsorted_terms = vec![
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(1),
        },
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(2),
        },
    ];

    let sorted_terms = vec![
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(1),
        },
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(2),
        },
    ];

    ScoredTerm::sort_vec(&mut unsorted_terms);
    assert_eq!(unsorted_terms, sorted_terms);
}

#[test]
fn scored_term_should_sort_so_that_smaller_leaves_are_put_first() {
    let mut unsorted_terms = vec![
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(2),
        },
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(1),
        },
    ];

    let sorted_terms = vec![
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(1),
        },
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::<ScoreAtom>::create_leaf_from_i64(2),
        },
    ];

    ScoredTerm::sort_vec(&mut unsorted_terms);
    assert_eq!(unsorted_terms, sorted_terms);
}

#[test]
fn scored_term_should_sort_so_that_smaller_nodes_are_put_first() {
    let mut unsorted_terms = vec![
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::Node(vec![
                Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 2]),
                Tree::<ScoreAtom>::create_node_from_i64s(vec![2, 2]),
            ]),
        },
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::Node(vec![
                Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 1]),
                Tree::<ScoreAtom>::create_node_from_i64s(vec![2, 2]),
            ]),
        },
    ];

    let sorted_terms = vec![
        ScoredTerm {
            term: "bar".to_string(),
            score: Tree::Node(vec![
                Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 1]),
                Tree::<ScoreAtom>::create_node_from_i64s(vec![2, 2]),
            ]),
        },
        ScoredTerm {
            term: "foo".to_string(),
            score: Tree::Node(vec![
                Tree::<ScoreAtom>::create_node_from_i64s(vec![1, 2]),
                Tree::<ScoreAtom>::create_node_from_i64s(vec![2, 2]),
            ]),
        },
    ];

    ScoredTerm::sort_vec(&mut unsorted_terms);
    assert_eq!(unsorted_terms, sorted_terms);
}

#[test]
fn scored_term_should_sort_so_that_whenever_scores_differ_then_result_terms_have_to_differ_and_the_other_way_around(
) {
    todo!()
}

#[test]
fn scored_term_should_sort_different_match_terms_free_count() {
    todo!()
}

#[test]
fn scored_term_should_sort_so_that_whenever_scores_or_result_terms_differ_then_the_initial_terms_differ_and_the_other_way_around(
) {
    todo!()
}

#[test]
fn scored_term_should_sort_so_that_unequal_terms_have_unequal_scores_and_the_other_way_around() {
    todo!()
}

#[test]
fn scored_term_should_score_in_different_byte_string_should_be_unequal() {
    todo!()
}

#[test]
fn scored_term_should_sort_so_that_unequal_new_have_unequal_scores() {
    todo!()
}

#[test]
fn scored_term_should_sort_so_that_unequal_emethod_have_unequal_scores() {
    todo!()
}

#[test]
fn scored_term_should_sort_so_that_unequal_par_map_have_unequal_scores() {
    todo!()
}

#[test]
fn scored_term_should_sort_so_that_unequal_par_set_have_unequal_scores() {
    let set_1 = Expr {
        expr_instance: Some(ExprInstance::ESetBody(ParSetTypeMapper::par_set_to_eset(
            ParSet {
                ps: SortedParHashSet::create_from_empty(),
                connective_used: true,
                locally_free: vec![],
                remainder: None,
            },
        ))),
    };

    let set_2 = Expr {
        expr_instance: Some(ExprInstance::ESetBody(ParSetTypeMapper::par_set_to_eset(
            ParSet {
                ps: SortedParHashSet::create_from_empty(),
                connective_used: false,
                locally_free: vec![],
                remainder: None,
            },
        ))),
    };

    assert!(ExprSortMatcher::sort_match(&set_1).score != ExprSortMatcher::sort_match(&set_2).score);

    let set_3 = Expr {
        expr_instance: Some(ExprInstance::ESetBody(ParSetTypeMapper::par_set_to_eset(
            ParSet {
                ps: SortedParHashSet::create_from_empty(),
                connective_used: true,
                locally_free: vec![],
                remainder: None,
            },
        ))),
    };

    let set_4 = Expr {
        expr_instance: Some(ExprInstance::ESetBody(ParSetTypeMapper::par_set_to_eset(
            ParSet {
                ps: SortedParHashSet::create_from_empty(),
                connective_used: true,
                locally_free: vec![],
                remainder: Some(Var::default()),
            },
        ))),
    };

    assert!(ExprSortMatcher::sort_match(&set_3).score != ExprSortMatcher::sort_match(&set_4).score);
}

#[test]
fn scored_term_should_sort_so_that_unequal_list_have_unequal_scores() {
    todo!()
}
