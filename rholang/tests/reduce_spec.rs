// See rholang/src/test/scala/coop/rchain/rholang/interpreter/ReduceSpec.scala

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    i64,
    sync::{Arc, RwLock},
};

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::{
    rhoapi::{
        expr::ExprInstance, g_unforgeable::UnfInstance, Bundle, EEq, EMethod, GPrivate,
        GUnforgeable, Match, MatchCase, New, Receive, ReceiveBind,
    },
    rust::{
        rholang::implicits::GPrivateBuilder,
        string_ops::StringOps,
        utils::{
            new_boundvar_par, new_bundle_par, new_elist_par, new_freevar_par, new_freevar_var,
            new_gbool_expr, new_gstring_par,
        },
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
    accounting::{cost_accounting::CostAccounting, costs::Cost},
    dispatch::RholangAndScalaDispatcher,
    env::Env,
    errors::InterpreterError,
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
async fn eval_of_bundle_should_throw_an_error_if_names_are_used_against_their_polarity_1() {
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

#[tokio::test]
async fn eval_of_bundle_should_throw_an_error_if_names_are_used_against_their_polarity_2() {
    let (space, reducer) = create_test_space().await;

    /* @bundle- { x } !(7) -> x!(7)
     */
    let x = new_gstring_par(String::from("channel"), Vec::new(), false);
    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_bundle_par(x, false, true)),
        data: vec![new_gint_par(7, Vec::new(), false)],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    let result = reducer
        .eval(send, &env, rand())
        .await
        .map(|_| space.lock().unwrap().to_map());
    assert!(result.is_err());
    if let Err(e) = result {
        assert_eq!(
            e,
            InterpreterError::ReduceError(String::from("Trying to send on non-writeable channel."))
        );
    }
}

#[tokio::test]
async fn eval_of_send_should_place_something_in_the_tuplespace() {
    let (space, reducer) = create_test_space().await;

    let channel = new_gstring_par(String::from("channel"), Vec::new(), false);
    let split_rand = rand().split_byte(0);
    let send = Par::default().with_sends(vec![Send {
        chan: Some(channel.clone()),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    let result = reducer
        .eval(send, &env, split_rand.clone())
        .await
        .map(|_| space.lock().unwrap().to_map());

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
    assert_eq!(result.unwrap(), expected_result);
}

#[tokio::test]
async fn eval_of_send_should_verify_that_bundle_is_writeable_before_sending_on_bundle() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);

    /* @bundle+ { x } !(7) -> x!(7)
     */
    let channel = new_gstring_par(String::from("channel"), Vec::new(), false);
    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_bundle_par(channel.clone(), true, false)),
        data: vec![new_gint_par(7, Vec::new(), false)],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    let result = reducer
        .eval(send, &env, split_rand.clone())
        .await
        .map(|_| space.lock().unwrap().to_map());

    assert!(result.is_ok());
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        channel,
        (vec![new_gint_par(7, Vec::new(), false)], split_rand),
    );
    let expected_result = map_data(expected_elements);
    assert_eq!(result.unwrap(), expected_result);
}

#[tokio::test]
async fn eval_of_single_channel_receive_should_place_something_in_the_tuplespace() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let channel = new_gstring_par(String::from("channel"), Vec::new(), false);
    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![
                new_freevar_par(0, Vec::new()),
                new_freevar_par(1, Vec::new()),
                new_freevar_par(2, Vec::new()),
            ],
            source: Some(channel.clone()),
            remainder: None,
            free_count: 0,
        }],
        body: Some(Par::default()),
        persistent: false,
        peek: false,
        bind_count: 3,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    let result = reducer
        .eval(receive, &env, split_rand.clone())
        .await
        .map(|_| space.lock().unwrap().to_map());

    assert!(result.is_ok());
    let bind_pattern = BindPattern {
        patterns: vec![
            new_freevar_par(0, Vec::new()),
            new_freevar_par(1, Vec::new()),
            new_freevar_par(2, Vec::new()),
        ],
        remainder: None,
        free_count: 0,
    };

    assert!(check_continuation(
        result.unwrap(),
        vec![channel],
        vec![bind_pattern],
        ParWithRandom {
            body: Some(Par::default()),
            random_state: split_rand.to_vec()
        }
    ))
}

