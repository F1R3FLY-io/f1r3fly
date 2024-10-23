use rholang::rust::interpreter::compiler::{
    normalizer::parser::parse_rholang_code_to_proc,
    rholang_ast::{
        Block, Branch, BundleType, Case, Collection, Decl, Decls, DeclsChoice, KeyValuePair,
        LinearBind, Name, NameDecl, Names, Proc, ProcList, Quotable, Quote, Receipt, Receipts,
        SendType, Source, SyncSendCont, UriLiteral, Var,
    },
};

// println!("\n{:#?}", result);

#[test]
fn parse_rholang_code_to_proc_should_parse_par() {
    let input_code = r#"
       new a, b in {
         a!() | b!()
       }  
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![
                NameDecl {
                    var: Var {
                        name: "a".to_string(),
                        line_num: 1,
                        col_num: 11,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 11,
                },
                NameDecl {
                    var: Var {
                        name: "b".to_string(),
                        line_num: 1,
                        col_num: 14,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 14,
                },
            ],
            line_num: 1,
            col_num: 11,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Par {
                left: Box::new(Proc::Send {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "a".to_string(),
                        line_num: 2,
                        col_num: 9,
                    }))),
                    send_type: SendType::Single {
                        line_num: 2,
                        col_num: 9,
                    },
                    inputs: ProcList {
                        procs: vec![],
                        line_num: 2,
                        col_num: 11,
                    },
                    line_num: 2,
                    col_num: 9,
                }),
                right: Box::new(Proc::Send {
                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "b".to_string(),
                        line_num: 2,
                        col_num: 16,
                    }))),
                    send_type: SendType::Single {
                        line_num: 2,
                        col_num: 16,
                    },
                    inputs: ProcList {
                        procs: vec![],
                        line_num: 2,
                        col_num: 18,
                    },
                    line_num: 2,
                    col_num: 16,
                }),
                line_num: 2,
                col_num: 9,
            },
            line_num: 1,
            col_num: 19,
        }))),
        line_num: 1,
        col_num: 7,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_send_sync() {
    let input_code = r#"
      new myChannel in {
        myChannel!?("Test Message", _).
      }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![NameDecl {
                var: Var {
                    name: "myChannel".to_string(),
                    line_num: 1,
                    col_num: 10,
                },
                uri: None,
                line_num: 1,
                col_num: 10,
            }],
            line_num: 1,
            col_num: 10,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::SendSync {
                name: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "myChannel".to_string(),
                    line_num: 2,
                    col_num: 8,
                }))),
                messages: ProcList {
                    procs: vec![
                        Proc::StringLiteral {
                            value: "Test Message".to_string(),
                            line_num: 2,
                            col_num: 20,
                        },
                        Proc::Wildcard {
                            line_num: 2,
                            col_num: 36,
                        },
                    ],
                    line_num: 2,
                    col_num: 19,
                },
                cont: SyncSendCont::Empty {
                    line_num: 2,
                    col_num: 38,
                },
                line_num: 2,
                col_num: 8,
            },
            line_num: 1,
            col_num: 23,
        }))),
        line_num: 1,
        col_num: 6,
    };

    assert_eq!(result.unwrap(), expected_result)
}

