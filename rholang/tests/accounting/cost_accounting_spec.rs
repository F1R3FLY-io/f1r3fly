// See rholang/src/test/scala/coop/rchain/rholang/interpreter/accounting/CostAccountingSpec.scala

use models::rhoapi::{BindPattern, ListParWithRandom, Par, TaggedContinuation};
use rholang::rust::interpreter::{
    accounting::costs::Cost,
    interpreter::EvaluateResult,
    matcher::r#match::Matcher,
    rho_runtime::{create_rho_runtime, RhoRuntime},
};
use rspace_plus_plus::rspace::{
    rspace::RSpace,
    shared::{
        in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
    },
};
use std::sync::{Arc, Mutex};

async fn evaluate_with_cost_log(initial_phlo: i64, contract: String) -> EvaluateResult {
    // let cost_log: Vec<Cost> = Vec::new();
    let mut kvm = InMemoryStoreManager::new();
    let store = kvm.r_space_stores().await.unwrap();
    let space: RSpace<Par, BindPattern, ListParWithRandom, TaggedContinuation> =
        RSpace::create(store, Arc::new(Box::new(Matcher))).unwrap();
    let runtime = create_rho_runtime(
        Arc::new(Mutex::new(Box::new(space))),
        Par::default(),
        false,
        &mut Vec::new(),
    )
    .await;

    let eval_result = runtime
        .try_lock()
        .unwrap()
        .evaluate_with_phlo(
            contract,
            Cost::create(initial_phlo, "cost_accounting_spec setup".to_string()),
        )
        .await;
    assert!(eval_result.is_ok());
    // (
    //     eval_result.as_ref().unwrap().clone(),
    //     eval_result.unwrap().cost,
    // )
    eval_result.unwrap()
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
      (String::from("new loop in {\n         contract loop(@n) = {\n           match n {\n             0 => Nil\n             _ => loop!(n-1)\n           }\n         } |\n         loop!(10)\n       }"),
3892i64),
      (String::from("42 | @0!(2) | for (x <- @0) { Nil }"), 336i64),
      (String::from("@1!(1) |\n        for(x <- @1) { Nil } |\n        new x in { x!(10) | for(X <- x) { @2!(Set(X!(7)).add(*X).contains(10)) }} |\n        match 42 {\n          38 => Nil\n          42 =>
@3!(42)\n        }\n     "), 1264i64),
      (String::from("new ret, keccak256Hash(`rho:crypto:keccak256Hash`) in {\n       |  keccak256Hash!(\"TEST\".toByteArray(), *ret) |\n       |  for (_ <- ret) { Nil }\n       |}"), 782i64),
]
}

// TODO: This function is also supposed to test for the sum of all individual costs within the evaluate call
#[tokio::test]
async fn total_cost_of_evaluation_should_be_equal_to_the_sum_of_all_costs_in_the_log() {
    // for (contract, expected_total_cost) in contracts() {
    //     let initial_phlo = 10000i64;
    //     let eval_result = evaluate_with_cost_log(initial_phlo, contract).await;
    //     assert_eq!(
    //         eval_result.cost,
    //         Cost::create(expected_total_cost, "cost_assertion".to_string())
    //     );
    //     assert_eq!(eval_result.errors, Vec::new());
    // }
}
