use models::{
    rhoapi::{Bundle, Connective, Expr, Match, New, Par, Receive, Send},
    rust::utils::union,
};

use super::matcher::has_locally_free::HasLocallyFree;

pub mod address_tools;
pub mod base58;
pub mod rev_address;

// Helper enum. This is 'GeneratedMessage' in Scala
#[derive(Clone)]
pub enum GeneratedMessage {
    Send(Send),
    Receive(Receive),
    New(New),
    Match(Match),
    Bundle(Bundle),
    Expr(Expr),
}

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