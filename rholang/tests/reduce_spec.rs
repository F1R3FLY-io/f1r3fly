// See rholang/src/test/scala/coop/rchain/rholang/interpreter/ReduceSpec.scala

use std::{
    collections::{BTreeSet, HashMap},
    i64,
};

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::{
    rhoapi::{expr::ExprInstance, Bundle, EEq, Receive, ReceiveBind},
    rust::{
        rholang::implicits::GPrivateBuilder,
        utils::{new_boundvar_par, new_bundle_par, new_gbool_expr, new_gstring_par},
    },
};
use models::{
    rhoapi::{
        tagged_continuation::TaggedCont, BindPattern, Expr, ListParWithRandom, Par, ParWithRandom,
        Send, TaggedContinuation,
    },
    rust::utils::{new_eplus_par, new_gint_expr, new_gint_par},
};
use rholang::rust::interpreter::{
    env::Env, errors::InterpreterError, reduce::Reduce,
    test_utils::persistent_store_tester::create_test_space,
};
use rspace_plus_plus::rspace::internal::{Datum, Row, WaitingContinuation};

fn rand() -> Blake2b512Random {
    Blake2b512Random::create(&Vec::new(), 0, 0)
}

struct DataMapEntry {
    data: Vec<Par>,
    rand: Blake2b512Random,
}

fn map_data(
    elements: HashMap<Par, (Vec<Par>, Blake2b512Random)>,
) -> HashMap<Vec<Par>, Row<BindPattern, ListParWithRandom, TaggedContinuation>> {
    let mapped_entries = elements.into_iter().map(|(channel, (data, rand))| {
        let entry = DataMapEntry { data, rand };

        let row = Row {
            data: vec![Datum::create(
                channel.clone(),
                ListParWithRandom {
                    pars: entry.data.clone(),
                    random_state: entry.rand.to_vec(),
                },
                false,
            )],
            wks: vec![],
        };

        (vec![channel], row)
    });

    mapped_entries.collect()
}

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
    let expected = vec![new_gint_expr(15)];

    assert!(result.is_ok());
    assert_eq!(result.unwrap().exprs, expected);
}

#[tokio::test]
async fn eval_expr_should_handle_long_addition() {
    let (_, reducer) = create_test_space().await;
    let add_expr = new_eplus_par(i64::MAX, i64::MAX, Vec::new(), false);
    let env: Env<Par> = Env::new();
    let result = reducer.eval_expr(&add_expr, &env);
    let expected = vec![new_gint_expr(i64::MAX.wrapping_mul(2))];

    assert!(result.is_ok());
    assert_eq!(result.unwrap().exprs, expected);
}

#[tokio::test]
async fn eval_expr_should_leave_ground_values_alone() {
    let (_, reducer) = create_test_space().await;
    let ground_expr = new_gint_par(7, Vec::new(), false);
    let env: Env<Par> = Env::new();
    let result = reducer.eval_expr(&ground_expr, &env);
    let expected = vec![new_gint_expr(7)];

    assert!(result.is_ok());
    assert_eq!(result.unwrap().exprs, expected);
}

#[tokio::test]
async fn eval_expr_should_handle_equality_between_arbitary_processes() {
    let (_, reducer) = create_test_space().await;
    let eq_expr = Par::default().with_exprs(vec![Expr {
        expr_instance: Some(ExprInstance::EEqBody(EEq {
            p1: Some(GPrivateBuilder::new_par(String::from("private_name"))),
            p2: Some(GPrivateBuilder::new_par(String::from("private_name"))),
        })),
    }]);
    let env: Env<Par> = Env::new();
    let result = reducer.eval_expr(&eq_expr, &env);
    let expected = vec![new_gbool_expr(true)];

    assert!(result.is_ok());
    assert_eq!(result.unwrap().exprs, expected);
}

#[tokio::test]
async fn eval_expr_should_substitute_before_comparison() {
    let (_, reducer) = create_test_space().await;

    let eq_expr = Par::default().with_exprs(vec![Expr {
        expr_instance: Some(ExprInstance::EEqBody(EEq {
            p1: Some(new_boundvar_par(0, Vec::new(), false)),
            p2: Some(new_boundvar_par(1, Vec::new(), false)),
        })),
    }]);

    let mut env: Env<Par> = Env::new();
    env = env.put(Par::default());
    env = env.put(Par::default());

    let result = reducer.eval_expr(&eq_expr, &env);
    println!("{:?}", result);
    let expected = vec![new_gbool_expr(true)];

    assert!(result.is_ok());
    assert_eq!(result.unwrap().exprs, expected);
}

#[tokio::test]
async fn eval_of_bundle_should_evaluate_contents_of_bundle() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let channel = new_gstring_par(String::from("channel"), Vec::new(), false);
    let bundle_send = Par::default().with_bundles(vec![Bundle {
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(channel.clone()),
            data: vec![
                new_gint_par(7, Vec::new(), false),
                new_gint_par(8, Vec::new(), false),
                new_gint_par(9, Vec::new(), false),
            ],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        write_flag: false,
        read_flag: false,
    }]);

    let env: Env<Par> = Env::new();
    let result = reducer.eval(bundle_send, &env, split_rand.clone()).await;
    assert!(result.is_ok());
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        channel,
        (
            vec![
                new_gint_par(7, Vec::new(), false),
                new_gint_par(8, Vec::new(), false),
                new_gint_par(9, Vec::new(), false),
            ],
            split_rand,
        ),
    );
    let expected_result = map_data(expected_elements);
    assert_eq!(space.lock().unwrap().to_map(), expected_result);
}

#[tokio::test]
async fn eval_of_bundle_should_throw_an_error_if_names_are_used_against_their_polarity() {
    let (space, reducer) = create_test_space().await;

    /* for (n <- @bundle+ { y } ) { }  -> for (n <- y) { }
     */
    let y = new_gstring_par(String::from("y"), Vec::new(), false);
    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![Par::default()],
            source: Some(new_bundle_par(y, true, false)),
            remainder: None,
            free_count: 0,
        }],
        body: Some(Par::default()),
        persistent: false,
        peek: false,
        bind_count: 0,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    let result = reducer
        .eval(receive, &env, rand())
        .await
        .map(|_| space.lock().unwrap().to_map());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e,
            InterpreterError::ReduceError(String::from(
                "Trying to read from non-readable channel."
            ))
        );
    }
}
