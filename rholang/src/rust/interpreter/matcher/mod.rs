use has_locally_free::HasLocallyFree;
use models::{
    rhoapi::{Connective, Expr, Par},
    rust::utils::union,
};

pub mod exports;
pub mod fold_match;
pub mod has_locally_free;
pub mod list_match;
pub mod r#match;
pub mod match_pars;
pub mod maximum_bipartite_match;
pub mod par_count;
pub mod spatial_matcher;
pub mod sub_pars;

// These two functions need to be under 'rholang' dir because of HasLocallyFree Trait.
// This trait should, I think, be moved to models

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - prepend
pub fn prepend_connective(mut p: Par, c: Connective, depth: i32) -> Par {
    let mut new_connectives = vec![c.clone()];
    new_connectives.append(&mut p.connectives);

    Par {
        connectives: new_connectives,
        locally_free: c.locally_free(c.clone(), depth),
        connective_used: p.connective_used || c.clone().connective_used(c),
        ..p.clone()
    }
}

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - prepend
pub fn prepend_expr(mut p: Par, e: Expr, depth: i32) -> Par {
    let mut new_exprs = vec![e.clone()];
    new_exprs.append(&mut p.exprs);

    Par {
        exprs: new_exprs,
        locally_free: union(p.locally_free.clone(), e.locally_free(e.clone(), depth)),
        connective_used: p.connective_used || e.clone().connective_used(e),
        ..p.clone()
    }
}
