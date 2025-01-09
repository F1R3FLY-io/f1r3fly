// See casper/src/test/scala/coop/rchain/casper/helper/RhoSpec.scala

// use crate::rust::helper::rho_logger_contract::RhoLoggerContract;

// use rholang::rust::interpreter::system_processes::{byte_name, Definition};
// use std::sync::Arc;

// use super::test_result_collector::TestResultCollector;

// pub fn test_framework_contracts(
//     test_result_collector: Arc<TestResultCollector>,
// ) -> Vec<Definition> {
//     vec![
//         Definition {
//             urn: "rho:test:assertAck".to_string(),
//             fixed_channel: byte_name(101),
//             arity: 5,
//             body_ref: 101,
//             handler: {
//                 let collector = Arc::clone(&test_result_collector);
//                 Box::new(move |ctx| {
//                     let collector = Arc::clone(&collector);
//                     Box::new(move |args| {
//                         let collector = Arc::clone(&collector);
//                         let ctx = ctx.clone();
//                         Box::pin(async move { collector.handle_message(ctx, args).await })
//                     })
//                 })
//             },
//             remainder: None,
//         },
//         Definition {
//             urn: "rho:test:testSuiteCompleted".to_string(),
//             fixed_channel: byte_name(102),
//             arity: 1,
//             body_ref: 102,
//             handler: {
//                 let collector = Arc::clone(&test_result_collector);
//                 Box::new(move |ctx| {
//                     let collector = Arc::clone(&collector);
//                     Box::new(move |args| {
//                         let collector = Arc::clone(&collector);
//                         let ctx = ctx.clone();
//                         Box::pin(async move { collector.handle_message(ctx, args).await })
//                     })
//                 })
//             },
//             remainder: None,
//         },
//         Definition {
//             urn: "rho:io:stdlog".to_string(),
//             fixed_channel: byte_name(103),
//             arity: 2,
//             body_ref: 103,
//             handler: {
//                 Box::new(move |ctx| {
//                     Box::new(move |args| {
//                         let ctx = ctx.clone();
//                         Box::pin(async move { RhoLoggerContract::std_log(ctx, args).await })
//                     })
//                 })
//             },
//             remainder: None,
//         },
//     ]
// }
