// See casper/src/main/scala/coop/rchain/casper/util/rholang/InterpreterUtil.scala

use models::rust::casper::pretty_printer::PrettyPrinter;
use prost::bytes::Bytes;
use rholang::rust::interpreter::errors::InterpreterError;

pub fn print_deploy_errors(deploy_sig: &Bytes, errors: &[InterpreterError]) {
    let deploy_info = PrettyPrinter::build_string_sig(&deploy_sig);
    let error_messages: String = errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    println!("Deploy ({}) errors: {}", deploy_info, error_messages);

    log::warn!("Deploy ({}) errors: {}", deploy_info, error_messages);
}