#[tokio::test]
async fn eval_of_single_channel_receive_should_verify_that_bundle_is_readable_if_receiving_on_bundle(
) {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(1);

    /* for (@Nil <- @bundle- { y } ) { }  -> for (n <- y) { }
     */
    let y = new_gstring_par(String::from("y"), Vec::new(), false);
    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![Par::default()],
            source: Some(new_bundle_par(y.clone(), false, true)),
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
        .eval(receive, &env, split_rand.clone())
        .await
        .map(|_| space.lock().unwrap().to_map());

    assert!(result.is_ok());
    assert!(check_continuation(
        result.unwrap(),
        vec![y],
        vec![BindPattern {
            patterns: vec![Par::default()],
            remainder: None,
            free_count: 0
        }],
        ParWithRandom {
            body: Some(Par::default()),
            random_state: split_rand.to_vec()
        }
    ))
}

#[tokio::test]
async fn eval_of_send_pipe_receive_should_meet_in_the_tuple_space_and_proceed() {
    let (space, reducer) = create_test_space().await;

    let split_rand0 = rand().split_byte(0);
    let split_rand1 = rand().split_byte(1);
    let merge_rand = Blake2b512Random::merge(vec![split_rand1.clone(), split_rand0.clone()]);
    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par(String::from("channel"), Vec::new(), false)),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![
                new_freevar_par(0, Vec::new()),
                new_freevar_par(1, Vec::new()),
                new_freevar_par(2, Vec::new()),
            ],
            source: Some(new_gstring_par("channel".to_string(), Vec::new(), false)),
            remainder: None,
            free_count: 3,
        }],
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
            data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        persistent: false,
        peek: false,
        bind_count: 3,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    assert!(reducer
        .eval(send.clone(), &env, split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .eval(receive.clone(), &env, split_rand1.clone())
        .await
        .is_ok());

    let send_result = space.lock().unwrap().to_map();
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        new_gstring_par(String::from("result"), Vec::new(), false),
        (
            vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            merge_rand,
        ),
    );
    assert_eq!(send_result, map_data(expected_elements.clone()));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .eval(receive, &env, split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer.eval(send, &env, split_rand0.clone()).await.is_ok());

    let receive_result = space.lock().unwrap().to_map();
    assert_eq!(receive_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_send_pipe_receive_with_peek_should_meet_in_the_tuple_space_and_proceed() {
    let (space, reducer) = create_test_space().await;

    let channel = new_gstring_par("channel".to_string(), Vec::new(), false);
    let result_channel = new_gstring_par("result".to_string(), Vec::new(), false);

    let split_rand0 = rand().split_byte(0);
    let split_rand1 = rand().split_byte(1);
    let merge_rand = Blake2b512Random::merge(vec![split_rand1.clone(), split_rand0.clone()]);

    let send = Par::default().with_sends(vec![Send {
        chan: Some(channel.clone()),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![
                new_freevar_par(0, Vec::new()),
                new_freevar_par(1, Vec::new()),
                new_freevar_par(2, Vec::new()),
            ],
            source: Some(channel.clone()),
            remainder: None,
            free_count: 3,
        }],
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(result_channel.clone()),
            data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        persistent: false,
        peek: true,
        bind_count: 3,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        channel,
        (
            vec![
                new_gint_par(7, Vec::new(), false),
                new_gint_par(8, Vec::new(), false),
                new_gint_par(9, Vec::new(), false),
            ],
            split_rand0.clone(),
        ),
    );
    expected_elements.insert(
        result_channel,
        (
            vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            merge_rand,
        ),
    );

    let env: Env<Par> = Env::new();
    assert!(reducer
        .eval(send.clone(), &env, split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .eval(receive.clone(), &env, split_rand1.clone())
        .await
        .is_ok());

    let send_result = space.lock().unwrap().to_map();
    assert_eq!(send_result, map_data(expected_elements.clone()));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .eval(receive, &env, split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer.eval(send, &env, split_rand0.clone()).await.is_ok());

    let receive_result = space.lock().unwrap().to_map();
    assert_eq!(receive_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_send_pipe_receive_when_whole_list_is_bound_to_list_remainder_should_meet_in_the_tuple_space_and_proceed(
) {
    let (space, reducer) = create_test_space().await;

    // for(@[...a] <- @"channel") { … } | @"channel"!([7,8,9])
    let channel = new_gstring_par("channel".to_string(), Vec::new(), false);
    let result_channel = new_gstring_par("result".to_string(), Vec::new(), false);
    let split_rand0 = rand().split_byte(0);
    let split_rand1 = rand().split_byte(1);
    let merge_rand = Blake2b512Random::merge(vec![split_rand1.clone(), split_rand0.clone()]);

    let send = Par::default().with_sends(vec![Send {
        chan: Some(channel.clone()),
        data: vec![new_elist_par(
            vec![
                new_gint_par(7, Vec::new(), false),
                new_gint_par(8, Vec::new(), false),
                new_gint_par(9, Vec::new(), false),
            ],
            Vec::new(),
            false,
            None,
            Vec::new(),
            false,
        )],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![new_elist_par(
                vec![new_freevar_par(0, Vec::new())],
                Vec::new(),
                true,
                Some(new_freevar_var(0)),
                Vec::new(),
                false,
            )],
            source: Some(channel.clone()),
            remainder: None,
            free_count: 1,
        }],
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(result_channel.clone()),
            data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        persistent: false,
        peek: false,
        bind_count: 1,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    assert!(reducer
        .eval(send.clone(), &env, split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .eval(receive.clone(), &env, split_rand1.clone())
        .await
        .is_ok());

    let send_result = space.lock().unwrap().to_map();

    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        result_channel,
        (
            vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            merge_rand,
        ),
    );
    assert_eq!(send_result, map_data(expected_elements.clone()));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .eval(receive, &env, split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer.eval(send, &env, split_rand0.clone()).await.is_ok());

    let receive_result = space.lock().unwrap().to_map();
    assert_eq!(receive_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_send_on_seven_plus_eight_pipe_receive_on_fifteen_should_meet_in_the_tuple_space_and_proceed(
) {
    let (space, reducer) = create_test_space().await;

    let split_rand0 = rand().split_byte(0);
    let split_rand1 = rand().split_byte(1);
    let merge_rand = Blake2b512Random::merge(vec![split_rand1.clone(), split_rand0.clone()]);

    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_eplus_par(7, 8, Vec::new(), false)),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![
                new_freevar_par(0, Vec::new()),
                new_freevar_par(1, Vec::new()),
                new_freevar_par(2, Vec::new()),
            ],
            source: Some(new_gint_par(15, Vec::new(), false)),
            remainder: None,
            free_count: 3,
        }],
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
            data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        persistent: false,
        peek: false,
        bind_count: 3,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    assert!(reducer
        .eval(send.clone(), &env, split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .eval(receive.clone(), &env, split_rand1.clone())
        .await
        .is_ok());

    let send_result = space.lock().unwrap().to_map();

    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            merge_rand,
        ),
    );
    assert_eq!(send_result, map_data(expected_elements.clone()));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .eval(receive, &env, split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer.eval(send, &env, split_rand0.clone()).await.is_ok());

    let receive_result = space.lock().unwrap().to_map();
    assert_eq!(receive_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_send_of_receive_pipe_receive_should_meet_in_the_tuple_space_and_proceed() {
    let (space, reducer) = create_test_space().await;

    let base_rand = rand().split_byte(2);
    let split_rand0 = base_rand.split_byte(0);
    let split_rand1 = base_rand.split_byte(1);
    let merge_rand = Blake2b512Random::merge(vec![split_rand1.clone(), split_rand0.clone()]);

    let simple_receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![new_gint_par(2, Vec::new(), false)],
            source: Some(new_gint_par(2, Vec::new(), false)),
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

    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_gint_par(1, Vec::new(), false)),
        data: vec![simple_receive.clone()],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![new_freevar_par(0, Vec::new())],
            source: Some(new_gint_par(1, Vec::new(), false)),
            remainder: None,
            free_count: 1,
        }],
        body: Some(new_boundvar_par(0, Vec::new(), false)),
        persistent: false,
        peek: false,
        bind_count: 1,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    assert!(reducer
        .eval(send.clone(), &env, split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .eval(receive.clone(), &env, split_rand1.clone())
        .await
        .is_ok());

    let send_result = space.lock().unwrap().to_map();
    let channels = vec![new_gint_par(2, Vec::new(), false)];
    // Because they are evaluated separately, nothing is split.
    assert!(check_continuation(
        send_result,
        channels.clone(),
        vec![BindPattern {
            patterns: vec![new_gint_par(2, Vec::new(), false)],
            remainder: None,
            free_count: 0,
        }],
        ParWithRandom {
            body: Some(Par::default()),
            random_state: merge_rand.to_vec(),
        },
    ));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .eval(receive, &env, split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer.eval(send, &env, split_rand0.clone()).await.is_ok());

    let receive_result = space.lock().unwrap().to_map();
    assert!(check_continuation(
        receive_result,
        channels.clone(),
        vec![BindPattern {
            patterns: vec![new_gint_par(2, Vec::new(), false)],
            remainder: None,
            free_count: 0,
        }],
        ParWithRandom {
            body: Some(Par::default()),
            random_state: merge_rand.to_vec(),
        },
    ));

    let (space, reducer) = create_test_space().await;
    let mut par_param = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![new_freevar_par(0, Vec::new())],
            source: Some(new_gint_par(1, Vec::new(), false)),
            remainder: None,
            free_count: 1,
        }],
        body: Some(new_boundvar_par(0, Vec::new(), false)),
        persistent: false,
        peek: false,
        bind_count: 1,
        locally_free: Vec::new(),
        connective_used: false,
    }]);
    par_param = par_param.with_sends(vec![Send {
        chan: Some(new_gint_par(1, Vec::new(), false)),
        data: vec![simple_receive],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);
    assert!(reducer.eval(par_param, &env, base_rand).await.is_ok());

    let both_result = space.lock().unwrap().to_map();
    assert!(check_continuation(
        both_result,
        channels,
        vec![BindPattern {
            patterns: vec![new_gint_par(2, Vec::new(), false)],
            remainder: None,
            free_count: 0,
        }],
        ParWithRandom {
            body: Some(Par::default()),
            random_state: merge_rand.to_vec(),
        },
    ));
}

#[tokio::test]
async fn simple_match_should_capture_and_add_to_the_environment() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let mut pattern = Par::default().with_sends(vec![Send {
        chan: Some(new_freevar_par(0, Vec::new())),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_freevar_par(1, Vec::new()),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: true,
    }]);
    pattern.connective_used = true;

    let send_target = Par::default().with_sends(vec![Send {
        chan: Some(new_boundvar_par(1, Vec::new(), false)),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_boundvar_par(0, Vec::new(), false),
        ],
        persistent: false,
        locally_free: vec![0b00000011],
        connective_used: false,
    }]);

    let match_term = Par::default().with_matches(vec![Match {
        target: Some(send_target),
        cases: vec![MatchCase {
            pattern: Some(pattern),
            source: Some(Par::default().with_sends(vec![Send {
                chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
                data: vec![
                    new_boundvar_par(1, Vec::new(), false),
                    new_boundvar_par(0, Vec::new(), false),
                ],
                persistent: false,
                locally_free: vec![0b00000011],
                connective_used: false,
            }])),
            free_count: 2,
        }],
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let mut env: Env<Par> = Env::new();
    env = env.put(GPrivateBuilder::new_par("one".to_string()));
    env = env.put(GPrivateBuilder::new_par("zero".to_string()));
    assert!(reducer
        .eval(match_term.clone(), &env, split_rand.clone())
        .await
        .is_ok());

    let match_result = space.lock().unwrap().to_map();
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![
                GPrivateBuilder::new_par("one".to_string()),
                GPrivateBuilder::new_par("zero".to_string()),
            ],
            split_rand,
        ),
    );
    assert_eq!(match_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_send_pipe_send_pipe_receive_join_should_meet_in_tuplespace_and_proceed() {
    let (space, reducer) = create_test_space().await;

    let split_rand0 = rand().split_byte(0);
    let split_rand1 = rand().split_byte(1);
    let split_rand2 = rand().split_byte(2);
    let merge_rand = Blake2b512Random::merge(vec![
        split_rand2.clone(),
        split_rand0.clone(),
        split_rand1.clone(),
    ]);

    let send1 = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par("channel1".to_string(), Vec::new(), false)),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let send2 = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par("channel2".to_string(), Vec::new(), false)),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![
            ReceiveBind {
                patterns: vec![
                    new_freevar_par(0, Vec::new()),
                    new_freevar_par(1, Vec::new()),
                    new_freevar_par(2, Vec::new()),
                ],
                source: Some(new_gstring_par("channel1".to_string(), Vec::new(), false)),
                remainder: None,
                free_count: 3,
            },
            ReceiveBind {
                patterns: vec![
                    new_freevar_par(0, Vec::new()),
                    new_freevar_par(1, Vec::new()),
                    new_freevar_par(2, Vec::new()),
                ],
                source: Some(new_gstring_par("channel2".to_string(), Vec::new(), false)),
                remainder: None,
                free_count: 3,
            },
        ],
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
            data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        persistent: false,
        peek: false,
        bind_count: 3,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    assert!(reducer
        .inj(send1.clone(), split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .inj(send2.clone(), split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer
        .inj(receive.clone(), split_rand2.clone())
        .await
        .is_ok());

    let send_result = space.lock().unwrap().to_map();
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            merge_rand,
        ),
    );
    assert_eq!(send_result, map_data(expected_elements.clone()));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .inj(receive.clone(), split_rand2.clone())
        .await
        .is_ok());
    assert!(reducer
        .inj(send1.clone(), split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .inj(send2.clone(), split_rand1.clone())
        .await
        .is_ok());

    let receive_result = space.lock().unwrap().to_map();
    assert_eq!(receive_result, map_data(expected_elements.clone()));

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .inj(send1.clone(), split_rand0.clone())
        .await
        .is_ok());
    assert!(reducer
        .inj(receive.clone(), split_rand2.clone())
        .await
        .is_ok());
    assert!(reducer
        .inj(send2.clone(), split_rand1.clone())
        .await
        .is_ok());

    let inter_leaved_result = space.lock().unwrap().to_map();
    assert_eq!(inter_leaved_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_send_with_remainder_receive_should_capture_the_remainder() {
    let (space, reducer) = create_test_space().await;

    let split_rand0 = rand().split_byte(0);
    let split_rand1 = rand().split_byte(1);
    let merge_rand = Blake2b512Random::merge(vec![split_rand1.clone(), split_rand0.clone()]);

    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par("channel".to_string(), Vec::new(), false)),
        data: vec![
            new_gint_par(7, Vec::new(), false),
            new_gint_par(8, Vec::new(), false),
            new_gint_par(9, Vec::new(), false),
        ],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let receive = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![],
            source: Some(new_gstring_par("channel".to_string(), Vec::new(), false)),
            remainder: Some(new_freevar_var(0)),
            free_count: 1,
        }],
        body: Some(Par::default().with_sends(vec![Send {
            chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
            data: vec![new_boundvar_par(0, Vec::new(), false)],
            persistent: false,
            locally_free: Vec::new(),
            connective_used: false,
        }])),
        persistent: false,
        peek: false,
        bind_count: 0,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env = Env::new();
    assert!(reducer
        .eval(receive.clone(), &env, split_rand1.clone())
        .await
        .is_ok());
    assert!(reducer
        .eval(send.clone(), &env, split_rand0.clone())
        .await
        .is_ok());

    let result = space.lock().unwrap().to_map();
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![new_elist_par(
                vec![
                    new_gint_par(7, Vec::new(), false),
                    new_gint_par(8, Vec::new(), false),
                    new_gint_par(9, Vec::new(), false),
                ],
                Vec::new(),
                false,
                None,
                Vec::new(),
                false,
            )],
            merge_rand,
        ),
    );
    assert_eq!(result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_nth_method_should_pick_out_the_nth_item_from_a_list() {
    let (_, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let nth_call = Expr {
        expr_instance: Some(ExprInstance::EMethodBody(EMethod {
            method_name: "nth".to_string(),
            target: Some(new_elist_par(
                vec![
                    new_gint_par(7, Vec::new(), false),
                    new_gint_par(8, Vec::new(), false),
                    new_gint_par(9, Vec::new(), false),
                ],
                Vec::new(),
                false,
                None,
                Vec::new(),
                false,
            )),
            arguments: vec![new_gint_par(2, Vec::new(), false)],
            locally_free: Vec::new(),
            connective_used: false,
        })),
    };

    let env = Env::new();
    let direct_result = reducer.eval_expr_to_par(&nth_call, &env).unwrap();
    assert_eq!(new_gint_par(9, Vec::new(), false), direct_result);

    let nth_call_eval_to_send = Par::default().with_exprs(vec![Expr {
        expr_instance: Some(ExprInstance::EMethodBody(EMethod {
            method_name: "nth".to_string(),
            target: Some(new_elist_par(
                vec![
                    new_gint_par(7, Vec::new(), false),
                    Par::default().with_sends(vec![Send {
                        chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
                        data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
                        persistent: false,
                        locally_free: Vec::new(),
                        connective_used: false,
                    }]),
                    new_gint_par(9, Vec::new(), false),
                    new_gint_par(10, Vec::new(), false),
                ],
                Vec::new(),
                false,
                None,
                Vec::new(),
                false,
            )),
            arguments: vec![new_gint_par(1, Vec::new(), false)],
            locally_free: Vec::new(),
            connective_used: false,
        })),
    }]);

    let (space, reducer) = create_test_space().await;
    assert!(reducer
        .eval(nth_call_eval_to_send, &env, split_rand.clone())
        .await
        .is_ok());

    let indirect_result = space.lock().unwrap().to_map();
    let mut expected_elements = HashMap::new();
    expected_elements.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
            split_rand,
        ),
    );
    assert_eq!(indirect_result, map_data(expected_elements));
}

