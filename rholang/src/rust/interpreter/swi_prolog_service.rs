use std::io::Write;
use std::path::PathBuf;
use std::{env, time::Duration};
// use std::process::Command;
use tokio::process::Command;

use models::rhoapi::Par;
use serde_json::Value;
use tempfile::NamedTempFile;

use crate::rust::interpreter::rho_type::{
    RhoBoolean, RhoList, RhoMap, RhoNil, RhoNumber, RhoString,
};

use super::errors::InterpreterError;

pub async fn petta_execute(metta_code: &str) -> Result<Par, InterpreterError> {
    // Write the MeTTa code to a temp file
    let mut metta_file = NamedTempFile::new()
        .map_err(|_| InterpreterError::SwiplError("Can't open temp file".into()))?;
    metta_file
        .write(metta_code.as_bytes())
        .map_err(|_| InterpreterError::SwiplError("Can't write MeTTa code to temp file".into()))?;

    let metta_file_path = metta_file
        .path()
        .to_str()
        .ok_or(InterpreterError::SwiplError(
            "Can't convert metta_file path to string".into(),
        ))?;

    // Get the path to PeTTa
    let metta_module_path: PathBuf = {
        let petta_path = PathBuf::from(env::var("PETTA_PATH").unwrap_or("./PeTTa".into()));
        [petta_path, PathBuf::from("src/metta.pl")].iter().collect()
    };

    if !metta_module_path.exists() {
        return Err(InterpreterError::SwiplError("Can't find PeTTa.".into()));
    }

    let goal = format!(
        r#"assertz(silent(true)),
           load_metta_file('{metta_file_path}', Results),
           use_module(library(json)),
           json_write_dict(current_output, #{{results:Results}})."#
    );

    // TODO: Make this a configuration parameter
    let timeout_secs: u64 = 10;
    let proc_handle = tokio::spawn(tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        Command::new("swipl")
            .arg("-s")
            .arg(metta_module_path)
            .arg("-g")
            .arg(goal)
            .arg("-t")
            .arg("halt")
            .output(),
    ));

    let output = proc_handle
        .await
        .map_err(|join_error| {
            InterpreterError::SwiplError(
                format!("Error while joining with the PeTTa task: {}", join_error).into(),
            )
        })?
        .map_err(|elapsed| {
            InterpreterError::SwiplError(
                format!("MeTTa execution timed out after {}", elapsed).into(),
            )
        })?
        .map_err(|e| {
            InterpreterError::SwiplError(format!("MeTTa execution failed: {}", e).into())
        })?;

    if !output.status.success() {
        return Err(InterpreterError::SwiplError(
            format!("PeTTa execution failed. {:#?}", output.stderr.as_slice()).into(),
        ));
    }

    // Get output as string
    let str_output = String::from_utf8(output.stdout)
        .map_err(|_| InterpreterError::SwiplError("Can't interpret PeTTa output".into()))?;

    let value_output = serde_json::from_str::<Value>(str_output.as_str()).map_err(|_| {
        InterpreterError::SwiplError("Can't parse JSON output from PeTTa execution".into())
    })?;
    let par_output = value_to_par(value_output)?;
    Ok(par_output)
}

fn value_to_par(v: Value) -> Result<Par, InterpreterError> {
    match v {
        Value::Null => Ok(RhoNil::create_par()),
        Value::Bool(b) => Ok(RhoBoolean::create_par(b)),
        Value::Number(n) => {
            let n64 = n.as_i64().ok_or(InterpreterError::SwiplError(
                "Could not parse number as i64".into(),
            ))?;
            Ok(RhoNumber::create_par(n64))
        }
        Value::String(s) => Ok(RhoString::create_par(s)),
        Value::Array(values) => {
            let ps = values
                .into_iter()
                .map(value_to_par)
                .collect::<Result<_, _>>()?;
            Ok(RhoList::create_par(ps))
        }
        Value::Object(map) => {
            let hashmap = map
                .into_iter()
                .map(|(k, v)| {
                    let p = value_to_par(v)?;
                    Ok((RhoString::create_par(k), p))
                })
                .collect::<Result<_, InterpreterError>>()?;
            Ok(RhoMap::create_par(hashmap))
        }
    }
}
