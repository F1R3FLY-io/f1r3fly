use models::{rhoapi::Par, rust::utils::FreeMap};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub enum Pattern<T: Clone> {
    Term(T),
    Remainder(i32),
}

pub trait ListMatch<T: Clone> {
    // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - listMatchSingle
    fn list_match_single(&mut self, tlist: Vec<T>, plist: Vec<T>) -> Option<()>;

    // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - listMatchSingle_
    fn list_match_single_(
        &mut self,
        tlist: Vec<T>,
        plist: Vec<T>,
        merger: &dyn Fn(Par, Vec<T>) -> Par,
        remainder: Option<i32>,
        wildcard: bool,
    ) -> Option<()>;

    // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - listMatch
    fn list_match(
        &mut self,
        targets: Vec<T>,
        patterns: Vec<T>,
        merger: &dyn Fn(Par, Vec<T>) -> Par,
        remainder: Option<i32>,
        wildcard: bool,
    ) -> Option<()>;

    // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - handleRemainder
    fn handle_remainder(
        &mut self,
        remainder_targets: Vec<T>,
        level: i32,
        merger: &dyn Fn(Par, Vec<T>) -> Par,
    ) -> Option<()>;

    fn match_function(&mut self, pattern: Pattern<T>, t: T) -> Option<FreeMap>;
}

