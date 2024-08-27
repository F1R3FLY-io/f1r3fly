use crate::{
    rhoapi::{Expr, GUnforgeable, Par},
    rust::utils::union,
};

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - object VectorPar
// Somehow they are not initializing 'locally_free' and 'connective_used' fields
pub fn vector_par(_locally_free: Vec<u8>, _connective_used: bool) -> Par {
    Par {
        sends: Vec::new(),
        receives: Vec::new(),
        news: Vec::new(),
        exprs: Vec::new(),
        matches: Vec::new(),
        unforgeables: Vec::new(),
        bundles: Vec::new(),
        connectives: Vec::new(),
        locally_free: _locally_free,
        connective_used: _connective_used,
    }
}

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - singleExpr
pub fn single_expr(p: &Par) -> Option<Expr> {
    if p.sends.is_empty()
        && p.receives.is_empty()
        && p.news.is_empty()
        && p.matches.is_empty()
        && p.bundles.is_empty()
    {
        match &p.exprs {
            vec if vec.len() == 1 => vec.get(0).cloned(),
            _ => None,
        }
    } else {
        None
    }
}

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - singleUnforgeable
pub fn single_unforgeable(p: &Par) -> Option<GUnforgeable> {
    if p.sends.is_empty()
        && p.receives.is_empty()
        && p.news.is_empty()
        && p.exprs.is_empty()
        && p.matches.is_empty()
        && p.bundles.is_empty()
        && p.connectives.is_empty()
    {
        match &p.unforgeables {
            vec if vec.len() == 1 => vec.get(0).cloned(),
            _ => None,
        }
    } else {
        None
    }
}

pub fn concatenate_pars(p: Par, that: Par) -> Par {
    Par {
        sends: that.sends.iter().chain(p.sends.iter()).cloned().collect(),
        receives: that
            .receives
            .iter()
            .chain(p.receives.iter())
            .cloned()
            .collect(),
        news: that.news.iter().chain(p.news.iter()).cloned().collect(),
        exprs: that.exprs.iter().chain(p.exprs.iter()).cloned().collect(),
        matches: that
            .matches
            .iter()
            .chain(p.matches.iter())
            .cloned()
            .collect(),
        unforgeables: that
            .unforgeables
            .iter()
            .chain(p.unforgeables.iter())
            .cloned()
            .collect(),
        bundles: that
            .bundles
            .iter()
            .chain(p.bundles.iter())
            .cloned()
            .collect(),
        connectives: that
            .connectives
            .iter()
            .chain(p.connectives.iter())
            .cloned()
            .collect(),
        locally_free: union(that.locally_free, p.locally_free),
        connective_used: that.connective_used || p.connective_used,
    }
}