#[tokio::test]
async fn eval_of_nth_method_should_pick_out_the_nth_item_from_a_byte_array() {
    let (_, reducer) = create_test_space().await;

    let nth_call = Expr {
        expr_instance: Some(ExprInstance::EMethodBody(EMethod {
            method_name: "nth".to_string(),
            target: Some(Par::default().with_exprs(vec![Expr {
                expr_instance: Some(ExprInstance::GByteArray(vec![1, 2, 255])),
            }])),
            arguments: vec![new_gint_par(2, Vec::new(), false)],
            locally_free: Vec::new(),
            connective_used: false,
        })),
    };

    let env = Env::new();
    let direct_result = reducer.eval_expr_to_par(&nth_call, &env).unwrap();
    assert_eq!(new_gint_par(255, Vec::new(), false), direct_result);
}

#[tokio::test]
async fn eval_of_length_method_should_get_length_of_byte_array() {
    let (_, reducer) = create_test_space().await;

    let nth_call = Expr {
        expr_instance: Some(ExprInstance::EMethodBody(EMethod {
            method_name: "length".to_string(),
            target: Some(Par::default().with_exprs(vec![Expr {
                expr_instance: Some(ExprInstance::GByteArray(vec![1, 2, 255])),
            }])),
            arguments: vec![],
            locally_free: Vec::new(),
            connective_used: false,
        })),
    };

    let env = Env::new();
    let direct_result = reducer.eval_expr_to_par(&nth_call, &env).unwrap();
    assert_eq!(new_gint_par(3, Vec::new(), false), direct_result);
}

