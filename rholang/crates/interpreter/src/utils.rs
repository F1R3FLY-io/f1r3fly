pub mod address_tools;
pub mod base58;
pub mod rev_address;
pub mod test_utils;
use crate::normal_forms::{MyBitVec, Par};

use bitvec::vec::BitVec;
use models::rust::utils::union;

use crate::normal_forms::{Bundle, Expr, New};

use super::matcher::has_locally_free::HasLocallyFree;

// These two functions need to be under 'rholang' dir because of HasLocallyFree Trait.
// This trait should, I think, be moved to models

// See models/src/main/scala/coop/rchain/models/rholang/implicits.scala - prepend
pub fn prepend_connective(
    p: crate::normal_forms::Par,
    c: crate::normal_forms::Connective,
    depth: u32,
) -> crate::normal_forms::Par {
    let mut new_connectives = vec![c.clone()];
    new_connectives.extend(p.connectives.into_iter());

    crate::normal_forms::Par {
        connectives: new_connectives,
        locally_free: c.locally_free(depth),
        connective_used: p.connective_used || c.clone().connective_used(),
        ..p
    }
}

pub fn prepend_expr(mut p: Par, e: Expr, depth: u32) -> Par {
    let mut new_exprs = vec![e.clone()];
    new_exprs.append(&mut p.exprs);

    let bitset1 = p
        .locally_free
        .to_owned()
        .into_vec()
        .into_iter()
        .map(|v| v as u8)
        .collect::<Vec<u8>>();
    let bitset2 = e
        .locally_free(depth)
        .into_vec()
        .into_iter()
        .map(|v| v as u8)
        .collect::<Vec<u8>>();

    let locally_free = union(bitset1, bitset2)
        .into_iter()
        .map(|v| v as usize)
        .collect();

    Par {
        exprs: new_exprs,
        locally_free: MyBitVec::from_vec(locally_free),
        connective_used: p.connective_used || e.clone().connective_used(),
        ..p.clone()
    }
}

pub fn prepend_new<N, T>(mut p: Par, n: N) -> Par
where
    N: HasLocallyFree<T>,
    T: Clone,
{
    let mut new_news = vec![n.clone()];
    new_news.extend(p.news);

    let bitset1 = p
        .locally_free
        .clone()
        .into_vec()
        .into_iter()
        .map(|v| v as u8)
        .collect();
    let bitset2 = n
        .clone()
        .locally_free
        .clone()
        .into_vec()
        .into_iter()
        .map(|v| v as u8)
        .collect();

    let connective_used = p.connective_used || n.connective_used(n);

    Par {
        news: new_news,
        locally_free: union(bitset1, bitset2),
        connective_used,
        ..p.clone()
    }
}

pub fn prepend_bundle(mut p: Par, b: Bundle) -> Par {
    let mut new_bundles = vec![b.clone()];
    new_bundles.append(&mut p.bundles);

    Par {
        bundles: new_bundles,
        locally_free: union(p.locally_free.clone(), b.body.unwrap().locally_free),
        ..p.clone()
    }
}

// for locally_free parameter, in case when we have (bodyResult.par.locallyFree.from(boundCount).map(x => x - boundCount))
pub(crate) fn filter_and_adjust_bitset(bitset: Vec<u8>, bound_count: usize) -> Vec<u8> {
    bitset
        .into_iter()
        .enumerate()
        .filter_map(|(i, _)| {
            if i >= bound_count {
                Some(i as u8 - bound_count as u8)
            } else {
                None
            }
        })
        .collect()
}
