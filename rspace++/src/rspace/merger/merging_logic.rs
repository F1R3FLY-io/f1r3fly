// See rspace/src/main/scala/coop/rchain/rspace/merger/MergingLogic.scala
// See rspace/src/test/scala/coop/rchain/rspace/merging/MergingLogicSpec.scala

use std::collections::{BTreeMap, HashMap, HashSet};
use shared::rust::hashable_set::HashableSet;

use crate::rspace::{
    hashing::blake2b256_hash::Blake2b256Hash,
    trace::event::{Consume, Produce},
};

use super::event_log_index::EventLogIndex;

pub type NumberChannelsEndVal = BTreeMap<Blake2b256Hash, i64>;

pub type NumberChannelsDiff = BTreeMap<Blake2b256Hash, i64>;

/// If target depends on source.
pub fn depends(target: &EventLogIndex, source: &EventLogIndex) -> bool {
    let produces_source: HashableSet<Produce> = HashableSet(
        produces_created_and_not_destroyed(source)
            .0
            .difference(&source.produces_mergeable.0)
            .cloned()
            .collect(),
    );

    let produces_target: HashableSet<Produce> = HashableSet(
        target
            .produces_consumed
            .0
            .difference(&source.produces_mergeable.0)
            .cloned()
            .collect(),
    );

    let consumes_source = consumes_created_and_not_destroyed(source);
    let consumes_target = &target.consumes_produced;

    let produces_depends: HashableSet<Produce> = HashableSet(
        produces_source
            .0
            .intersection(&produces_target.0)
            .cloned()
            .collect(),
    );

    let consumes_depends: HashableSet<Consume> = HashableSet(
        consumes_source
            .0
            .intersection(&consumes_target.0)
            .cloned()
            .collect(),
    );

    !produces_depends.0.is_empty() || !consumes_depends.0.is_empty()
}

/// If two event logs are conflicting.
pub fn are_conflicting(a: &EventLogIndex, b: &EventLogIndex) -> bool {
    !conflicts(a, b).is_empty()
}

/// Channels conflicting between a pair of event logs.
pub fn conflicts(a: &EventLogIndex, b: &EventLogIndex) -> Vec<Blake2b256Hash> {
    // Check #1
    // If the same produce or consume is destroyed in COMM in both branches, this might be a race.
    // All events created in event logs are unique, this match can be identified by comparing case classes.
    //
    // Produce is considered destroyed in COMM if it is not persistent and been consumed without peek.
    // Consume is considered destroyed in COMM when it is not persistent.
    //
    // If produces/consumes are mergeable in both indices, they are not considered as conflicts.
    let races_for_same_io_event = {
        let shared_consumes: HashableSet<Consume> = HashableSet(
            a.consumes_produced
                .0
                .intersection(&b.consumes_produced.0)
                .cloned()
                .collect(),
        );
        let mergeable_consumes: HashableSet<Consume> = HashableSet(
            a.consumes_mergeable
                .0
                .intersection(&b.consumes_mergeable.0)
                .cloned()
                .collect(),
        );
        let consume_races: Vec<Consume> = shared_consumes
            .0
            .difference(&mergeable_consumes.0)
            .filter(|c| !c.persistent)
            .cloned()
            .collect();

        let shared_produces: HashableSet<Produce> = HashableSet(
            a.produces_consumed
                .0
                .intersection(&b.produces_consumed.0)
                .cloned()
                .collect(),
        );
        let mergeable_produces: HashableSet<Produce> = HashableSet(
            a.produces_mergeable
                .0
                .intersection(&b.produces_mergeable.0)
                .cloned()
                .collect(),
        );
        let produce_races: Vec<Produce> = shared_produces
            .0
            .difference(&mergeable_produces.0)
            .filter(|p| !p.persistent)
            .cloned()
            .collect();

        let mut result = Vec::new();
        for consume in consume_races {
            result.extend(consume.channel_hashes.iter().cloned());
        }
        for produce in produce_races {
            result.push(produce.channel_hash.clone());
        }
        result
    };

    // Check #2
    // Events that are created inside branch and has not been destroyed in branch's COMMs
    // can lead to potential COMM during merge.
    let potential_comms = {
        // Helper function to check if consume matches produce
        fn match_found(consume: &Consume, produce: &Produce) -> bool {
            consume.channel_hashes.contains(&produce.channel_hash)
        }

        // Search for match in both directions
        fn check(left: &EventLogIndex, right: &EventLogIndex) -> Vec<Blake2b256Hash> {
            let p = produces_created_and_not_destroyed(left);
            let c = consumes_created_and_not_destroyed(right);
            let mut result = Vec::new();
            for produce in &p.0 {
                for consume in &c.0 {
                    if match_found(consume, produce) {
                        result.push(produce.channel_hash.clone());
                    }
                }
            }
            result
        }

        let mut result = check(a, b);
        result.extend(check(b, a));
        result
    };

    // Now we don't analyze joins and declare conflicting cases when produce touch join because applying
    // produces from both event logs might trigger continuation of some join, so COMM event
    let produce_touch_base_join = {
        let mut result = Vec::new();
        for produce in a
            .produces_touching_base_joins
            .0
            .iter()
            .chain(b.produces_touching_base_joins.0.iter())
        {
            result.push(produce.channel_hash.clone());
        }
        result
    };

    // Combine all conflicts
    let mut all_conflicts = Vec::new();
    all_conflicts.extend(races_for_same_io_event);
    all_conflicts.extend(potential_comms);
    all_conflicts.extend(produce_touch_base_join);
    all_conflicts
}