// Also tests 'uri' within 'name_decl'
#[test]
fn parse_rholang_code_to_proc_should_parse_new() {
    let input_code = r#"
       new x(`rho:registry:lookup`) in {
         Nil
       }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![NameDecl {
                var: Var {
                    name: "x".to_string(),
                    line_num: 1,
                    col_num: 11,
                },
                uri: Some(UriLiteral {
                    value: "`rho:registry:lookup`".to_string(),
                    line_num: 1,
                    col_num: 13,
                }),
                line_num: 1,
                col_num: 11,
            }],
            line_num: 1,
            col_num: 11,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 9,
            },
            line_num: 1,
            col_num: 39,
        }))),
        line_num: 1,
        col_num: 7,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_if_else() {
    let input_code = r#"
       if (true) {
         Nil
       } else {
         Nil
       }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::IfElse {
        condition: Box::new(Proc::BoolLiteral {
            value: true,
            line_num: 1,
            col_num: 11,
        }),
        if_true: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 9,
            },
            line_num: 1,
            col_num: 17,
        }))),
        alternative: Some(Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Nil {
                line_num: 4,
                col_num: 9,
            },
            line_num: 3,
            col_num: 14,
        })))),
        line_num: 1,
        col_num: 7,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_if_without_else() {
    let input_code = r#"
       if (true) {
         Nil
       }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::IfElse {
        condition: Box::new(Proc::BoolLiteral {
            value: true,
            line_num: 1,
            col_num: 11,
        }),
        if_true: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 9,
            },
            line_num: 1,
            col_num: 17,
        }))),
        alternative: None,
        line_num: 1,
        col_num: 7,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_let() {
    let input_code = r#"
        let x = 5; y = 10 in {
          x + y
        }
      "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Let {
        decls: DeclsChoice::LinearDecls {
            decls: vec![
                Decl {
                    names: Names {
                        names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "x".to_string(),
                            line_num: 1,
                            col_num: 12,
                        })))],
                        cont: None,
                        line_num: 1,
                        col_num: 12,
                    },
                    procs: vec![Proc::LongLiteral {
                        value: 5,
                        line_num: 1,
                        col_num: 16,
                    }],
                    line_num: 1,
                    col_num: 12,
                },
                Decl {
                    names: Names {
                        names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "y".to_string(),
                            line_num: 1,
                            col_num: 19,
                        })))],
                        cont: None,
                        line_num: 1,
                        col_num: 19,
                    },
                    procs: vec![Proc::LongLiteral {
                        value: 10,
                        line_num: 1,
                        col_num: 23,
                    }],
                    line_num: 1,
                    col_num: 19,
                },
            ],
            line_num: 1,
            col_num: 12,
        },
        body: Box::new(Block {
            proc: Proc::Add {
                left: Box::new(Proc::Var(Var {
                    name: "x".to_string(),
                    line_num: 2,
                    col_num: 10,
                })),
                right: Box::new(Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 2,
                    col_num: 14,
                })),
                line_num: 2,
                col_num: 10,
            },
            line_num: 1,
            col_num: 29,
        }),
        line_num: 1,
        col_num: 8,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_bundle() {
    let input_code_bundle_write = r#"
        bundle+ {Nil}
      "#;

    let bundle_write_result = parse_rholang_code_to_proc(&input_code_bundle_write);
    assert!(bundle_write_result.is_ok());

    let bundle_write_expected_result = Proc::Bundle {
        bundle_type: BundleType::BundleWrite {
            line_num: 1,
            col_num: 8,
        },
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 1,
                col_num: 17,
            },
            line_num: 1,
            col_num: 16,
        }),
        line_num: 1,
        col_num: 8,
    };

    assert_eq!(bundle_write_result.unwrap(), bundle_write_expected_result);

    let input_code_bundle_read = r#"
        bundle- {Nil}
      "#;

    let bundle_read_result = parse_rholang_code_to_proc(&input_code_bundle_read);
    assert!(bundle_read_result.is_ok());

    let bundle_read_expected_result = Proc::Bundle {
        bundle_type: BundleType::BundleRead {
            line_num: 1,
            col_num: 8,
        },
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 1,
                col_num: 17,
            },
            line_num: 1,
            col_num: 16,
        }),
        line_num: 1,
        col_num: 8,
    };

    assert_eq!(bundle_read_result.unwrap(), bundle_read_expected_result);

    let input_code_bundle_equiv = r#"
        bundle0 {Nil}
      "#;

    let bundle_equiv_result = parse_rholang_code_to_proc(&input_code_bundle_equiv);
    assert!(bundle_equiv_result.is_ok());

    let bundle_equiv_expected_result = Proc::Bundle {
        bundle_type: BundleType::BundleEquiv {
            line_num: 1,
            col_num: 8,
        },
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 1,
                col_num: 17,
            },
            line_num: 1,
            col_num: 16,
        }),
        line_num: 1,
        col_num: 8,
    };

    assert_eq!(bundle_equiv_result.unwrap(), bundle_equiv_expected_result);

    let input_code_bundle_read_write = r#"
        bundle {Nil}
      "#;

    let bundle_read_write_result = parse_rholang_code_to_proc(&input_code_bundle_read_write);
    assert!(bundle_read_write_result.is_ok());

    let bundle_read_write_expected_result = Proc::Bundle {
        bundle_type: BundleType::BundleReadWrite {
            line_num: 1,
            col_num: 8,
        },
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 1,
                col_num: 16,
            },
            line_num: 1,
            col_num: 15,
        }),
        line_num: 1,
        col_num: 8,
    };

    assert_eq!(
        bundle_read_write_result.unwrap(),
        bundle_read_write_expected_result
    );
}

