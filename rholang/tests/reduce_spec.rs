// See rholang/src/test/scala/coop/rchain/rholang/interpreter/ReduceSpec.scala

use std::collections::{BTreeSet, HashMap};

use models::{
    rhoapi::{
        tagged_continuation::TaggedCont, BindPattern, ListParWithRandom, Par, ParWithRandom,
        TaggedContinuation,
    },
    rust::utils::{new_eplus_par, new_gint_expr},
};
use rholang::rust::interpreter::{
    env::Env, test_utils::persistent_store_tester::create_test_space,
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

#[tokio::test]
async fn eval_expr_should_handle_simple_addition() {
    let (_, reducer) = create_test_space().await;
    let add_expr = new_eplus_par(7, 8, Vec::new(), false);
    let env: Env<Par> = Env::new();
    let result = reducer.eval_expr(&add_expr, &env);
    println!("{:?}", result);
    let expected = vec![new_gint_expr(15)];

    assert!(result.is_ok());
    assert_eq!(result.unwrap().exprs, expected);
}