/// Produce created inside event log.
pub fn produces_created(e: &EventLogIndex) -> HashableSet<Produce> {
    let mut result: HashSet<Produce> = e
        .produces_linear
        .0
        .union(&e.produces_persistent.0)
        .cloned()
        .collect();

    for produce in &e.produces_copied_by_peek.0 {
        result.remove(produce);
    }
    HashableSet(result)
}

/// Consume created inside event log.
pub fn consumes_created(e: &EventLogIndex) -> HashableSet<Consume> {
    HashableSet(
        e.consumes_linear_and_peeks
            .0
            .union(&e.consumes_persistent.0)
            .cloned()
            .collect(),
    )
}

/// Produces that are created inside event log and not destroyed via COMM inside event log.
pub fn produces_created_and_not_destroyed(e: &EventLogIndex) -> HashableSet<Produce> {
    let linear_not_consumed: HashSet<Produce> = e
        .produces_linear
        .0
        .difference(&e.produces_consumed.0)
        .cloned()
        .collect();
    let combined: HashSet<Produce> = linear_not_consumed
        .union(&e.produces_persistent.0)
        .cloned()
        .collect();

    HashableSet(
        combined
            .difference(&e.produces_copied_by_peek.0)
            .cloned()
            .collect(),
    )
}

/// Consumes that are created inside event log and not destroyed via COMM inside event log.
pub fn consumes_created_and_not_destroyed(e: &EventLogIndex) -> HashableSet<Consume> {
    let linear_not_produced: HashSet<Consume> = e
        .consumes_linear_and_peeks
        .0
        .difference(&e.consumes_produced.0)
        .cloned()
        .collect();
    HashableSet(
        linear_not_produced
            .union(&e.consumes_persistent.0)
            .cloned()
            .collect(),
    )
}

/// Produces that are affected by event log - locally created + external destroyed.
pub fn produces_affected(e: &EventLogIndex) -> HashableSet<Produce> {
    let created = produces_created(e);
    let external_produces_destroyed: HashableSet<Produce> = HashableSet(
        e.produces_consumed
            .0
            .difference(&created.0)
            .filter(|p| !p.persistent)
            .cloned()
            .collect(),
    );

    HashableSet(
        produces_created_and_not_destroyed(e)
            .0
            .union(&external_produces_destroyed.0)
            .cloned()
            .collect(),
    )
}

/// Consumes that are affected by event log - locally created + external destroyed.
pub fn consumes_affected(e: &EventLogIndex) -> HashableSet<Consume> {
    let created = consumes_created(e);
    let external_consumes_destroyed: HashableSet<Consume> = HashableSet(
        e.consumes_produced
            .0
            .difference(&created.0)
            .filter(|c| !c.persistent)
            .cloned()
            .collect(),
    );

    HashableSet(
        consumes_created_and_not_destroyed(e)
            .0
            .union(&external_consumes_destroyed.0)
            .cloned()
            .collect(),
    )
}

