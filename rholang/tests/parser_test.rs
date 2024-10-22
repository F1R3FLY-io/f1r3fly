use rholang::rust::interpreter::compiler::{
    normalizer::parser::parse_rholang_code_to_proc,
    rholang_ast::{
        Block, Branch, Collection, KeyValuePair, LinearBind, Name, Names, Proc, ProcList, Receipt,
        Receipts, SendType, Source, UriLiteral, Var,
    },
};

// println!("\n{:#?}", result);

#[test]
fn parse_rholang_code_to_proc_should_parse_select() {
    let input_code = r#"
      select {
        x <- chan1 & y <- chan2 => { Nil }
        z <- chan3 => { Nil }
      }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let branches = vec![
        Branch {
            pattern: vec![
                LinearBind {
                    names: Names {
                        names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "x".to_string(),
                            line_num: 2,
                            col_num: 8,
                        })))],
                        cont: None,
                        line_num: 2,
                        col_num: 8,
                    },
                    input: Source::Simple {
                        name: Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "chan1".to_string(),
                            line_num: 2,
                            col_num: 13,
                        }))),
                        line_num: 2,
                        col_num: 13,
                    },
                    line_num: 2,
                    col_num: 8,
                },
                LinearBind {
                    names: Names {
                        names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "y".to_string(),
                            line_num: 2,
                            col_num: 21,
                        })))],
                        cont: None,
                        line_num: 2,
                        col_num: 21,
                    },
                    input: Source::Simple {
                        name: Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "chan2".to_string(),
                            line_num: 2,
                            col_num: 26,
                        }))),
                        line_num: 2,
                        col_num: 26,
                    },
                    line_num: 2,
                    col_num: 21,
                },
            ],
            proc: Proc::Block(Box::new(Block {
                proc: Proc::Nil {
                    line_num: 2,
                    col_num: 37,
                },
                line_num: 2,
                col_num: 35,
            })),
            line_num: 2,
            col_num: 8,
        },
        Branch {
            pattern: vec![LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "z".to_string(),
                        line_num: 3,
                        col_num: 8,
                    })))],
                    cont: None,
                    line_num: 3,
                    col_num: 8,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "chan3".to_string(),
                        line_num: 3,
                        col_num: 13,
                    }))),
                    line_num: 3,
                    col_num: 13,
                },
                line_num: 3,
                col_num: 8,
            }],
            proc: Proc::Block(Box::new(Block {
                proc: Proc::Nil {
                    line_num: 3,
                    col_num: 24,
                },
                line_num: 3,
                col_num: 22,
            })),
            line_num: 3,
            col_num: 8,
        },
    ];

    let expected_result = Proc::Choice {
        branches,
        line_num: 1,
        col_num: 6,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_list() {
    let input_code = r#"[1, "two", true, `rho:uri`, Nil]"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Collection(Collection::List {
        elements: vec![
            Proc::LongLiteral {
                value: 1,
                line_num: 0,
                col_num: 1,
            },
            Proc::StringLiteral {
                value: "two".to_string(),
                line_num: 0,
                col_num: 4,
            },
            Proc::BoolLiteral {
                value: true,
                line_num: 0,
                col_num: 11,
            },
            Proc::UriLiteral(UriLiteral {
                value: "`rho:uri`".to_string(),
                line_num: 0,
                col_num: 17,
            }),
            Proc::Nil {
                line_num: 0,
                col_num: 28,
            },
        ],
        cont: None,
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_set() {
    let input_code = r#"Set(true, Nil)"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Collection(Collection::Set {
        elements: vec![
            Proc::BoolLiteral {
                value: true,
                line_num: 0,
                col_num: 4,
            },
            Proc::Nil {
                line_num: 0,
                col_num: 10,
            },
        ],
        cont: None,
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_map() {
    let input_code = r#"{ "integer": 1, "string": "two" }"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Collection(Collection::Map {
        pairs: vec![
            KeyValuePair {
                key: Proc::StringLiteral {
                    value: "integer".to_string(),
                    line_num: 0,
                    col_num: 2,
                },
                value: Proc::LongLiteral {
                    value: 1,
                    line_num: 0,
                    col_num: 13,
                },
                line_num: 0,
                col_num: 2,
            },
            KeyValuePair {
                key: Proc::StringLiteral {
                    value: "string".to_string(),
                    line_num: 0,
                    col_num: 16,
                },
                value: Proc::StringLiteral {
                    value: "two".to_string(),
                    line_num: 0,
                    col_num: 26,
                },
                line_num: 0,
                col_num: 16,
            },
        ],
        cont: None,
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_handle_tuple() {
    let input_code = r#"(1, true, Nil)"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Collection(Collection::Tuple {
        elements: vec![
            Proc::LongLiteral {
                value: 1,
                line_num: 0,
                col_num: 1,
            },
            Proc::BoolLiteral {
                value: true,
                line_num: 0,
                col_num: 4,
            },
            Proc::Nil {
                line_num: 0,
                col_num: 10,
            },
        ],
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result.unwrap(), expected_result)
}

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
fn parse_rholang_code_to_proc_should_parse_simple_send() {
    let input_code = r#"x!("Hello")"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Send {
        name: Name::ProcVar(Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        }))),
        send_type: SendType::Single {
            line_num: 0,
            col_num: 0,
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

#[test]
fn parse_rholang_code_to_proc_should_parse_simple_input_process() {
    let input_code = r#"
     for (x <- y) {
       Nil
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let receipts = Receipts {
        receipts: vec![Receipt::LinearBinds(LinearBind {
            names: Names {
                names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "x".to_string(),
                    line_num: 1,
                    col_num: 10,
                })))],
                cont: None,
                line_num: 1,
                col_num: 10,
            },
            input: Source::Simple {
                name: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 1,
                    col_num: 15,
                }))),
                line_num: 1,
                col_num: 15,
            },
            line_num: 1,
            col_num: 10,
        })],
        line_num: 1,
        col_num: 10,
    };

    let expected_result = Proc::Input {
        formals: receipts,
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 7,
            },
            line_num: 1,
            col_num: 18,
        }),
        line_num: 1,
        col_num: 5,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_input_with_multiple_receipts() {
    let input_code = r#"
     for (x <- y; a <- b) {
       Nil
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let receipts = Receipts {
        receipts: vec![
            Receipt::LinearBinds(LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "x".to_string(),
                        line_num: 1,
                        col_num: 10,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 10,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "y".to_string(),
                        line_num: 1,
                        col_num: 15,
                    }))),
                    line_num: 1,
                    col_num: 15,
                },
                line_num: 1,
                col_num: 10,
            }),
            Receipt::LinearBinds(LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "a".to_string(),
                        line_num: 1,
                        col_num: 18,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 18,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "b".to_string(),
                        line_num: 1,
                        col_num: 23,
                    }))),
                    line_num: 1,
                    col_num: 23,
                },
                line_num: 1,
                col_num: 18,
            }),
        ],
        line_num: 1,
        col_num: 10,
    };

    let expected_result = Proc::Input {
        formals: receipts,
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 7,
            },
            line_num: 1,
            col_num: 26,
        }),
        line_num: 1,
        col_num: 5,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_input_with_multiple_receipts_and_linear_binds() {
    let input_code = r#"
     for (x <- y & z <- w; a <- b & c <- d) {
       Nil
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let receipts = Receipts {
        receipts: vec![
            Receipt::LinearBinds(LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "x".to_string(),
                        line_num: 1,
                        col_num: 10,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 10,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "y".to_string(),
                        line_num: 1,
                        col_num: 15,
                    }))),
                    line_num: 1,
                    col_num: 15,
                },
                line_num: 1,
                col_num: 10,
            }),
            Receipt::LinearBinds(LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "z".to_string(),
                        line_num: 1,
                        col_num: 19,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 19,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "w".to_string(),
                        line_num: 1,
                        col_num: 24,
                    }))),
                    line_num: 1,
                    col_num: 24,
                },
                line_num: 1,
                col_num: 19,
            }),
            Receipt::LinearBinds(LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "a".to_string(),
                        line_num: 1,
                        col_num: 27,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 27,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "b".to_string(),
                        line_num: 1,
                        col_num: 32,
                    }))),
                    line_num: 1,
                    col_num: 32,
                },
                line_num: 1,
                col_num: 27,
            }),
            Receipt::LinearBinds(LinearBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "c".to_string(),
                        line_num: 1,
                        col_num: 36,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 36,
                },
                input: Source::Simple {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "d".to_string(),
                        line_num: 1,
                        col_num: 41,
                    }))),
                    line_num: 1,
                    col_num: 41,
                },
                line_num: 1,
                col_num: 36,
            }),
        ],
        line_num: 1,
        col_num: 10,
    };

    let expected_result = Proc::Input {
        formals: receipts,
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 7,
            },
            line_num: 1,
            col_num: 44,
        }),
        line_num: 1,
        col_num: 5,
    };

    assert_eq!(result.unwrap(), expected_result)
}
