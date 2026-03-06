use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use swipl::atom;
use swipl::context::{ActivatedEngine, Context};
use swipl::engine::Engine;
use swipl::init::initialize_swipl;
use swipl::pred;
use swipl::term;
use tempfile::NamedTempFile;

use super::errors::InterpreterError;

#[derive(Clone)]
pub struct SwiplClient {
    engine: Arc<Engine>
}

impl SwiplClient {
    pub fn new() -> Self {
        let engine = Arc::new(Engine::new());

        Self {
            engine
        }
    }

    pub fn evaluate(&self, swi_prolog_code: &str) -> Result<(), InterpreterError> {
        let context: Context<_> = self.engine.activate().into();

        let term = term! {context: swi_prolog_code}
            .map_err(|_| InterpreterError::SwiplError("".into()))?;
        let query = context.open_call(&term);

        Ok(())
    }

    pub fn petta_compile(&self, metta_code: &str) -> Result<String, InterpreterError> {
        // Write the MeTTa code to a temp file
        let mut metta_file = NamedTempFile::new()
            .map_err(|_| InterpreterError::SwiplError("Can't open temp file".into()))?;
        metta_file.write(metta_code.as_bytes())
            .map_err(|_| InterpreterError::SwiplError("Can't write MeTTa code to temp file".into()))?;

        // Get the path to PeTTa
        let root = Path::new(env!("CARGO_MANIFEST_DIR")); 
        let petta_path = root.join("../PeTTa/src/main.pl");

        // Execute PeTTa
        let output = Command::new("swipl")
            .arg(petta_path)
            .arg(metta_file.path())
            .output()
            .map_err(|_| InterpreterError::SwiplError("Can't translate MeTTa with PeTTa".into()))?;

        // Get output as string
        String::from_utf8(output.stdout)
            .map_err(|_| InterpreterError::SwiplError("Can't interpret PeTTa output".into()))
    }
}