/// If produce is copied by peek in one index and originated in another - it is considered as created in aggregate.
pub fn combine_produces_copied_by_peek(
    x: &EventLogIndex,
    y: &EventLogIndex,
) -> HashableSet<Produce> {
    let combined_copied_by_peek: HashableSet<Produce> = HashableSet(
        x.produces_copied_by_peek
            .0
            .union(&y.produces_copied_by_peek.0)
            .cloned()
            .collect(),
    );

    let combined_created: HashableSet<Produce> = HashableSet(
        produces_created(x)
            .0
            .union(&produces_created(y).0)
            .cloned()
            .collect(),
    );

    HashableSet(
        combined_copied_by_peek
            .0
            .difference(&combined_created.0)
            .cloned()
            .collect(),
    )
}

/// Arrange list[v] into map v -> Vec[v] for items that match predicate.
/// NOTE: predicate here is forced to be non directional.
/// If either (a,b) or (b,a) is true, both relations are recorded as true.
/// TODO: adjust once dependency graph is implemented for branch computing - OLD
pub fn compute_relation_map<A: Eq + std::hash::Hash + Clone>(
    items: &HashableSet<A>,
    relation: impl Fn(&A, &A) -> bool,
) -> HashMap<A, HashableSet<A>> {
    let mut init: HashMap<A, HashableSet<A>> = items
        .0
        .iter()
        .map(|item| (item.clone(), HashableSet(HashSet::new())))
        .collect();

    let items_vec: Vec<A> = items.0.iter().cloned().collect();
    for i in 0..items_vec.len() {
        for j in (i + 1)..items_vec.len() {
            let l = &items_vec[i];
            let r = &items_vec[j];

            if relation(l, r) || relation(r, l) {
                if let Some(l_set) = init.get_mut(l) {
                    l_set.0.insert(r.clone());
                }
                if let Some(r_set) = init.get_mut(r) {
                    r_set.0.insert(l.clone());
                }
            }
        }
    }

    init
}

/// Given relation map, return sets of related items.
pub fn gather_related_sets<A: Eq + std::hash::Hash + Clone>(
    relation_map: &HashMap<A, HashableSet<A>>,
) -> Vec<HashableSet<A>> {
    fn add_relations<A: Eq + std::hash::Hash + Clone>(
        to_add: &HashableSet<A>,
        acc: &HashableSet<A>,
        relation_map: &HashMap<A, HashableSet<A>>,
    ) -> HashableSet<A> {
        // stop if all new dependencies are already in set
        let mut next = acc.clone();
        for item in &to_add.0 {
            next.0.insert(item.clone());
        }

        let stop = next.0.len() == acc.0.len();
        if stop {
            acc.clone()
        } else {
            let mut n = HashSet::new();
            for v in &to_add.0 {
                if let Some(related) = relation_map.get(v) {
                    for r in &related.0 {
                        n.insert(r.clone());
                    }
                }
            }
            add_relations(&HashableSet(n), &next, relation_map)
        }
    }

    let mut result = Vec::new();
    for k in relation_map.keys() {
        let related = relation_map.get(k).unwrap();
        let mut start = HashSet::new();
        start.insert(k.clone());
        let set = add_relations(&related, &HashableSet(start), relation_map);

        // Check if this set is already in result to avoid duplicates
        if !result.iter().any(|existing_set: &HashableSet<A>| {
            existing_set.0.len() == set.0.len()
                && existing_set.0.iter().all(|item| set.0.contains(item))
        }) {
            result.push(set);
        }
    }

    result
}

/// Compute related sets directly from items and relation
pub fn compute_related_sets<A: Eq + std::hash::Hash + Clone>(
    items: &HashableSet<A>,
    relation: impl Fn(&A, &A) -> bool,
) -> Vec<HashableSet<A>> {
    let relation_map = compute_relation_map(items, relation);
    gather_related_sets(&relation_map)
}

