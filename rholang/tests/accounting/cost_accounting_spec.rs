//See rholang/src/test/scala/coop/rchain/rholang/interpreter/accounting/CostAccountingSpec.scala

use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use rholang::rust::interpreter::rho_runtime::{create_replay_rho_runtime, RhoRuntimeImpl};
use rholang::rust::interpreter::system_processes::Definition;
use rholang::rust::interpreter::test_utils::resources::create_runtimes;
use rholang::rust::interpreter::{
    accounting::costs::Cost,
    interpreter::EvaluateResult,
    matcher::r#match::Matcher,
    rho_runtime::{create_rho_runtime, RhoRuntime},
};
use rspace_plus_plus::rspace::history::history_repository::HistoryRepository;
use rspace_plus_plus::rspace::rspace::RSpaceStore;
use rspace_plus_plus::rspace::{
    rspace::RSpace,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
};

use std::collections::HashMap;
use std::option::Option;
use std::sync::{Arc, Mutex};
async fn evaluate_with_cost_log(initial_phlo: i64, contract: String) -> EvaluateResult {
    let mut kvm = InMemoryStoreManager::new();

    //TODO costLog implementation
    /*
    We need to come up with an implementation of `costLog` in Rust (check how it works in Scala), because `costLog` is a kind of global tracker and in tests we have checks like this:
    costLog.map(_.value).toList.sum shouldEqual expectedTotalCost

    But first we need to finish everything else and port the tests without `costLog` assertions.

    1. Port all the code
    2. Ensure tests pass without `costLog` assertions
    3. Implement `costLog` (global tracker) in Rust and make sure all tests work as they do in Scala
    */
    let store = kvm.r_space_stores().await.unwrap();

    let (runtime, _, _) =
        create_runtimes_with_cost_log(store, Some(false), Some(&mut Vec::new())).await;

    let eval_result = runtime
        .try_lock()
        .unwrap()
        .evaluate_with_phlo(
            &contract,
            Cost::create(initial_phlo, "cost_accounting_spec setup".to_string()),
        )
        .await;

    assert!(eval_result.is_ok());
    eval_result.unwrap()
}

async fn create_runtimes_with_cost_log(
    stores: RSpaceStore,
    init_registry: Option<bool>,
    additional_system_processes: Option<&mut Vec<Definition>>,
) -> (
    Arc<Mutex<RhoRuntimeImpl>>,
    Arc<Mutex<RhoRuntimeImpl>>,
    Arc<Box<dyn HistoryRepository<Par, BindPattern, ListParWithRandom, TaggedContinuation>>>,
) {
    let init_registry = init_registry.unwrap_or(false);

    let mut empty_vec = Vec::new();
    let additional_system_processes = additional_system_processes.unwrap_or(&mut empty_vec);

    let hrstores =
        RSpace::<Par, BindPattern, ListParWithRandom, TaggedContinuation>::create_with_replay(
            stores,
            Arc::new(Box::new(Matcher)),
        )
        .unwrap();

    let (space, replay) = hrstores;

    let history_repository = space.history_repository.clone();

    let rho_runtime = create_rho_runtime(
        space.clone(),
        Par::default(),
        init_registry,
        additional_system_processes,
    )
    .await;

    let replay_rho_runtime = create_replay_rho_runtime(
        replay,
        Par::default(),
        init_registry,
        additional_system_processes,
    )
    .await;

    (rho_runtime, replay_rho_runtime, history_repository)
}

async fn evaluate_and_replay(initial_phlo: Cost, term: String) -> (EvaluateResult, EvaluateResult) {
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let (runtime, replay_runtime, _): (
        Arc<Mutex<RhoRuntimeImpl>>,
        Arc<Mutex<RhoRuntimeImpl>>,
        Arc<Box<dyn HistoryRepository<Par, BindPattern, ListParWithRandom, TaggedContinuation>>>,
    ) = create_runtimes(store, false, &mut Vec::new()).await;

    let rand = Blake2b512Random::create_from_bytes(&[]);

    let play_result = {
        let mut runtime_lock = runtime.lock().unwrap();
        runtime_lock
            .evaluate(&term, initial_phlo.clone(), HashMap::new(), rand.clone())
            .await
            .expect("Evaluation failed")
    };

    let replay_result = {
        let checkpoint = runtime.lock().unwrap().create_checkpoint();
        let root = checkpoint.root;
        let log = checkpoint.log;

        let mut replay_lock = replay_runtime.lock().unwrap();
        replay_lock.reset(root);
        replay_lock.rig(log).expect("Rig failed");

        let result = replay_lock
            .evaluate(&term, initial_phlo, HashMap::new(), rand)
            .await
            .expect("Replay evaluation failed");

        replay_lock
            .check_replay_data()
            .expect("Replay data check failed");

        result
    };

    (play_result, replay_result)
}

