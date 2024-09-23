// See rholang/src/test/scala/coop/rchain/rholang/interpreter/SubstituteTest.scala

use models::rhoapi::{Connective, Par};
use rholang::rust::interpreter::env::Env;

const DEPTH: i32 = 0;

fn env() -> Env<Par> {
    Env::new()
}

#[tokio::test]
async fn substitute_should_retain_all_non_empty_par_ed_connectives() {
    let sample_connectives: Vec<Connective> = vec![];

    todo!();
}