#[tokio::test]
async fn eval_of_new_should_use_deterministic_names_and_provide_urn_based_resources() {
    let (space, _) = create_test_space().await;

    let split_rand = rand().split_byte(42);
    let mut result_rand = rand().split_byte(42);
    let chosen_name = result_rand.next();
    let result0_rand = result_rand.split_byte(0);
    let result1_rand = result_rand.split_byte(1);

    let new = Par::default().with_news(vec![New {
        bind_count: 2,
        p: Some(Par::default().with_sends(vec![
            Send {
                chan: Some(new_gstring_par("result0".to_string(), Vec::new(), false)),
                data: vec![new_boundvar_par(0, Vec::new(), false)],
                persistent: false,
                locally_free: vec![0],
                connective_used: false,
            },
            Send {
                chan: Some(new_gstring_par("result1".to_string(), Vec::new(), false)),
                data: vec![new_boundvar_par(1, Vec::new(), false)],
                persistent: false,
                locally_free: vec![1],
                connective_used: false,
            },
        ])),
        uri: vec!["rho:test:foo".to_string()],
        injections: BTreeMap::new(),
        locally_free: vec![0b00000011],
    }]);

    let cost = CostAccounting::empty_cost();
    let mut urn_map = HashMap::new();
    urn_map.insert(
        "rho:test:foo".to_string(),
        Par::default().with_unforgeables(vec![GUnforgeable {
            unf_instance: Some(UnfInstance::GPrivateBody(GPrivate { id: vec![42] })),
        }]),
    );
    let reducer = RholangAndScalaDispatcher::create(
        space.clone(),
        HashMap::new(),
        urn_map,
        Arc::new(RwLock::new(HashSet::new())),
        Par::default(),
        cost.clone(),
    )
    .1;
    cost.set(Cost::unsafe_max());
    let env = Env::new();
    assert!(reducer.eval(new, &env, split_rand).await.is_ok());
    let result = space.lock().unwrap().to_map();

    let channel0 = new_gstring_par("result0".to_string(), Vec::new(), false);
    let channel1 = new_gstring_par("result1".to_string(), Vec::new(), false);

    let mut expected_result = HashMap::new();
    expected_result.insert(
        vec![channel0.clone()],
        Row {
            data: vec![Datum::create(
                channel0,
                ListParWithRandom {
                    pars: vec![Par::default().with_unforgeables(vec![GUnforgeable {
                        unf_instance: Some(UnfInstance::GPrivateBody(GPrivate { id: vec![42] })),
                    }])],
                    random_state: result0_rand.to_vec(),
                },
                false,
            )],
            wks: vec![],
        },
    );

    expected_result.insert(
        vec![channel1.clone()],
        Row {
            data: vec![Datum::create(
                channel1,
                ListParWithRandom {
                    pars: vec![Par::default().with_unforgeables(vec![GUnforgeable {
                        unf_instance: Some(UnfInstance::GPrivateBody(GPrivate { id: chosen_name })),
                    }])],
                    random_state: result1_rand.to_vec(),
                },
                false,
            )],
            wks: vec![],
        },
    );

    assert_eq!(result, expected_result);
}

