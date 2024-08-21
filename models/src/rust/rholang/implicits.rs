use crate::rhoapi::{Expr, GUnforgeable, Par};

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