#[test]
fn parse_rholang_code_to_proc_should_parse_match() {
    let input_code = r#"
         match x {
           1 => { @"one"!("Matched one") }
           true => { @"true"!("Matched true") }
           _ => { @"default"!("Matched default") }
         }
      "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Match {
        expression: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 1,
            col_num: 15,
        })),
        cases: vec![
            Case {
                pattern: Proc::LongLiteral {
                    value: 1,
                    line_num: 2,
                    col_num: 11,
                },
                proc: Proc::Block(Box::new(Block {
                    proc: Proc::Send {
                        name: Name::Quote(Box::new(Quote {
                            quotable: Box::new(Quotable::GroundExpression(Proc::StringLiteral {
                                value: "one".to_string(),
                                line_num: 2,
                                col_num: 19,
                            })),
                            line_num: 2,
                            col_num: 18,
                        })),
                        send_type: SendType::Single {
                            line_num: 2,
                            col_num: 18,
                        },
                        inputs: ProcList {
                            procs: vec![Proc::StringLiteral {
                                value: "Matched one".to_string(),
                                line_num: 2,
                                col_num: 26,
                            }],
                            line_num: 2,
                            col_num: 25,
                        },
                        line_num: 2,
                        col_num: 18,
                    },
                    line_num: 2,
                    col_num: 16,
                })),
                line_num: 2,
                col_num: 11,
            },
            Case {
                pattern: Proc::BoolLiteral {
                    value: true,
                    line_num: 3,
                    col_num: 11,
                },
                proc: Proc::Block(Box::new(Block {
                    proc: Proc::Send {
                        name: Name::Quote(Box::new(Quote {
                            quotable: Box::new(Quotable::GroundExpression(Proc::StringLiteral {
                                value: "true".to_string(),
                                line_num: 3,
                                col_num: 22,
                            })),
                            line_num: 3,
                            col_num: 21,
                        })),
                        send_type: SendType::Single {
                            line_num: 3,
                            col_num: 21,
                        },
                        inputs: ProcList {
                            procs: vec![Proc::StringLiteral {
                                value: "Matched true".to_string(),
                                line_num: 3,
                                col_num: 30,
                            }],
                            line_num: 3,
                            col_num: 29,
                        },
                        line_num: 3,
                        col_num: 21,
                    },
                    line_num: 3,
                    col_num: 19,
                })),
                line_num: 3,
                col_num: 11,
            },
            Case {
                pattern: Proc::Wildcard {
                    line_num: 4,
                    col_num: 11,
                },
                proc: Proc::Block(Box::new(Block {
                    proc: Proc::Send {
                        name: Name::Quote(Box::new(Quote {
                            quotable: Box::new(Quotable::GroundExpression(Proc::StringLiteral {
                                value: "default".to_string(),
                                line_num: 4,
                                col_num: 19,
                            })),
                            line_num: 4,
                            col_num: 18,
                        })),
                        send_type: SendType::Single {
                            line_num: 4,
                            col_num: 18,
                        },
                        inputs: ProcList {
                            procs: vec![Proc::StringLiteral {
                                value: "Matched default".to_string(),
                                line_num: 4,
                                col_num: 30,
                            }],
                            line_num: 4,
                            col_num: 29,
                        },
                        line_num: 4,
                        col_num: 18,
                    },
                    line_num: 4,
                    col_num: 16,
                })),
                line_num: 4,
                col_num: 11,
            },
        ],
        line_num: 1,
        col_num: 9,
    };

    assert_eq!(result.unwrap(), expected_result)
}