#[tokio::test]
async fn eval_of_nth_method_in_send_position_should_change_what_is_sent() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let nth_call_eval_to_send = Par::default().with_exprs(vec![Expr {
        expr_instance: Some(ExprInstance::EMethodBody(EMethod {
            method_name: "nth".to_string(),
            target: Some(new_elist_par(
                vec![
                    new_gint_par(7, Vec::new(), false),
                    Par::default().with_sends(vec![Send {
                        chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
                        data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
                        persistent: false,
                        locally_free: Vec::new(),
                        connective_used: false,
                    }]),
                    new_gint_par(9, Vec::new(), false),
                    new_gint_par(10, Vec::new(), false),
                ],
                Vec::new(),
                false,
                None,
                Vec::new(),
                false,
            )),
            arguments: vec![new_gint_par(1, Vec::new(), false)],
            locally_free: Vec::new(),
            connective_used: false,
        })),
    }]);

    let send = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
        data: vec![nth_call_eval_to_send],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env = Env::new();
    assert!(reducer.eval(send, &env, split_rand.clone()).await.is_ok());
    let result = space.lock().unwrap().to_map();

    let channel = new_gstring_par("result".to_string(), Vec::new(), false);
    let mut expected_result = HashMap::new();
    expected_result.insert(
        channel.clone(),
        (
            vec![Par::default().with_sends(vec![Send {
                chan: Some(channel),
                data: vec![new_gstring_par("Success".to_string(), Vec::new(), false)],
                persistent: false,
                locally_free: Vec::new(),
                connective_used: false,
            }])],
            split_rand,
        ),
    );
    assert_eq!(result, map_data(expected_result));
}

