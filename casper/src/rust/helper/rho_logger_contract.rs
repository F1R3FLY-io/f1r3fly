// See casper/src/test/scala/coop/rchain/casper/helper/RhoLoggerContract.scala

use models::rhoapi::ListParWithRandom;
use rholang::rust::interpreter::{
    contract_call::ContractCall, pretty_printer::PrettyPrinter, rho_type::RhoString,
    system_processes::ProcessContext,
};

pub struct RhoLoggerContract;

impl RhoLoggerContract {
    pub async fn std_log(ctx: ProcessContext, message: Vec<ListParWithRandom>) -> () {
        let mut pretty_printer = PrettyPrinter::new();

        let is_contract_call = ContractCall {
            space: ctx.space.clone(),
            dispatcher: ctx.dispatcher.clone(),
        };

        if let Some((_, args)) = is_contract_call.unapply(message) {
            match args.as_slice() {
                [log_level_par, par] => {
                    if let Some(log_level) = RhoString::unapply(log_level_par) {
                        let msg = pretty_printer.build_string_from_message(par);

                        match log_level.as_str() {
                            "trace" => println!("trace: {}", msg),
                            "debug" => println!("debug: {}", msg),
                            "info" => println!("info: {}", msg),
                            "warn" => println!("warn: {}", msg),
                            "error" => println!("error: {}", msg),
                            _ => (),
                        }
                    } else {
                        ()
                    }
                }
                _ => (),
            }
        } else {
            panic!("SystemProcesses: is_contract_call failed");
        }
    }
}
