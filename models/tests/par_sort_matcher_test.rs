// See models/src/test/scala/coop/rchain/models/rholang/SortTest.scala - ParSortMatcherSpec

use models::{
    rhoapi::Par,
    rust::{
        rholang::sorter::{par_sort_matcher::ParSortMatcher, sortable::Sortable},
        utils::new_gint_expr,
    },
};

// TODO: Finish these tests

#[test]
fn par_should_sort_so_that_smaller_integers_come_first() {
    let par_ground = Par::default().with_exprs(vec![
        new_gint_expr(2),
        new_gint_expr(1),
        new_gint_expr(-1),
        new_gint_expr(-2),
        new_gint_expr(0),
    ]);

    let sorted_par_ground = Par::default().with_exprs(vec![
        new_gint_expr(-2),
        new_gint_expr(-1),
        new_gint_expr(0),
        new_gint_expr(1),
        new_gint_expr(2),
    ]);

    assert_eq!(
        ParSortMatcher::sort_match(&par_ground).term,
        sorted_par_ground
    );
}