#[tokio::test]
async fn eval_of_a_method_should_substitute_target_before_evaluating() {
    let (_, reducer) = create_test_space().await;

    let hex_to_bytes_call = Expr {
        expr_instance: Some(ExprInstance::EMethodBody(EMethod {
            method_name: "hexToBytes".to_string(),
            target: Some(new_boundvar_par(0, Vec::new(), false)),
            arguments: vec![],
            locally_free: Vec::new(),
            connective_used: false,
        })),
    };

    let mut env = Env::new();
    env = env.put(new_gstring_par("deadbeef".to_string(), Vec::new(), false));
    let direct_result = reducer.eval_expr_to_par(&hex_to_bytes_call, &env).unwrap();
    assert_eq!(
        direct_result,
        Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::GByteArray(StringOps::unsafe_decode_hex(
                "deadbeef".to_string()
            )))
        }])
    );
}

#[tokio::test]
async fn eval_of_to_byte_array_method_on_any_process_should_return_that_process_serialized() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let proc = Par::default().with_receives(vec![Receive {
        binds: vec![ReceiveBind {
            patterns: vec![new_freevar_par(0, Vec::new())],
            source: Some(new_gstring_par("channel".to_string(), Vec::new(), false)),
            remainder: None,
            free_count: 0,
        }],
        body: Some(Par::default()),
        persistent: false,
        peek: false,
        bind_count: 1,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let serialized_process = bincode::serialize(&proc).unwrap();
    let to_byte_array_call = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
        data: vec![Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::EMethodBody(EMethod {
                method_name: "toByteArray".to_string(),
                target: Some(proc),
                arguments: Vec::new(),
                locally_free: Vec::new(),
                connective_used: false,
            })),
        }])],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let env: Env<Par> = Env::new();
    assert!(reducer
        .eval(to_byte_array_call, &env, split_rand.clone())
        .await
        .is_ok());
    let result = space.lock().unwrap().to_map();
    let mut expected_result = HashMap::new();
    expected_result.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![Par::default().with_exprs(vec![Expr {
                expr_instance: Some(ExprInstance::GByteArray(serialized_process)),
            }])],
            split_rand,
        ),
    );
    assert_eq!(result, map_data(expected_result));
}

