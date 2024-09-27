// See rholang/src/test/scala/coop/rchain/rholang/interpreter/accounting/CostAccountingSpec.scala

use rholang::rust::interpreter::{
    accounting::costs::Cost,
    interpreter::EvaluateResult,
    rho_runtime::{ReplayRhoRuntime, RhoHistoryRepository, RhoRuntime, RhoRuntimeImpl},
    system_processes::Definition,
};
use rspace_plus_plus::rspace::{rspace::{RSpace, RSpaceInstances}, shared::{
    in_mem_store_manager::InMemoryStoreManager, key_value_store_manager::KeyValueStoreManager,
}};

async fn evaluate_with_cost_log(
    initial_phlo: u64,
    contract: String,
) -> (EvaluateResult, Vec<Cost>) {
    let cost_log: Vec<Cost> = Vec::new();
    // let hrstores = RSpaceInstances::cre
    // let spaces = create_runtime_with_cost_log

    todo!()
}

fn contracts() -> Vec<(String, u64)> {
    vec![
      (String::from("@0!(2)"), 97u64),
      (String::from("@0!(2) | @1!(1)"), 197u64),
      (String::from("for(x <- @0){ Nil }"), 128u64),
      (String::from("for(x <- @0){ Nil } | @0!(2)"), 329u64),
      (String::from("@0!!(0) | for (_ <- @0) { 0 }"), 342u64),
      (String::from("@0!!(0) | for (x <- @0) { 0 }"), 342u64),
      (String::from("@0!!(0) | for (@0 <- @0) { 0 }"), 336u64),
      (String::from("@0!!(0) | @0!!(0) | for (_ <- @0) { 0 }"), 443u64),
      (String::from("@0!!(0) | @1!!(1) | for (_ <- @0 & _ <- @1) { 0 }"), 596u64),
      (String::from("@0!(0) | for (_ <- @0) { 0 }"), 333u64),
      (String::from("@0!(0) | for (x <- @0) { 0 }"), 333u64),
      (String::from("@0!(0) | for (@0 <- @0) { 0 }"), 327u64),
      (String::from("@0!(0) | for (_ <= @0) { 0 }"), 354u64),
      (String::from("@0!(0) | for (x <= @0) { 0 }"), 356u64),
      (String::from("@0!(0) | for (@0 <= @0) { 0 }"), 341u64),
      (String::from("@0!(0) | @0!(0) | for (_ <= @0) { 0 }"), 574u64),
      (String::from("@0!(0) | for (@0 <- @0) { 0 } | @0!(0) | for (_ <- @0) { 0 }"), 663u64),
      (String::from("@0!(0) | for (@0 <- @0) { 0 } | @0!(0) | for (@1 <- @0) { 0 }"), 551u64),
      (String::from("@0!(0) | for (_ <<- @0) { 0 }"), 406u64),
      (String::from("@0!!(0) | for (_ <<- @0) { 0 }"), 343u64),
      (String::from("@0!!(0) | @0!!(0) | for (_ <<- @0) { 0 }"), 444u64),
      (String::from("new loop in {\n         contract loop(@n) = {\n           match n {\n             0 => Nil\n             _ => loop!(n-1)\n           }\n         } |\n         loop!(10)\n       }"),
3892u64),
      (String::from("42 | @0!(2) | for (x <- @0) { Nil }"), 336u64),
      (String::from("@1!(1) |\n        for(x <- @1) { Nil } |\n        new x in { x!(10) | for(X <- x) { @2!(Set(X!(7)).add(*X).contains(10)) }} |\n        match 42 {\n          38 => Nil\n          42 =>
@3!(42)\n        }\n     "), 1264u64),
      (String::from("new ret, keccak256Hash(`rho:crypto:keccak256Hash`) in {\n       |  keccak256Hash!(\"TEST\".toByteArray(), *ret) |\n       |  for (_ <- ret) { Nil }\n       |}"), 782u64),
]
}

#[test]
fn total_cost_of_evaluation_should_be_equal_to_the_sum_of_all_costs_in_the_log() {
    for (contract, expected_total_cost) in contracts() {
        let initial_plo = 10000u64;
    }
}