/// Given conflicts map, output possible rejection options.
pub fn compute_rejection_options<A: Eq + std::hash::Hash + Clone>(
    conflict_map: &HashMap<A, HashableSet<A>>,
) -> Vec<HashableSet<A>> {
    // Set of rejection paths with corresponding remaining conflicts map
    #[derive(Clone)]
    struct RejectionOption<A: Eq + std::hash::Hash + Clone> {
        rejected_so_far: HashableSet<A>,
        remaining_conflicts_map: HashMap<A, HashableSet<A>>,
    }

    fn gather_rej_options<A: Eq + std::hash::Hash + Clone>(
        conflicts_map: &HashMap<A, HashableSet<A>>,
    ) -> Vec<RejectionOption<A>> {
        let mut result = Vec::new();

        for (_, to_reject) in conflicts_map {
            // keeping each key - reject conflicting values
            let mut remaining_conflicts_map = HashMap::new();

            for (key, conflicts) in conflicts_map {
                if !to_reject.0.contains(key) {
                    // Filter out rejected items from conflicts
                    let updated_conflicts: HashSet<A> = conflicts
                        .0
                        .iter()
                        .filter(|c| !to_reject.0.contains(c))
                        .cloned()
                        .collect();

                    // Only include keys that still have conflicts
                    if !updated_conflicts.is_empty() {
                        remaining_conflicts_map.insert(key.clone(), HashableSet(updated_conflicts));
                    }
                }
            }

            let option = RejectionOption {
                rejected_so_far: to_reject.clone(),
                remaining_conflicts_map,
            };

            // Only add if not already in result
            if !result.iter().any(|existing: &RejectionOption<A>| {
                existing.rejected_so_far.0 == option.rejected_so_far.0
            }) {
                result.push(option);
            }
        }

        result
    }

    // Only keys that have conflicts associated should be examined
    let mut conflicts_only_map = HashMap::new();
    for (key, conflicts) in conflict_map {
        if !conflicts.0.is_empty() {
            conflicts_only_map.insert(key.clone(), conflicts.clone());
        }
    }

    // Start with rejecting nothing and full conflicts map
    let start = RejectionOption {
        rejected_so_far: HashableSet(HashSet::new()),
        remaining_conflicts_map: conflicts_only_map,
    };

    let mut result = Vec::new();
    let mut current = vec![start];

    while !current.is_empty() {
        let mut next = Vec::new();

        for option in current {
            if option.remaining_conflicts_map.is_empty() {
                // No more conflicts, this is a valid rejection option
                // Only add if not already in result
                if !result.iter().any(|existing_set: &HashableSet<A>| {
                    existing_set.0.len() == option.rejected_so_far.0.len()
                        && existing_set
                            .0
                            .iter()
                            .all(|item| option.rejected_so_far.0.contains(item))
                }) {
                    result.push(option.rejected_so_far);
                }
            } else {
                // Continue resolving conflicts
                for mut new_option in gather_rej_options(&option.remaining_conflicts_map) {
                    // Add previously rejected items
                    for item in &option.rejected_so_far.0 {
                        new_option.rejected_so_far.0.insert(item.clone());
                    }
                    next.push(new_option);
                }
            }
        }

        current = next;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_compute_rejection_options() {
        // Test 1
        let mut map1: HashMap<i32, HashableSet<i32>> = HashMap::new();
        map1.insert(1, HashableSet(HashSet::from_iter(vec![2, 3, 4])));
        map1.insert(2, HashableSet(HashSet::from_iter(vec![1])));
        map1.insert(3, HashableSet(HashSet::from_iter(vec![1, 2])));
        map1.insert(4, HashableSet(HashSet::from_iter(vec![1])));

        let result1 = compute_rejection_options(&map1);
        assert_eq!(result1.len(), 2);
        assert!(
            result1
                .iter()
                .any(|set| set.0.len() == 2 && set.0.contains(&1) && set.0.contains(&2))
        );
        assert!(result1.iter().any(|set| set.0.len() == 3
            && set.0.contains(&2)
            && set.0.contains(&3)
            && set.0.contains(&4)));

        // Test 2
        let mut map2: HashMap<i32, HashableSet<i32>> = HashMap::new();
        map2.insert(1, HashableSet(HashSet::from_iter(vec![2, 3, 4])));
        map2.insert(2, HashableSet(HashSet::from_iter(vec![1, 3, 4])));
        map2.insert(3, HashableSet(HashSet::from_iter(vec![1, 2, 4])));
        map2.insert(4, HashableSet(HashSet::from_iter(vec![1, 2, 3])));

        let result2 = compute_rejection_options(&map2);
        assert_eq!(result2.len(), 4);
        assert!(result2.iter().any(|set| set.0.len() == 3
            && set.0.contains(&2)
            && set.0.contains(&3)
            && set.0.contains(&4)));
        assert!(result2.iter().any(|set| set.0.len() == 3
            && set.0.contains(&1)
            && set.0.contains(&3)
            && set.0.contains(&4)));
        assert!(result2.iter().any(|set| set.0.len() == 3
            && set.0.contains(&1)
            && set.0.contains(&2)
            && set.0.contains(&4)));
        assert!(result2.iter().any(|set| set.0.len() == 3
            && set.0.contains(&1)
            && set.0.contains(&2)
            && set.0.contains(&3)));

        // Test 3
        let mut map3: HashMap<i32, HashableSet<i32>> = HashMap::new();
        map3.insert(1, HashableSet(HashSet::from_iter(vec![2, 3, 4])));
        map3.insert(2, HashableSet(HashSet::from_iter(vec![1])));
        map3.insert(3, HashableSet(HashSet::from_iter(vec![1, 4])));
        map3.insert(4, HashableSet(HashSet::from_iter(vec![1, 3])));

        let result3 = compute_rejection_options(&map3);
        assert_eq!(result3.len(), 3);
        assert!(result3.iter().any(|set| set.0.len() == 3
            && set.0.contains(&2)
            && set.0.contains(&3)
            && set.0.contains(&4)));
        assert!(
            result3
                .iter()
                .any(|set| set.0.len() == 2 && set.0.contains(&1) && set.0.contains(&3))
        );
        assert!(
            result3
                .iter()
                .any(|set| set.0.len() == 2 && set.0.contains(&1) && set.0.contains(&4))
        );

        // Test 4
        let mut map4: HashMap<i32, HashableSet<i32>> = HashMap::new();
        map4.insert(1, HashableSet(HashSet::new()));
        map4.insert(2, HashableSet(HashSet::from_iter(vec![3])));
        map4.insert(3, HashableSet(HashSet::from_iter(vec![2, 4])));
        map4.insert(4, HashableSet(HashSet::from_iter(vec![3])));

        let result4 = compute_rejection_options(&map4);
        assert_eq!(result4.len(), 2);
        assert!(
            result4
                .iter()
                .any(|set| set.0.len() == 1 && set.0.contains(&3))
        );
        assert!(
            result4
                .iter()
                .any(|set| set.0.len() == 2 && set.0.contains(&2) && set.0.contains(&4))
        );

        let all: HashSet<i32> = (1..=1000).collect();
        let mut map5: HashMap<i32, HashableSet<i32>> = HashMap::new();
        for i in 1..=1000 {
            let mut conflicts = all.clone();
            conflicts.remove(&i);
            map5.insert(i, HashableSet(conflicts));
        }

        let result5 = compute_rejection_options(&map5);
        assert_eq!(result5.len(), 1000);
        for i in 1..=1000 {
            let mut expected = all.clone();
            expected.remove(&i);
            assert!(result5.iter().any(|set| {
                set.0.len() == 999
                    && !set.0.contains(&i)
                    && (1..=1000).filter(|j| *j != i).all(|j| set.0.contains(&j))
            }));
        }
    }

    #[test]
    fn test_compute_related_sets() {
        // Test relation: numbers with the same parity (both odd or both even)
        let items: HashableSet<i32> = HashableSet(HashSet::from_iter(vec![1, 2, 3, 4, 5]));
        let same_parity = |a: &i32, b: &i32| a % 2 == b % 2;

        let result = compute_related_sets(&items, same_parity);
        assert_eq!(result.len(), 2);

        // Should have one set with odd numbers and one with even numbers
        let mut found_odd = false;
        let mut found_even = false;

        for set in result {
            if set.0.len() == 3 && set.0.contains(&1) && set.0.contains(&3) && set.0.contains(&5) {
                found_odd = true;
            }
            if set.0.len() == 2 && set.0.contains(&2) && set.0.contains(&4) {
                found_even = true;
            }
        }

        assert!(found_odd);
        assert!(found_even);
    }

    #[test]
    fn test_relation_map() {
        // Test creating relation map for divisibility relationship
        let items: HashableSet<i32> = HashableSet(HashSet::from_iter(vec![2, 3, 4, 6, 12]));
        let is_divisible = |a: &i32, b: &i32| b % a == 0;

        let relation_map = compute_relation_map(&items, is_divisible);

        // Check a few key relationships
        assert!(relation_map.get(&2).unwrap().0.contains(&4));
        assert!(relation_map.get(&2).unwrap().0.contains(&6));
        assert!(relation_map.get(&2).unwrap().0.contains(&12));

        assert!(relation_map.get(&3).unwrap().0.contains(&6));
        assert!(relation_map.get(&3).unwrap().0.contains(&12));

        assert!(!relation_map.get(&3).unwrap().0.contains(&4));
        assert!(!relation_map.get(&4).unwrap().0.contains(&6));
    }
}