#[tokio::test]
async fn eval_of_to_byte_array_method_on_any_process_should_substitute_before_serialization() {
    let (space, reducer) = create_test_space().await;

    let split_rand = rand().split_byte(0);
    let unsub_proc = Par::default().with_news(vec![New {
        bind_count: 1,
        p: Some(new_boundvar_par(1, Vec::new(), false)),
        uri: Vec::new(),
        injections: BTreeMap::new(),
        locally_free: vec![0],
    }]);
    let sub_proc = Par::default().with_news(vec![New {
        bind_count: 1,
        p: Some(GPrivateBuilder::new_par("zero".to_string())),
        uri: Vec::new(),
        injections: BTreeMap::new(),
        locally_free: vec![],
    }]);

    let serialized_process = bincode::serialize(&sub_proc).unwrap();
    let to_byte_array_call = Par::default().with_sends(vec![Send {
        chan: Some(new_gstring_par("result".to_string(), Vec::new(), false)),
        data: vec![Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::EMethodBody(EMethod {
                method_name: "toByteArray".to_string(),
                target: Some(unsub_proc),
                arguments: Vec::new(),
                locally_free: vec![0],
                connective_used: false,
            })),
        }])],
        persistent: false,
        locally_free: Vec::new(),
        connective_used: false,
    }]);

    let mut env: Env<Par> = Env::new();
    env = env.put(GPrivateBuilder::new_par("one".to_string()));
    env = env.put(GPrivateBuilder::new_par("zero".to_string()));

    println!("{:?}", GPrivateBuilder::new_par("one".to_string()));
    println!("{:?}", GPrivateBuilder::new_par("zero".to_string()));

    assert!(reducer
        .eval(to_byte_array_call, &env, split_rand.clone())
        .await
        .is_ok());
    let result = space.lock().unwrap().to_map();
    let mut expected_result = HashMap::new();
    expected_result.insert(
        new_gstring_par("result".to_string(), Vec::new(), false),
        (
            vec![Par::default().with_exprs(vec![Expr {
                expr_instance: Some(ExprInstance::GByteArray(serialized_process)),
            }])],
            split_rand,
        ),
    );
    assert_eq!(result, map_data(expected_result));
}
