// See casper/src/test/scala/coop/rchain/casper/helper/RhoLoggerContract.scala

use models::rhoapi::{ListParWithRandom, Par};
use rholang::rust::interpreter::{
    contract_call::ContractCall,
    errors::{illegal_argument_error, InterpreterError},
    pretty_printer::PrettyPrinter,
    rho_type::RhoString,
    system_processes::ProcessContext,
};

pub struct RhoLoggerContract;

impl RhoLoggerContract {
    pub async fn std_log(
        ctx: ProcessContext,
        message: (Vec<ListParWithRandom>, bool, Vec<Par>),
    ) -> Result<Vec<Par>, InterpreterError> {
        let mut pretty_printer = PrettyPrinter::new();

        let is_contract_call = ContractCall {
            space: ctx.space.clone(),
            dispatcher: ctx.dispatcher.clone(),
        };

        if let Some((_, _, _, args)) = is_contract_call.unapply(message) {
            match args.as_slice() {
                [log_level_par, par] => {
                    if let Some(log_level) = RhoString::unapply(log_level_par) {
                        let msg = pretty_printer.build_string_from_message(par);

                        match log_level.as_str() {
                            "trace" => {
                                println!("trace: {}", msg);
                                Ok(vec![])
                            }
                            "debug" => {
                                println!("debug: {}", msg);
                                Ok(vec![])
                            }
                            "info" => {
                                println!("info: {}", msg);
                                Ok(vec![])
                            }
                            "warn" => {
                                println!("warn: {}", msg);
                                Ok(vec![])
                            }
                            "error" => {
                                println!("error: {}", msg);
                                Ok(vec![])
                            }
                            _ => Err(illegal_argument_error("std_log")),
                        }
                    } else {
                        Err(illegal_argument_error("std_log"))
                    }
                }
                _ => Err(illegal_argument_error("std_log")),
            }
        } else {
            Err(illegal_argument_error("std_log"))
        }
    }
}