// aka 'choice'
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

// Also tests 'cont' within 'names'
#[test]
fn parse_rholang_code_to_proc_should_parse_contract() {
    let input_code = r#"
       contract @"example"(x, y ...@rest) = {
         Nil
       }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Contract {
        name: Name::Quote(Box::new(Quote {
            quotable: Box::new(Quotable::GroundExpression(Proc::StringLiteral {
                value: "example".to_string(),
                line_num: 1,
                col_num: 17,
            })),
            line_num: 1,
            col_num: 16,
        })),
        formals: Names {
            names: vec![
                Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "x".to_string(),
                    line_num: 1,
                    col_num: 27,
                }))),
                Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 1,
                    col_num: 30,
                }))),
            ],
            cont: Some(Box::new(Proc::Var(Var {
                name: "rest".to_string(),
                line_num: 1,
                col_num: 36,
            }))),
            line_num: 1,
            col_num: 27,
        },
        proc: Box::new(Block {
            proc: Proc::Nil {
                line_num: 2,
                col_num: 9,
            },
            line_num: 1,
            col_num: 44,
        }),
        line_num: 1,
        col_num: 7,
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
fn parse_rholang_code_to_proc_should_parse_or() {
    let input_code = r#"x or y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Or {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_and() {
    let input_code = r#"x and y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::And {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 6,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_matches() {
    let input_code = r#"x matches y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Matches {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 10,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_eq() {
    let input_code = r#"x == y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Eq {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_neq() {
    let input_code = r#"x != y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Neq {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_lt() {
    let input_code = r#"x < y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Lt {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_lte() {
    let input_code = r#"x <= y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Lte {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_gt() {
    let input_code = r#"x > y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Gt {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_gte() {
    let input_code = r#"x >= y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Gte {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_concat() {
    let input_code = r#"x ++ y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Concat {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_minus_minus() {
    let input_code = r#"x -- y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::MinusMinus {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_minus() {
    let input_code = r#"x - y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Minus {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_add() {
    let input_code = r#"x + y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Add {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_percent_percent() {
    let input_code = r#"x %% y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::PercentPercent {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_mult() {
    let input_code = r#"x * y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Mult {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_div() {
    let input_code = r#"x / y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Div {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_mod() {
    let input_code = r#"x % y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Mod {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_not() {
    let input_code = r#"not x"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Not {
        proc: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 4,
        })),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_neg() {
    let input_code = r#"-x"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Neg {
        proc: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 1,
        })),
        line_num: 0,
        col_num: 0,
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
fn parse_rholang_code_to_proc_should_handle_list_with_continuation() {
    let input_code = r#"[1, "two", true, `rho:uri` ...Nil]"#;
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
        ],
        cont: Some(Box::new(Proc::Var(Var {
            name: "Nil".to_string(),
            line_num: 0,
            col_num: 30,
        }))),
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
fn parse_rholang_code_to_proc_should_handle_set_with_continuation() {
    let input_code = r#"Set(true, Nil ...true)"#;
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
        cont: Some(Box::new(Proc::Var(Var {
            name: "true".to_string(),
            line_num: 0,
            col_num: 17,
        }))),
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
fn parse_rholang_code_to_proc_should_handle_map_with_continuation() {
    let input_code = r#"{ "integer": 1, "string": "two" ...Nil }"#;
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
        cont: Some(Box::new(Proc::Var(Var {
            name: "Nil".to_string(),
            line_num: 0,
            col_num: 35,
        }))),
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
