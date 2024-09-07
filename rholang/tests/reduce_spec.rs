// See rholang/src/test/scala/coop/rchain/rholang/interpreter/ReduceSpec.scala

use std::collections::{BTreeSet, HashMap};

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{
    tagged_continuation::TaggedCont, BindPattern, ListParWithRandom, Par, ParWithRandom,
    TaggedContinuation,
};
use rspace_plus_plus::rspace::internal::{Row, WaitingContinuation};

fn check_continuation(
    result: HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>>,
    channels: Vec<Par>,
    bind_patterns: Vec<BindPattern>,
    body: ParWithRandom,
) -> bool {
    let mut expected_result = HashMap::new();
    expected_result.insert(
        channels.clone(),
        Row {
            data: Vec::new(),
            wks: vec![WaitingContinuation::create(
                channels,
                bind_patterns,
                TaggedContinuation {
                    tagged_cont: Some(TaggedCont::ParBody(body)),
                },
                false,
                BTreeSet::new(),
            )],
        },
    );

    if result.len() != expected_result.len() {
        return false;
    }

    for (key, value) in &result {
        if let Some(expected_value) = expected_result.get(key) {
            if value != expected_value {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}


