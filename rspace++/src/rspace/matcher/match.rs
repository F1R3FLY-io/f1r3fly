use super::{exports::*, fold_match::FoldMatch, spatial_matcher::SpatialMatcherContext};

/**
 * See rspace/src/main/scala/coop/rchain/rspace/Match.scala
 *
 * Type trait for matching patterns with data.
 *
 * @tparam P A type representing patterns
 * @tparam A A type representing data and match result
 */
pub trait Match<P, A> {
    fn get(&self, p: P, a: A) -> Option<A>;
}

#[derive(Clone, Default)]
pub struct Matcher;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/storage/package.scala - matchListPar
impl Match<BindPattern, ListParWithRandom> for Matcher {
    fn get(&self, pattern: BindPattern, data: ListParWithRandom) -> Option<ListParWithRandom> {
        let mut spatial_matcher = SpatialMatcherContext::new();

        let fold_match_result =
            spatial_matcher.fold_match(data.pars, pattern.patterns, pattern.remainder.clone());
        let match_result = match fold_match_result {
            Some(pars) => Some((spatial_matcher.free_map, pars)),
            None => None,
        };

        match match_result {
            Some((mut free_map, caught_rem)) => {
                let remainder_map = match pattern.remainder {
                    Some(Var {
                        var_instance: Some(FreeVar(level)),
                    }) => {
                        free_map.insert(
                            level,
                            vector_par(Vec::new(), false).with_exprs(vec![new_elist_expr(
                                caught_rem,
                                Vec::new(),
                                false,
                                None,
                            )]),
                        );
                        free_map
                    }
                    _ => free_map,
                };
                Some(ListParWithRandom {
                    pars: to_vec(remainder_map, pattern.free_count),
                    random_state: data.random_state,
                })
            }
            None => None,
        }
    }
}
