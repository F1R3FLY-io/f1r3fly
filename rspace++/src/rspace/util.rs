use super::internal::{ContResult, RSpaceResult};

// See rspace/src/main/scala/coop/rchain/rspace/util/package.scala
pub fn unpack_option<C, P, K: Clone, R: Clone>(
    v: &Option<(ContResult<C, P, K>, Vec<RSpaceResult<C, R>>)>,
) -> Option<(K, Vec<R>)> {
    match v {
        Some(tuple) => Some(unpack_tuple(tuple)),
        None => None,
    }
}

pub fn unpack_tuple<C, P, K: Clone, R: Clone>(
    tuple: &(ContResult<C, P, K>, Vec<RSpaceResult<C, R>>),
) -> (K, Vec<R>) {
    match tuple {
        (ContResult { continuation, .. }, data) => (
            continuation.clone(),
            data.into_iter()
                .map(|result| result.matched_datum.clone())
                .collect(),
        ),
    }
}
