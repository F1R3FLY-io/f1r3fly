// See rholang/src/test/scala/coop/rchain/rholang/InterpreterSpec.scala

use rholang::rust::interpreter::{
    errors::InterpreterError,
    interpreter::EvaluateResult,
    rho_runtime::{RhoRuntime, RhoRuntimeImpl},
    storage::storage_printer,
    test_utils::resources::with_runtime,
};

fn storage_contents(runtime: &RhoRuntimeImpl) -> String {
    storage_printer::pretty_print(runtime)
}

async fn success(runtime: &mut RhoRuntimeImpl, term: &str) -> Result<(), InterpreterError> {
    execute(runtime, term).await.map(|res| {
        assert!(
            res.errors.is_empty(),
            "{}",
            format!("Execution failed for: {}. Cause: {:?}", term, res.errors)
        )
    })
}

async fn failure(runtime: &mut RhoRuntimeImpl, term: &str) -> Result<(), InterpreterError> {
    execute(runtime, term).await.map(|res| {
        assert!(
            res.errors.is_empty(),
            "Expected {} to fail - it didn't.",
            term
        )
    })
}

async fn execute(
    runtime: &mut RhoRuntimeImpl,
    term: &str,
) -> Result<EvaluateResult, InterpreterError> {
    runtime.evaluate_with_term(term).await
}

#[tokio::test]
async fn interpreter_should_restore_rspace_to_its_prior_state_after_evaluation_error() {
    with_runtime("interpreter-spec-", |mut runtime| async move {
        let send_rho = "@{0}!(0)";

        let init_storage = storage_contents(&runtime);
        success(&mut runtime, send_rho).await.unwrap();
        let before_error = storage_contents(&runtime);
        println!("\nbefore_error: {}", before_error);
        assert!(before_error.contains(&send_rho));
    })
    .await
}
