use rholang::rust::interpreter::compiler::{
    normalizer::parser::parse_rholang_code_to_proc,
    rholang_ast::{Collection, Name, Proc, ProcList, SendType, UriLiteral, Var},
};

// println!("\n{:?}", result);

#[test]
fn parse_rholang_code_to_proc_should_handle_string_literal() {
    let input_code = r#""Hello, Rholang!""#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
    let expected_result = Proc::StringLiteral {
        value: "Hello, Rholang!".to_owned(),
        line_num: 0,
        col_num: 0,
    };
    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_bool_literal() {
    let input_code = "true";
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
    let expected_result = Proc::BoolLiteral {
        value: true,
        line_num: 0,
        col_num: 0,
    };
    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_uri_literal() {
    let input_code = "`http://example.com`";
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
    let expected_result = Proc::UriLiteral(UriLiteral {
        value: "`http://example.com`".to_string(),
        line_num: 0,
        col_num: 0,
    });
    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_int_literal() {
    let input_code = "42";
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
    let expected_result = Proc::LongLiteral {
        value: 42,
        line_num: 0,
        col_num: 0,
    };
    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_nil() {
    let input_code = "Nil";
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
    let expected_result = Proc::Nil {
        line_num: 0,
        col_num: 0,
    };
    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_collection_list() {
    let input_code = "[1,2,3]";
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
    let expected_result = Proc::Collection(Collection::List {
        elements: vec![
            Proc::new_int_proc(1, 0, 1),
            Proc::new_int_proc(2, 0, 3),
            Proc::new_int_proc(3, 0, 5),
        ],
        cont: None,
        line_num: 0,
        col_num: 0,
    });
    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_successfully_parse_simple_send() {
    let rholang_code = r#"x!("Hello")"#;
    let result = parse_rholang_code_to_proc(&rholang_code);

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
