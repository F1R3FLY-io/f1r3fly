use std::{
    cmp::Eq,
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    hash::Hash,
};

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/matcher/MaximumBipartiteMatch.scala
pub struct MaximumBipartiteMatch<P, T, R>
where
    P: Debug + Clone,
    T: Debug + Clone + Hash + Eq,
    R: Debug + Clone,
{
    match_function: Box<dyn FnMut(P, T) -> Option<R>>,
    matches: BTreeMap<Candidate<T>, (Pattern<P, T>, R)>,
    seen_targets: BTreeSet<Candidate<T>>,
}

type Pattern<P, T> = (P, Vec<Candidate<T>>);
type Candidate<T> = Indexed<T>;

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
struct Indexed<A> {
    value: A,
    index: usize,
}

impl<P, T, R> MaximumBipartiteMatch<P, T, R>
where
    P: Debug + Clone,
    T: Debug + Clone + Hash + Eq + Ord,
    R: Debug + Clone,
{
    pub fn new(match_function: Box<dyn FnMut(P, T) -> Option<R>>) -> Self {
        MaximumBipartiteMatch {
            match_function,
            matches: BTreeMap::new(),
            seen_targets: BTreeSet::new(),
        }
    }

    pub fn find_matches(&mut self, patterns: Vec<P>, targets: Vec<T>) -> Option<Vec<(T, P, R)>> {
        // println!("\nHit find_matches");
        // println!("\ntargets in find_matches: {:#?}", targets);
        // println!("\npatterns in find_matches: {:#?}", patterns);

        let ts: Vec<Candidate<T>> = targets
            .into_iter()
            .enumerate()
            .map(|(index, value)| Indexed { value, index })
            .collect();

        let ps: Vec<Pattern<P, T>> = patterns
            .into_iter()
            .map(|pattern| (pattern, ts.clone()))
            .collect();

        // println!("\nts: {:#?}", ts);
        // println!("\nps: {:#?}", ps);

        for pattern in ps {
            self.reset_seen();
            if !self.find_match(pattern) {
                // println!("\nreturning None in find_matches");
                return None;
            }
        }

        Some(
            self.matches
                .iter()
                .map(|(t, (p, r))| (t.value.clone(), p.0.clone(), r.clone()))
                .collect(),
        )
    }

    fn find_match(&mut self, pattern: Pattern<P, T>) -> bool {
        // println!("\nHit find_match");
        // println!("\nfind_match pattern: {:?}", pattern);

        match pattern.clone() {
            (_, candidates) if candidates.is_empty() => {
                // println!("\ncandidates empty");
                false
            }
            (p, candidates) => {
                if let Some((candidate, candidates)) = candidates.split_first() {
                    // println!("\ncandidate: {:?}", _candidate);
                    // println!("\ncandidates: {:?}", _candidates);

                    if self.not_seen(candidate.clone()) {
                        match (self.match_function)(p.clone(), candidate.clone().value) {
                            Some(match_result) => {
                                // println!("\nadding seen");
                                self.add_seen(candidate.clone());
                                self.try_claim_match(candidate.clone(), pattern, match_result)
                            }
                            None => {
                                // println!("\nthis candidate doesn't match, proceed to the others");
                                self.find_match((p, candidates.to_vec()))
                            }
                        }
                    } else {
                        self.find_match((p, candidates.to_vec()))
                    }
                } else {
                    // This should never happen
                    // println!("\nthis should never happen");
                    false
                }
            }
        }
    }

    fn try_claim_match(
        &mut self,
        candidate: Candidate<T>,
        pattern: Pattern<P, T>,
        result: R,
    ) -> bool {
        // println!("\nhit try_claim_match");
        match self.get_match(candidate.clone()) {
            None => {
                // we're first, we claim a match
                self.claim_match(candidate, pattern, result);
                true
            }
            Some(previous_pattern) => {
                // try to find a different match for the previous pattern
                if self.find_match(previous_pattern) {
                    // if found, we can match current pattern with this candidate despite it being taken
                    self.claim_match(candidate, pattern, result);
                    true
                } else {
                    // else, current pattern can't be matched with this candidate given the current matches, try others
                    self.find_match(pattern)
                }
            }
        }
    }

    fn reset_seen(&mut self) -> () {
        self.seen_targets.clear();
    }

    fn not_seen(&self, candidate: Candidate<T>) -> bool {
        !self.seen_targets.contains(&candidate)
    }

    fn add_seen(&mut self, candidate: Candidate<T>) -> () {
        self.seen_targets.insert(candidate);
    }

    fn get_match(&self, candidate: Candidate<T>) -> Option<Pattern<P, T>> {
        self.matches.get(&candidate).map(|x| x.0.clone())
    }

    fn claim_match(&mut self, candidate: Candidate<T>, pattern: Pattern<P, T>, result: R) -> () {
        let new_match = (candidate, (pattern, result));
        self.matches.insert(new_match.0, new_match.1);
    }
}