// Rust extension doesn't auto-format custom macros.
#[macro_export]
macro_rules! list_match {
  ($($type:ty),*) => {
      $(
          impl ListMatch<$type> for SpatialMatcherContext {
              // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - listMatchSingle
              fn list_match_single(&mut self, tlist: Vec<$type>, plist: Vec<$type>) -> Option<()> {
                let _merger: &dyn Fn(Par, Vec<$type>) -> Par = &|p, _| p;

                self.list_match_single_(tlist, plist, _merger, None, false)
              }

              // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - listMatchSingle_
              fn list_match_single_(
                  &mut self,
                  tlist: Vec<$type>,
                  plist: Vec<$type>,
                  merger: &dyn Fn(Par, Vec<$type>) -> Par,
                  remainder: Option<i32>,
                  wildcard: bool,
              ) -> Option<()> {

                // println!("\nHit list_match_single_");
                // println!("\ntlist: {:#?}", tlist);
                // println!("\nplist: {:#?}", plist);
                // let merger_type_name = std::any::type_name::<&dyn Fn(Par, Vec<$type>) -> Par>();
                // println!("merger: {:?}", merger_type_name);
                // println!("remainder: {:#?}", remainder);
                // println!("wildcard: {:#?}", wildcard);

                let exact_match = !wildcard && remainder.is_none();
                let plen = plist.len();
                let tlen = tlist.len();

                let result: Option<()> = if exact_match && plen != tlen {
                    // println!("\nreturning None in list_match_single_");
                    None
                } else if plen > tlen {
                    // println!("\nreturning None in list_match_single_");
                    None
                } else if plen == 0 && tlen == 0 && remainder.is_none() {
                    // println!("\ncurrent free_map: {:?}", self.free_map);
                    // println!("\nreturning Some in list_match_single_");
                    Some(())
                } else if plen == 0 && remainder.is_some() {
                    if tlist
                        .iter()
                        .all(|t| self.locally_free(t.to_owned(), 0).is_empty())
                    {
                        self.handle_remainder(tlist, remainder.unwrap(), merger)
                    } else {
                        // println!("\nreturning None in list_match_single_");
                        None
                    }
                } else {
                    // println!("calling list_match");
                    self.list_match(tlist, plist, merger, remainder, wildcard)
                };
                // println!("\nend of list_match_single_");
                result
              }

              // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - listMatch
              fn list_match(
                  &mut self,
                  targets: Vec<$type>,
                  patterns: Vec<$type>,
                  merger: &dyn Fn(Par, Vec<$type>) -> Par,
                  remainder: Option<i32>,
                  wildcard: bool,
              ) -> Option<()> {
                // println!("\nHit list_match");
                // println!("\nlist_match targets: {:?}", targets);
                // println!("\nlist_match patterns: {:?}", patterns);

                let remainder_patterns: Vec<Pattern<$type>> = remainder
                    .map(|level| vec![Pattern::Remainder(level); targets.len() - patterns.len()])
                    .unwrap_or(Vec::new());
                let mut all_patterns: Vec<Pattern<$type>> = remainder_patterns;
                all_patterns.extend(patterns.clone().into_iter().map(Pattern::Term));

                // println!("\nlist_match all_patterns: {:?}", all_patterns);

                let mut cloned_self = self.clone();
                let _match_function = Box::new(move |pattern, t| cloned_self.match_function(pattern, t));
                // NOTE: Bypassing 'memoizeInHashMap' here
                let mut maximum_bipartite_match: MaximumBipartiteMatch<Pattern<$type>, $type, FreeMap> = MaximumBipartiteMatch::new(_match_function);

                // println!("\ncurrent free_map: {:?}", self.free_map);

                let matches = maximum_bipartite_match.find_matches(all_patterns, targets.clone())?;

                // println!("\nfree_map after MBM: {:?}", self.free_map);
                // println!("\nmatches: {:?}", matches);

                let free_maps: Vec<FreeMap> = matches
                    .iter()
                    .map(|(_, _, free_map)| free_map.clone())
                    .collect();

                let updated_free_map = aggregate_updates(self.free_map.clone(), free_maps)?;
                // println!("\nsetting free_map in list_match");
                self.free_map = updated_free_map;
                // println!("\nnew free_map: {:?}", self.free_map);

                let remainder_targets: Vec<$type> = matches
                    .iter()
                    .filter_map(|(target, pattern, _)| match pattern {
                        Pattern::Remainder(_) => Some(target.clone()),
                        _ => None,
                    })
                    .collect();

                let remainder_targets_sorted: Vec<$type> = targets
                    .iter()
                    .filter(|target| remainder_targets.contains(target))
                    .cloned()
                    .collect();

                match remainder {
                    None => {
                        if wildcard || remainder_targets_sorted.is_empty() {
                            Some(())
                        } else {
                            None
                        }
                    }
                    Some(level) => self.handle_remainder(remainder_targets_sorted, level, merger),
                }
              }

              // See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/SpatialMatcher.scala - handleRemainder
              fn handle_remainder(
                  &mut self,
                  remainder_targets: Vec<$type>,
                  level: i32,
                  merger: &dyn Fn(Par, Vec<$type>) -> Par,
              ) -> Option<()> {
                // println!("\nhit handle_remainder");
                // println!("remainder_targets: {:#?}", remainder_targets);
                // println!("level: {:#?}", level);
                // let merger_type_name = std::any::type_name::<&dyn Fn(Par, Vec<$type>) -> Par>();
                // println!("merger: {:?}\n", merger_type_name);

                let remainder_par = self
                .free_map
                .clone()
                .get(&level)
                .cloned()
                .unwrap_or(vector_par(Vec::new(), false));

                // println!("\nremainder_par: {:#?}", remainder_par);

                let remainder_par_updated = merger(remainder_par, remainder_targets);

                // println!("\nremainder_par_updated: {:#?}", remainder_par_updated);

                // println!("\nmodifying free_map in handle_remainder");
                self.free_map.insert(level, remainder_par_updated);
                // println!("\ncurrent free_map: {:#?}", self.free_map);

                Some(())
              }

              /*
                  'The Box is used here to store the function on the heap rather than the stack. This is because the size of the function is not known at
                compile time (it's a closure that captures its environment), so it cannot be stored directly on the stack. The Box provides a fixed-size
                pointer to the function on the heap, which can be stored on the stack.' - GPT-4
              */
              fn match_function(&mut self, pattern: Pattern<$type>, t: $type) -> Option<FreeMap> {
                // println!("\nCalling match_function");
                // println!("matchFunction pattern: {:#?}", pattern);

                let match_effect: Option<()> = match pattern {
                  Pattern::Term(p) => {
                     if !self.connective_used(p.clone()) {
                         guard(t == p)
                      } else {
                        // println!("\ncalling spatial_match in match_function");
                        // println!("\nt in match_function: {:#?}", t);
                        // println!("\np in match_function: {:#?}", p);
                         self.spatial_match(t, p)
                      }
                  }
                  Pattern::Remainder(_) => guard(self.locally_free(t, 0).is_empty()),
                };

                match match_effect {
                  Some(_) => {
                    let free_map = &self.free_map;
                    Some(free_map.clone())},
                  None => None,
                }
              }
          }
      )*
  };
}

pub fn aggregate_updates(current_free_map: FreeMap, free_maps: Vec<FreeMap>) -> Option<FreeMap> {
    let current_vars: HashSet<_> = current_free_map.keys().cloned().collect();
    let added_vars: HashSet<_> = free_maps
        .iter()
        .flat_map(|m| m.keys())
        .filter(|k| !current_vars.contains(*k))
        .cloned()
        .collect();

    // The correctness of isolating MBM from changing FreeMap relies
    // on our ability to aggregate the var assignments from subsequent matches.
    // This means all the variables populated by MBM must not duplicate each other.
    if added_vars.len() != added_vars.iter().collect::<HashSet<_>>().len() {
        panic!(
            "RUST ERROR: Aggregated updates conflicted with each other: {:?}",
            free_maps
        )
    }

    let updated_free_map = free_maps
        .into_iter()
        .fold(current_free_map.clone(), |mut acc, fm| {
            acc.extend(fm);
            acc
        });

    Some(updated_free_map)
}