// Uses Godel numbering and a https://en.wikipedia.org/wiki/Mixed_radix
// to encode certain terms as numbers in the range [0, 0x144000000).
// Every number gets decoded into a unique term, but some terms can
// be encoded by more than one number.
fn from_long(index: i64) -> String {
    let mut remainder = index;
    let num_pars = (remainder % 4) + 1;
    remainder /= 4;

    let mut result = Vec::new();
    let mut nonlinear_send = false;
    let mut nonlinear_recv = false;

    for _ in 0..num_pars {
        let dir = remainder % 2;
        remainder /= 2;

        if dir == 0 {
            //send
            let bang = if remainder % 2 == 0 { "!" } else { "!!" };
            remainder /= 2;

            if bang == "!" || !nonlinear_recv {
                let ch = remainder % 4;
                remainder /= 4;
                result.push(format!("@{}{}(0)", ch, bang));
                nonlinear_send |= (bang == "!!");
            }
        } else {
            //receive
            let arrow = match remainder % 3 {
                0 => "<-",
                1 => "<=",
                2 => "<<-",
                _ => unreachable!(),
            };
            remainder /= 3;

            if arrow != "<=" || !nonlinear_send {
                let num_joins = (remainder % 2) + 1;
                remainder /= 2;

                let mut joins = Vec::new();
                for _ in 1..=num_joins {
                    let ch = remainder % 4;
                    remainder /= 4;
                    joins.push(format!("_ {} @{}", arrow, ch));
                }

                let join_str = joins.join(" & ");
                result.push(format!("for ({}) {{ 0 }}", join_str));
                nonlinear_recv |= (arrow == "<=");
            }
        }
    }

    result.join(" | ")
}

fn contracts() -> Vec<(String, i64)> {
    vec![
      (String::from("@0!(2)"), 97i64),
      (String::from("@0!(2) | @1!(1)"), 197i64),
      (String::from("for(x <- @0){ Nil }"), 128i64),
      (String::from("for(x <- @0){ Nil } | @0!(2)"), 329i64),
      (String::from("@0!!(0) | for (_ <- @0) { 0 }"), 342i64),
      (String::from("@0!!(0) | for (x <- @0) { 0 }"), 342i64),
      (String::from("@0!!(0) | for (@0 <- @0) { 0 }"), 336i64),
      (String::from("@0!!(0) | @0!!(0) | for (_ <- @0) { 0 }"), 443i64),
      (String::from("@0!!(0) | @1!!(1) | for (_ <- @0 & _ <- @1) { 0 }"), 596i64),
      (String::from("@0!(0) | for (_ <- @0) { 0 }"), 333i64),
      (String::from("@0!(0) | for (x <- @0) { 0 }"), 333i64),
      (String::from("@0!(0) | for (@0 <- @0) { 0 }"), 327i64),
      (String::from("@0!(0) | for (_ <= @0) { 0 }"), 354i64),
      (String::from("@0!(0) | for (x <= @0) { 0 }"), 356i64),
      (String::from("@0!(0) | for (@0 <= @0) { 0 }"), 341i64),
      (String::from("@0!(0) | @0!(0) | for (_ <= @0) { 0 }"), 574i64),
      (String::from("@0!(0) | for (@0 <- @0) { 0 } | @0!(0) | for (_ <- @0) { 0 }"), 663i64),
      (String::from("@0!(0) | for (@0 <- @0) { 0 } | @0!(0) | for (@1 <- @0) { 0 }"), 551i64),
      (String::from("@0!(0) | for (_ <<- @0) { 0 }"), 406i64),
      (String::from("@0!!(0) | for (_ <<- @0) { 0 }"), 343i64),
      (String::from("@0!!(0) | @0!!(0) | for (_ <<- @0) { 0 }"), 444i64),
//       (String::from("new loop in {\n         contract loop(@n) = {\n           match n {\n             0 => Nil\n             _ => loop!(n-1)\n           }\n         } |\n         loop!(10)\n       }"),
// 3892i64),
      (String::from("42 | @0!(2) | for (x <- @0) { Nil }"), 336i64),
      (String::from("@1!(1) |\n        for(x <- @1) { Nil } |\n        new x in { x!(10) | for(X <- x) { @2!(Set(X!(7)).add(*X).contains(10)) }} |\n        match 42 {\n          38 => Nil\n          42 =>
@3!(42)\n        }\n     "), 1264i64),
      //(String::from("new ret, keccak256Hash(`rho:crypto:keccak256Hash`) in {\n       |  keccak256Hash!(\"TEST\".toByteArray(), *ret) |\n       |  for (_ <- ret) { Nil }\n       |}"), 782i64),
]
}

#[tokio::test]
async fn total_cost_of_evaluation_should_be_equal_to_the_sum_of_all_costs_in_the_log() {
    for (contract, expected_total_cost) in contracts() {
        let initial_phlo = 10000i64;
        let eval_result = evaluate_with_cost_log(initial_phlo, contract.clone()).await;
        println!(
            "Contract: {}, Expected cost: {}, Actual cost: {}",
            contract, expected_total_cost, eval_result.cost.value
        );
        assert_eq!(
            eval_result.cost,
            Cost::create(expected_total_cost, "subtraction".to_string())
        );
        assert_eq!(eval_result.errors, Vec::new());

        //TODO add costLog asserts
    }
}
