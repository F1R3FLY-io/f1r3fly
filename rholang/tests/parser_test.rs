use rholang::rust::interpreter::compiler::{
    normalizer::parser::parse_rholang_code_to_proc,
    rholang_ast::{Name, Proc, ProcList, SendType, Var},
};

/*
 * Not sure how to test line_num and row_num yet. Currently, using 'start_position' only.
 * From tree sitter node, how do we use 'start_position' and 'end_position' methods to match BNFC/Scala impl.
 *
*/

#[test]
fn parse_rholang_code_to_proc_should_successfully_parse_simple_send_of_string_literal() {
    let rholang_code = r#"x!("Hello")"#;
    let result = parse_rholang_code_to_proc(&rholang_code);

    println!("\n{:?}", result);
    assert!(result.is_ok());

    let expected_result = Proc::Send {
        name: Name::ProcVar(Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        }))),
        send_type: SendType::Single {
            line_num: 0,
            col_num: 1,
        },
        inputs: ProcList {
            procs: vec![Proc::StringLiteral {
                value: "Hello".to_string(),
                line_num: 0,
                col_num: 3,
            }],
            line_num: 0,
            col_num: 2,
        },
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

// #[test]
// fn parse_rholang_code_to_proc_should_successfully_parse_simple_send_of_bool_literal() {
//     let rholang_code = r#"x!(true)"#;
//     let result = parse_rholang_code_to_proc(&rholang_code);

//     // println!("\n{:?}", result);
//     assert!(result.is_ok());

//     let expected_result = Proc::Send {
//         name: Name::NameProcVar(ProcVar::Var(Var {
//             name: "x".to_string(),
//             line_num: 0,
//             col_num: 0,
//         })),
//         send_type: SendType::Single {
//             line_num: 0,
//             col_num: 1,
//         },
//         inputs: ProcList {
//             procs: vec![Proc::BoolLiteral {
//                 value: true,
//                 line_num: 0,
//                 col_num: 3,
//             }],
//             line_num: 0,
//             col_num: 2,
//         },
//         line_num: 0,
//         col_num: 0,
//     };

//     assert_eq!(result.unwrap(), expected_result)
// }
