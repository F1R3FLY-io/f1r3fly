// See models/src/test/scala/coop/rchain/models/rholang/SortTest.scala - VarSortMatcherSpec

use models::create_bit_vector;
use models::rhoapi::expr::ExprInstance;
use models::rhoapi::var::{VarInstance, WildcardMsg};
use models::rhoapi::{EVar, Expr, Par, Var};
use models::rust::rholang::sorter::par_sort_matcher::ParSortMatcher;
use models::rust::rholang::sorter::sortable::Sortable;

#[test]
fn different_kinds_of_variables_should_bin_separately() {
    let par_vars = Par {
        exprs: vec![
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::BoundVar(2)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::Wildcard(WildcardMsg {})),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::BoundVar(1)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::FreeVar(0)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::FreeVar(2)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::BoundVar(0)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::FreeVar(1)),
                    }),
                })),
            },
        ],
        locally_free: create_bit_vector(&vec![2]),
        connective_used: true,
        ..Default::default()
    };

    let sorted_par_vars = Par {
        exprs: vec![
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::BoundVar(0)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::BoundVar(1)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::BoundVar(2)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::FreeVar(0)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::FreeVar(1)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::FreeVar(2)),
                    }),
                })),
            },
            Expr {
                expr_instance: Some(ExprInstance::EVarBody(EVar {
                    v: Some(Var {
                        var_instance: Some(VarInstance::Wildcard(WildcardMsg {})),
                    }),
                })),
            },
        ],
        locally_free: create_bit_vector(&vec![2]),
        connective_used: true,
        ..Default::default()
    };

    let result = ParSortMatcher::sort_match(&par_vars);
    assert_eq!(
        result.term, sorted_par_vars,
        "Variables were not sorted correctly"
    );
}
