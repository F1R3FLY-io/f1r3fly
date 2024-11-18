use rholang::rust::interpreter::compiler::{
    normalizer::parser::parse_rholang_code_to_proc,
    rholang_ast::{
        Block, Branch, BundleType, Case, Collection, Conjunction, Decl, Decls, DeclsChoice,
        Disjunction, Eval, KeyValuePair, LinearBind, Name, NameDecl, Names, Negation, PeekBind,
        Proc, ProcList, Quote, Receipt, Receipts, RepeatedBind, SendType, Source, SyncSendCont,
        UriLiteral, Var, VarRef, VarRefKind,
    },
};

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
                    value: "rho:registry:lookup".to_string(),
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
                            quotable: Box::new(Proc::StringLiteral {
                                value: "one".to_string(),
                                line_num: 2,
                                col_num: 19,
                            }),
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
                            quotable: Box::new(Proc::StringLiteral {
                                value: "true".to_string(),
                                line_num: 3,
                                col_num: 22,
                            }),
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
                            quotable: Box::new(Proc::StringLiteral {
                                value: "default".to_string(),
                                line_num: 4,
                                col_num: 19,
                            }),
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
            quotable: Box::new(Proc::StringLiteral {
                value: "example".to_string(),
                line_num: 1,
                col_num: 17,
            }),
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
fn parse_rholang_code_to_proc_should_parse_input_with_linear_binds() {
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
fn parse_rholang_code_to_proc_should_parse_input_with_repeated_binds() {
    let input_code = r#"
     for (x <= y; a <= b) {
       Nil
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let receipts = Receipts {
        receipts: vec![
            Receipt::RepeatedBinds(RepeatedBind {
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
                input: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 1,
                    col_num: 15,
                }))),
                line_num: 1,
                col_num: 10,
            }),
            Receipt::RepeatedBinds(RepeatedBind {
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
                input: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "b".to_string(),
                    line_num: 1,
                    col_num: 23,
                }))),
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
fn parse_rholang_code_to_proc_should_parse_input_with_peek_binds() {
    let input_code = r#"
     for (x <<- y; a <<- b) {
       Nil
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let receipts = Receipts {
        receipts: vec![
            Receipt::PeekBinds(PeekBind {
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
                input: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 1,
                    col_num: 16,
                }))),
                line_num: 1,
                col_num: 10,
            }),
            Receipt::PeekBinds(PeekBind {
                names: Names {
                    names: vec![Name::ProcVar(Box::new(Proc::Var(Var {
                        name: "a".to_string(),
                        line_num: 1,
                        col_num: 19,
                    })))],
                    cont: None,
                    line_num: 1,
                    col_num: 19,
                },
                input: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "b".to_string(),
                    line_num: 1,
                    col_num: 25,
                }))),
                line_num: 1,
                col_num: 19,
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
            col_num: 28,
        }),
        line_num: 1,
        col_num: 5,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_input_with_mixed_binds() {
    let input_code = r#"
     for (x <- y & z <- w; a <<- b & c <<- d) {
       Nil
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());
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
    let input_code = r#"-(364154898664707247)"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Neg {
        proc: Box::new(Proc::LongLiteral {
            value: 364154898664707247,
            line_num: 0,
            col_num: 2,
        }),
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_method() {
    let input_code = r#"x.method(y, z)"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Method {
        receiver: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 0,
        })),
        name: Var {
            name: "method".to_string(),
            line_num: 0,
            col_num: 2,
        },
        args: ProcList {
            procs: vec![
                Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 0,
                    col_num: 9,
                }),
                Proc::Var(Var {
                    name: "z".to_string(),
                    line_num: 0,
                    col_num: 12,
                }),
            ],
            line_num: 0,
            col_num: 8,
        },
        line_num: 0,
        col_num: 0,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_parenthesized() {
    let input_code = r#"(x + y)"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Add {
        left: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 1,
        })),
        right: Box::new(Proc::Var(Var {
            name: "y".to_string(),
            line_num: 0,
            col_num: 5,
        })),
        line_num: 0,
        col_num: 1,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_eval() {
    let input_code = r#"*x"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Eval(Eval {
        name: Name::ProcVar(Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 1,
        }))),
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_disjunction() {
    let input_code = r#"x \/ y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Disjunction(Disjunction {
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
    });

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_conjunction() {
    let input_code = r#"x /\ y"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Conjunction(Conjunction {
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
    });

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_negation() {
    let input_code = r#"~x"#;
    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::Negation(Negation {
        proc: Box::new(Proc::Var(Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 1,
        })),
        line_num: 0,
        col_num: 0,
    });

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
                value: "rho:uri".to_string(),
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
                value: "rho:uri".to_string(),
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
        value: "http://example.com".to_string(),
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

#[test]
fn parse_rholang_code_to_proc_should_parse_var_ref() {
    let input_code_proc = r#"=x"#;
    let result_proc = parse_rholang_code_to_proc(&input_code_proc);
    assert!(result_proc.is_ok());

    let expected_result_proc = Proc::VarRef(VarRef {
        var_ref_kind: VarRefKind::Proc,
        var: Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 1,
        },
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result_proc.unwrap(), expected_result_proc);

    let input_code_name = r#"=*x"#;
    let result_name = parse_rholang_code_to_proc(&input_code_name);
    assert!(result_name.is_ok());

    let expected_result_name = Proc::VarRef(VarRef {
        var_ref_kind: VarRefKind::Name,
        var: Var {
            name: "x".to_string(),
            line_num: 0,
            col_num: 2,
        },
        line_num: 0,
        col_num: 0,
    });

    assert_eq!(result_name.unwrap(), expected_result_name);
}

#[test]
fn parse_rholang_code_to_proc_should_ignore_line_comment() {
    let input_code = r#"
       // This is a line comment
       new x in {
         x!("Hello")
       }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![NameDecl {
                var: Var {
                    name: "x".to_string(),
                    line_num: 2,
                    col_num: 11,
                },
                uri: None,
                line_num: 2,
                col_num: 11,
            }],
            line_num: 2,
            col_num: 11,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Send {
                name: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "x".to_string(),
                    line_num: 3,
                    col_num: 9,
                }))),
                send_type: SendType::Single {
                    line_num: 3,
                    col_num: 9,
                },
                inputs: ProcList {
                    procs: vec![Proc::StringLiteral {
                        value: "Hello".to_string(),
                        line_num: 3,
                        col_num: 12,
                    }],
                    line_num: 3,
                    col_num: 11,
                },
                line_num: 3,
                col_num: 9,
            },
            line_num: 2,
            col_num: 16,
        }))),
        line_num: 2,
        col_num: 7,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_ignore_block_comment() {
    let input_code = r#"
       /* This is a block comment */
       new y in {
         y!("World")
       }
    "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![NameDecl {
                var: Var {
                    name: "y".to_string(),
                    line_num: 2,
                    col_num: 11,
                },
                uri: None,
                line_num: 2,
                col_num: 11,
            }],
            line_num: 2,
            col_num: 11,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Send {
                name: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "y".to_string(),
                    line_num: 3,
                    col_num: 9,
                }))),
                send_type: SendType::Single {
                    line_num: 3,
                    col_num: 9,
                },
                inputs: ProcList {
                    procs: vec![Proc::StringLiteral {
                        value: "World".to_string(),
                        line_num: 3,
                        col_num: 12,
                    }],
                    line_num: 3,
                    col_num: 11,
                },
                line_num: 3,
                col_num: 9,
            },
            line_num: 2,
            col_num: 16,
        }))),
        line_num: 2,
        col_num: 7,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_attenuation_example() {
    let input_code = r#"
     new MakeGetForwarder in {
       contract MakeGetForwarder(target, ret) = {
         new port in {
           ret!(*port) |
           contract port(@method, @arg, ack) = {
             match method == "get" { true => target!("get", arg, *ack) }
           }
         }
       }
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![NameDecl {
                var: Var {
                    name: "MakeGetForwarder".to_string(),
                    line_num: 1,
                    col_num: 9,
                },
                uri: None,
                line_num: 1,
                col_num: 9,
            }],
            line_num: 1,
            col_num: 9,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Contract {
                name: Name::ProcVar(Box::new(Proc::Var(Var {
                    name: "MakeGetForwarder".to_string(),
                    line_num: 2,
                    col_num: 16,
                }))),
                formals: Names {
                    names: vec![
                        Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "target".to_string(),
                            line_num: 2,
                            col_num: 33,
                        }))),
                        Name::ProcVar(Box::new(Proc::Var(Var {
                            name: "ret".to_string(),
                            line_num: 2,
                            col_num: 41,
                        }))),
                    ],
                    cont: None,
                    line_num: 2,
                    col_num: 33,
                },
                proc: Box::new(Block {
                    proc: Proc::New {
                        decls: Decls {
                            decls: vec![NameDecl {
                                var: Var {
                                    name: "port".to_string(),
                                    line_num: 3,
                                    col_num: 13,
                                },
                                uri: None,
                                line_num: 3,
                                col_num: 13,
                            }],
                            line_num: 3,
                            col_num: 13,
                        },
                        proc: Box::new(Proc::Block(Box::new(Block {
                            proc: Proc::Par {
                                left: Box::new(Proc::Send {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "ret".to_string(),
                                        line_num: 4,
                                        col_num: 11,
                                    }))),
                                    send_type: SendType::Single {
                                        line_num: 4,
                                        col_num: 11,
                                    },
                                    inputs: ProcList {
                                        procs: vec![Proc::Eval(Eval {
                                            name: Name::ProcVar(Box::new(Proc::Var(Var {
                                                name: "port".to_string(),
                                                line_num: 4,
                                                col_num: 17,
                                            }))),
                                            line_num: 4,
                                            col_num: 16,
                                        })],
                                        line_num: 4,
                                        col_num: 15,
                                    },
                                    line_num: 4,
                                    col_num: 11,
                                }),
                                right: Box::new(Proc::Contract {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "port".to_string(),
                                        line_num: 5,
                                        col_num: 20,
                                    }))),
                                    formals: Names {
                                        names: vec![
                                            Name::Quote(Box::new(Quote {
                                                quotable: Box::new(Proc::Var(Var {
                                                    name: "method".to_string(),
                                                    line_num: 5,
                                                    col_num: 26,
                                                })),
                                                line_num: 5,
                                                col_num: 25,
                                            })),
                                            Name::Quote(Box::new(Quote {
                                                quotable: Box::new(Proc::Var(Var {
                                                    name: "arg".to_string(),
                                                    line_num: 5,
                                                    col_num: 35,
                                                })),
                                                line_num: 5,
                                                col_num: 34,
                                            })),
                                            Name::ProcVar(Box::new(Proc::Var(Var {
                                                name: "ack".to_string(),
                                                line_num: 5,
                                                col_num: 40,
                                            }))),
                                        ],
                                        cont: None,
                                        line_num: 5,
                                        col_num: 25,
                                    },
                                    proc: Box::new(Block {
                                        proc: Proc::Match {
                                            expression: Box::new(Proc::Eq {
                                                left: Box::new(Proc::Var(Var {
                                                    name: "method".to_string(),
                                                    line_num: 6,
                                                    col_num: 19,
                                                })),
                                                right: Box::new(Proc::StringLiteral {
                                                    value: "get".to_string(),
                                                    line_num: 6,
                                                    col_num: 29,
                                                }),
                                                line_num: 6,
                                                col_num: 19,
                                            }),
                                            cases: vec![Case {
                                                pattern: Proc::BoolLiteral {
                                                    value: true,
                                                    line_num: 6,
                                                    col_num: 37,
                                                },
                                                proc: Proc::Send {
                                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                                        name: "target".to_string(),
                                                        line_num: 6,
                                                        col_num: 45,
                                                    }))),
                                                    send_type: SendType::Single {
                                                        line_num: 6,
                                                        col_num: 45,
                                                    },
                                                    inputs: ProcList {
                                                        procs: vec![
                                                            Proc::StringLiteral {
                                                                value: "get".to_string(),
                                                                line_num: 6,
                                                                col_num: 53,
                                                            },
                                                            Proc::Var(Var {
                                                                name: "arg".to_string(),
                                                                line_num: 6,
                                                                col_num: 60,
                                                            }),
                                                            Proc::Eval(Eval {
                                                                name: Name::ProcVar(Box::new(
                                                                    Proc::Var(Var {
                                                                        name: "ack".to_string(),
                                                                        line_num: 6,
                                                                        col_num: 66,
                                                                    }),
                                                                )),
                                                                line_num: 6,
                                                                col_num: 65,
                                                            }),
                                                        ],
                                                        line_num: 6,
                                                        col_num: 52,
                                                    },
                                                    line_num: 6,
                                                    col_num: 45,
                                                },
                                                line_num: 6,
                                                col_num: 37,
                                            }],
                                            line_num: 6,
                                            col_num: 13,
                                        },
                                        line_num: 5,
                                        col_num: 47,
                                    }),
                                    line_num: 5,
                                    col_num: 11,
                                }),
                                line_num: 4,
                                col_num: 11,
                            },
                            line_num: 3,
                            col_num: 21,
                        }))),
                        line_num: 3,
                        col_num: 9,
                    },
                    line_num: 2,
                    col_num: 48,
                }),
                line_num: 2,
                col_num: 7,
            },
            line_num: 1,
            col_num: 29,
        }))),
        line_num: 1,
        col_num: 5,
    };

    assert_eq!(result.unwrap(), expected_result)
}

#[test]
fn parse_rholang_code_to_proc_should_parse_dining_philosophers_example() {
    let input_code = r#"
     new philosopher1, philosopher2, north, south, knife, spoon in {
         north!(*knife) |
         south!(*spoon) |
         for (@knf <- north; @spn <- south) {
           philosopher1!("Complete!") |
           north!(knf) |
           south!(spn)
         } |
         for (@spn <- south; @knf <- north) {
           philosopher2!("Complete!") |
           north!(knf) |
           south!(spn)
         }
     }
   "#;

    let result = parse_rholang_code_to_proc(&input_code);
    assert!(result.is_ok());

    let expected_result = Proc::New {
        decls: Decls {
            decls: vec![
                NameDecl {
                    var: Var {
                        name: "philosopher1".to_string(),
                        line_num: 1,
                        col_num: 9,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 9,
                },
                NameDecl {
                    var: Var {
                        name: "philosopher2".to_string(),
                        line_num: 1,
                        col_num: 23,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 23,
                },
                NameDecl {
                    var: Var {
                        name: "north".to_string(),
                        line_num: 1,
                        col_num: 37,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 37,
                },
                NameDecl {
                    var: Var {
                        name: "south".to_string(),
                        line_num: 1,
                        col_num: 44,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 44,
                },
                NameDecl {
                    var: Var {
                        name: "knife".to_string(),
                        line_num: 1,
                        col_num: 51,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 51,
                },
                NameDecl {
                    var: Var {
                        name: "spoon".to_string(),
                        line_num: 1,
                        col_num: 58,
                    },
                    uri: None,
                    line_num: 1,
                    col_num: 58,
                },
            ],
            line_num: 1,
            col_num: 9,
        },
        proc: Box::new(Proc::Block(Box::new(Block {
            proc: Proc::Par {
                left: Box::new(Proc::Par {
                    left: Box::new(Proc::Par {
                        left: Box::new(Proc::Send {
                            name: Name::ProcVar(Box::new(Proc::Var(Var {
                                name: "north".to_string(),
                                line_num: 2,
                                col_num: 9,
                            }))),
                            send_type: SendType::Single {
                                line_num: 2,
                                col_num: 9,
                            },
                            inputs: ProcList {
                                procs: vec![Proc::Eval(Eval {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "knife".to_string(),
                                        line_num: 2,
                                        col_num: 17,
                                    }))),
                                    line_num: 2,
                                    col_num: 16,
                                })],
                                line_num: 2,
                                col_num: 15,
                            },
                            line_num: 2,
                            col_num: 9,
                        }),
                        right: Box::new(Proc::Send {
                            name: Name::ProcVar(Box::new(Proc::Var(Var {
                                name: "south".to_string(),
                                line_num: 3,
                                col_num: 9,
                            }))),
                            send_type: SendType::Single {
                                line_num: 3,
                                col_num: 9,
                            },
                            inputs: ProcList {
                                procs: vec![Proc::Eval(Eval {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "spoon".to_string(),
                                        line_num: 3,
                                        col_num: 17,
                                    }))),
                                    line_num: 3,
                                    col_num: 16,
                                })],
                                line_num: 3,
                                col_num: 15,
                            },
                            line_num: 3,
                            col_num: 9,
                        }),
                        line_num: 2,
                        col_num: 9,
                    }),
                    right: Box::new(Proc::Input {
                        formals: Receipts {
                            receipts: vec![
                                Receipt::LinearBinds(LinearBind {
                                    names: Names {
                                        names: vec![Name::Quote(Box::new(Quote {
                                            quotable: Box::new(Proc::Var(Var {
                                                name: "knf".to_string(),
                                                line_num: 4,
                                                col_num: 15,
                                            })),
                                            line_num: 4,
                                            col_num: 14,
                                        }))],
                                        cont: None,
                                        line_num: 4,
                                        col_num: 14,
                                    },
                                    input: Source::Simple {
                                        name: Name::ProcVar(Box::new(Proc::Var(Var {
                                            name: "north".to_string(),
                                            line_num: 4,
                                            col_num: 22,
                                        }))),
                                        line_num: 4,
                                        col_num: 22,
                                    },
                                    line_num: 4,
                                    col_num: 14,
                                }),
                                Receipt::LinearBinds(LinearBind {
                                    names: Names {
                                        names: vec![Name::Quote(Box::new(Quote {
                                            quotable: Box::new(Proc::Var(Var {
                                                name: "spn".to_string(),
                                                line_num: 4,
                                                col_num: 30,
                                            })),
                                            line_num: 4,
                                            col_num: 29,
                                        }))],
                                        cont: None,
                                        line_num: 4,
                                        col_num: 29,
                                    },
                                    input: Source::Simple {
                                        name: Name::ProcVar(Box::new(Proc::Var(Var {
                                            name: "south".to_string(),
                                            line_num: 4,
                                            col_num: 37,
                                        }))),
                                        line_num: 4,
                                        col_num: 37,
                                    },
                                    line_num: 4,
                                    col_num: 29,
                                }),
                            ],
                            line_num: 4,
                            col_num: 14,
                        },
                        proc: Box::new(Block {
                            proc: Proc::Par {
                                left: Box::new(Proc::Par {
                                    left: Box::new(Proc::Send {
                                        name: Name::ProcVar(Box::new(Proc::Var(Var {
                                            name: "philosopher1".to_string(),
                                            line_num: 5,
                                            col_num: 11,
                                        }))),
                                        send_type: SendType::Single {
                                            line_num: 5,
                                            col_num: 11,
                                        },
                                        inputs: ProcList {
                                            procs: vec![Proc::StringLiteral {
                                                value: "Complete!".to_string(),
                                                line_num: 5,
                                                col_num: 25,
                                            }],
                                            line_num: 5,
                                            col_num: 24,
                                        },
                                        line_num: 5,
                                        col_num: 11,
                                    }),
                                    right: Box::new(Proc::Send {
                                        name: Name::ProcVar(Box::new(Proc::Var(Var {
                                            name: "north".to_string(),
                                            line_num: 6,
                                            col_num: 11,
                                        }))),
                                        send_type: SendType::Single {
                                            line_num: 6,
                                            col_num: 11,
                                        },
                                        inputs: ProcList {
                                            procs: vec![Proc::Var(Var {
                                                name: "knf".to_string(),
                                                line_num: 6,
                                                col_num: 18,
                                            })],
                                            line_num: 6,
                                            col_num: 17,
                                        },
                                        line_num: 6,
                                        col_num: 11,
                                    }),
                                    line_num: 5,
                                    col_num: 11,
                                }),
                                right: Box::new(Proc::Send {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "south".to_string(),
                                        line_num: 7,
                                        col_num: 11,
                                    }))),
                                    send_type: SendType::Single {
                                        line_num: 7,
                                        col_num: 11,
                                    },
                                    inputs: ProcList {
                                        procs: vec![Proc::Var(Var {
                                            name: "spn".to_string(),
                                            line_num: 7,
                                            col_num: 18,
                                        })],
                                        line_num: 7,
                                        col_num: 17,
                                    },
                                    line_num: 7,
                                    col_num: 11,
                                }),
                                line_num: 5,
                                col_num: 11,
                            },
                            line_num: 4,
                            col_num: 44,
                        }),
                        line_num: 4,
                        col_num: 9,
                    }),
                    line_num: 2,
                    col_num: 9,
                }),
                right: Box::new(Proc::Input {
                    formals: Receipts {
                        receipts: vec![
                            Receipt::LinearBinds(LinearBind {
                                names: Names {
                                    names: vec![Name::Quote(Box::new(Quote {
                                        quotable: Box::new(Proc::Var(Var {
                                            name: "spn".to_string(),
                                            line_num: 9,
                                            col_num: 15,
                                        })),
                                        line_num: 9,
                                        col_num: 14,
                                    }))],
                                    cont: None,
                                    line_num: 9,
                                    col_num: 14,
                                },
                                input: Source::Simple {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "south".to_string(),
                                        line_num: 9,
                                        col_num: 22,
                                    }))),
                                    line_num: 9,
                                    col_num: 22,
                                },
                                line_num: 9,
                                col_num: 14,
                            }),
                            Receipt::LinearBinds(LinearBind {
                                names: Names {
                                    names: vec![Name::Quote(Box::new(Quote {
                                        quotable: Box::new(Proc::Var(Var {
                                            name: "knf".to_string(),
                                            line_num: 9,
                                            col_num: 30,
                                        })),
                                        line_num: 9,
                                        col_num: 29,
                                    }))],
                                    cont: None,
                                    line_num: 9,
                                    col_num: 29,
                                },
                                input: Source::Simple {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "north".to_string(),
                                        line_num: 9,
                                        col_num: 37,
                                    }))),
                                    line_num: 9,
                                    col_num: 37,
                                },
                                line_num: 9,
                                col_num: 29,
                            }),
                        ],
                        line_num: 9,
                        col_num: 14,
                    },
                    proc: Box::new(Block {
                        proc: Proc::Par {
                            left: Box::new(Proc::Par {
                                left: Box::new(Proc::Send {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "philosopher2".to_string(),
                                        line_num: 10,
                                        col_num: 11,
                                    }))),
                                    send_type: SendType::Single {
                                        line_num: 10,
                                        col_num: 11,
                                    },
                                    inputs: ProcList {
                                        procs: vec![Proc::StringLiteral {
                                            value: "Complete!".to_string(),
                                            line_num: 10,
                                            col_num: 25,
                                        }],
                                        line_num: 10,
                                        col_num: 24,
                                    },
                                    line_num: 10,
                                    col_num: 11,
                                }),
                                right: Box::new(Proc::Send {
                                    name: Name::ProcVar(Box::new(Proc::Var(Var {
                                        name: "north".to_string(),
                                        line_num: 11,
                                        col_num: 11,
                                    }))),
                                    send_type: SendType::Single {
                                        line_num: 11,
                                        col_num: 11,
                                    },
                                    inputs: ProcList {
                                        procs: vec![Proc::Var(Var {
                                            name: "knf".to_string(),
                                            line_num: 11,
                                            col_num: 18,
                                        })],
                                        line_num: 11,
                                        col_num: 17,
                                    },
                                    line_num: 11,
                                    col_num: 11,
                                }),
                                line_num: 10,
                                col_num: 11,
                            }),
                            right: Box::new(Proc::Send {
                                name: Name::ProcVar(Box::new(Proc::Var(Var {
                                    name: "south".to_string(),
                                    line_num: 12,
                                    col_num: 11,
                                }))),
                                send_type: SendType::Single {
                                    line_num: 12,
                                    col_num: 11,
                                },
                                inputs: ProcList {
                                    procs: vec![Proc::Var(Var {
                                        name: "spn".to_string(),
                                        line_num: 12,
                                        col_num: 18,
                                    })],
                                    line_num: 12,
                                    col_num: 17,
                                },
                                line_num: 12,
                                col_num: 11,
                            }),
                            line_num: 10,
                            col_num: 11,
                        },
                        line_num: 9,
                        col_num: 44,
                    }),
                    line_num: 9,
                    col_num: 9,
                }),
                line_num: 2,
                col_num: 9,
            },
            line_num: 1,
            col_num: 67,
        }))),
        line_num: 1,
        col_num: 5,
    };

    assert_eq!(result.unwrap(), expected_result)
}

// This test is commented out because it needs this env parameter to test `RUST_MIN_STACK=536870912`
// #[test]
// fn parse_rholang_code_to_proc_should_parse_registry_rho_file_contents() {
//     let input_code = r#"
//   new _registryStore,
//     lookupCh,
//     bootstrapLookup(`rho:registry:lookup`),
//     insertArbitraryCh,
//     bootstrapInsertArbitrary(`rho:registry:insertArbitrary`),
//     insertSignedCh,
//     bootstrapInsertSigned(`rho:registry:insertSigned:secp256k1`),
//     buildUri,
//     ops(`rho:registry:ops`),
//     secpVerify(`rho:crypto:secp256k1Verify`),
//     blake2b256(`rho:crypto:blake2b256Hash`),
//     TreeHashMap
// in {

//   // TreeHashMap is defined here because the registry uses it internally.
//   // We can't define it in another file like MakeMint or NonNegativeNumber because
//   // that method assumes the registry already exists.

//   // Rholang map desiderata: speedy insert & lookup,  no conflicts on lookup, no conflicts on inserts to different keys
//   // This implementation: O(log n) insert & lookup; also provides O(1) lookup when it is known that the value at a key exists.
//   // Conflict analysis
//   //   Lookup
//   //     When looking up a value, only peeks are used, so lookups will not conflict.
//   //   Insert
//   //     When inserting, only peeks are used on existing nodes (except the last
//   //     shared one in the path), while newly created nodes have a different name.
//   //     So there's conflict only if the keys share a common prefix that hadn't
//   //     already been populated.
//   // Usage
//   // new o(`rho:io:stdout`), mapCh in {
//   //   o!("Initializing map") |
//   //   // Use 3 * 8 = 24 bits of parallelization
//   //   TreeHashMap!("init", 3, *mapCh) |
//   //   for (@map <- mapCh) {
//   //     o!("Map initialized, setting") |
//   //     new ret1, ret2, ret3, ret4, ret5, ret6, ret7 in {
//   //       TreeHashMap!("set", map, "some key", "some val", *ret1) |
//   //       TreeHashMap!("set", map, "monkey", "some other val", *ret2) |
//   //       TreeHashMap!("set", map, "donkey", Nil, *ret3) |
//   //       for (_ <- ret1 & _ <- ret2 & _ <- ret3) {
//   //         o!("Value set, getting") |
//   //         TreeHashMap!("get", map, "some key", *ret1) |             // "some val"
//   //         TreeHashMap!("fastUnsafeGet", map, "monkey", *ret2) |     // "some other val"
//   //         TreeHashMap!("get", map, "some unused key", *ret3) |      // Nil
//   //         TreeHashMap!("fastUnsafeGet", map, "donkey", *ret4) |     // Nil
//   //         TreeHashMap!("contains", map, "donkey", *ret5) |          // true
//   //         TreeHashMap!("contains", map, "monkey", *ret6) |          // true
//   //         TreeHashMap!("contains", map, "some unused key", *ret7) | // false
//   //         for (@val1 <- ret1 & @val2 <- ret2 & @val3 <- ret3 & @val4 <- ret4 & @val5 <- ret5 & @val6 <- ret6 & @val7 <- ret7) {
//   //           o!(["Got these from the map: ", val1, val2, val3, val4, val5, val6, val7])
//   //         }
//   //       }
//   //     }
//   //   }
//   // }

//   new MakeNode, ByteArrayToNybbleList,
//       TreeHashMapSetter, TreeHashMapGetter, TreeHashMapContains, TreeHashMapUpdater,
//       powersCh, storeToken, nodeGet in {
//     match [1,2,4,8,16,32,64,128,256,512,1024,2048,4096,8192,16384,32768,65536] {
//       powers => {
//         contract MakeNode(@initVal, @node) = {
//           @[node, *storeToken]!(initVal)
//         } |

//         contract nodeGet(@node, ret) = {
//           for (@val <<- @[node, *storeToken]) {
//             ret!(val)
//           }
//         } |

//         contract ByteArrayToNybbleList(@ba, @n, @len, @acc, ret) = {
//           if (n == len) {
//             ret!(acc)
//           } else {
//             ByteArrayToNybbleList!(ba, n+1, len, acc ++ [ ba.nth(n) % 16, ba.nth(n) / 16 ], *ret)
//           }
//         } |

//         contract TreeHashMap(@"init", @depth, ret) = {
//           new map in {
//             MakeNode!(0, (*map, [])) |
//             @(*map, "depth")!!(depth) |
//             ret!(*map)
//           }
//         } |

//         contract TreeHashMapGetter(@map, @nybList, @n, @len, @suffix, ret) = {
//           // Look up the value of the node at (map, nybList.slice(0, n + 1))
//           new valCh in {
//             nodeGet!((map, nybList.slice(0, n)), *valCh) |
//             for (@val <- valCh) {
//               if (n == len) {
//                 ret!(val.get(suffix))
//               } else {
//                 // Otherwise check if the rest of the path exists.
//                 // Bit k set means node k exists.
//                 // nybList.nth(n) is the node number
//                 // val & powers.nth(nybList.nth(n)) is nonzero if the node exists
//                 // (val / powers.nth(nybList.nth(n))) % 2 is 1 if the node exists
//                 if ((val / powers.nth(nybList.nth(n))) % 2 == 0) {
//                   ret!(Nil)
//                 } else {
//                   TreeHashMapGetter!(map, nybList, n + 1, len, suffix, *ret)
//                 }
//               }
//             }
//           }
//         } |

//         contract TreeHashMap(@"get", @map, @key, ret) = {
//           new hashCh, nybListCh, keccak256Hash(`rho:crypto:keccak256Hash`) in {
//             // Hash the key to get a 256-bit array
//             keccak256Hash!(key.toByteArray(), *hashCh) |
//             for (@hash <- hashCh) {
//               for (@depth <<- @(map, "depth")) {
//                 // Get the bit list
//                 ByteArrayToNybbleList!(hash, 0, depth, [], *nybListCh) |
//                 for (@nybList <- nybListCh) {
//                   TreeHashMapGetter!(map, nybList, 0, 2 * depth, hash.slice(depth, 32), *ret)
//                 }
//               }
//             }
//           }
//         } |

//         // Doesn't walk the path, just tries to fetch it directly.
//         // Will hang if there's no key with that 64-bit prefix.
//         // Returns Nil like "get" does if there is some other key with
//         // the same prefix but no value there.
//         contract TreeHashMap(@"fastUnsafeGet", @map, @key, ret) = {
//           new hashCh, nybListCh, keccak256Hash(`rho:crypto:keccak256Hash`) in {
//             // Hash the key to get a 256-bit array
//             keccak256Hash!(key.toByteArray(), *hashCh) |
//             for (@hash <- hashCh) {
//               for(@depth <<- @(map, "depth")) {
//                 // Get the bit list
//                 ByteArrayToNybbleList!(hash, 0, depth, [], *nybListCh) |
//                 for (@nybList <- nybListCh) {
//                   new restCh, valCh in {
//                     nodeGet!((map, nybList), *restCh) |
//                     for (@rest <- restCh) {
//                       ret!(rest.get(hash.slice(depth, 32)))
//                     }
//                   }
//                 }
//               }
//             }
//           }
//         } |

//         contract TreeHashMapSetter(@map, @nybList, @n, @len, @newVal, @suffix, ret) = {
//           // Look up the value of the node at (map, nybList.slice(0, n + 1))
//           new valCh, restCh in {
//             match (map, nybList.slice(0, n)) {
//               node => {
//                 for (@val <<- @[node, *storeToken]) {
//                   if (n == len) {
//                     // Acquire the lock on this node
//                     for (@val <- @[node, *storeToken]) {
//                       // If we're at the end of the path, set the node to newVal.
//                       if (val == 0) {
//                         // Release the lock
//                         @[node, *storeToken]!({suffix: newVal}) |
//                         // Return
//                         ret!(Nil)
//                       }
//                       else {
//                         // Release the lock
//                         @[node, *storeToken]!(val.set(suffix, newVal)) |
//                         // Return
//                         ret!(Nil)
//                       }
//                     }
//                   } else {
//                     // Otherwise make the rest of the path exist.
//                     // Bit k set means child node k exists.
//                     if ((val/powers.nth(nybList.nth(n))) % 2 == 0) {
//                       // Child node missing
//                       // Acquire the lock
//                       for (@val <- @[node, *storeToken]) {
//                         // Re-test value
//                         if ((val/powers.nth(nybList.nth(n))) % 2 == 0) {
//                           // Child node still missing
//                           // Create node, set node to 0
//                           MakeNode!(0, (map, nybList.slice(0, n + 1))) |
//                           // Update current node to val | (1 << nybList.nth(n))
//                           match nybList.nth(n) {
//                             bit => {
//                               // val | (1 << bit)
//                               // Bitwise operators would be really nice to have!
//                               // Release the lock
//                               @[node, *storeToken]!((val % powers.nth(bit)) +
//                                 (val / powers.nth(bit + 1)) * powers.nth(bit + 1) +
//                                 powers.nth(bit))
//                             }
//                           } |
//                           // Child node now exists, loop
//                           TreeHashMapSetter!(map, nybList, n + 1, len, newVal, suffix, *ret)
//                         } else {
//                           // Child node created between reads
//                           // Release lock
//                           @[node, *storeToken]!(val) |
//                           // Loop
//                           TreeHashMapSetter!(map, nybList, n + 1, len, newVal, suffix, *ret)
//                         }
//                       }
//                     } else {
//                       // Child node exists, loop
//                       TreeHashMapSetter!(map, nybList, n + 1, len, newVal, suffix, *ret)
//                     }
//                   }
//                 }
//               }
//             }
//           }
//         } |

//         contract TreeHashMap(@"set", @map, @key, @newVal, ret) = {
//           new hashCh, nybListCh, keccak256Hash(`rho:crypto:keccak256Hash`) in {
//             // Hash the key to get a 256-bit array
//             keccak256Hash!(key.toByteArray(), *hashCh) |
//             for (@hash <- hashCh) {
//               for (@depth <<- @(map, "depth")) {
//                 // Get the bit list
//                 ByteArrayToNybbleList!(hash, 0, depth, [], *nybListCh) |
//                 for (@nybList <- nybListCh) {
//                   TreeHashMapSetter!(map, nybList, 0, 2 * depth, newVal, hash.slice(depth, 32), *ret)
//                 }
//               }
//             }
//           }
//         } |

//         contract TreeHashMapContains(@map, @nybList, @n, @len, @suffix, ret) = {
//           // Look up the value of the node at [map, nybList.slice(0, n + 1)]
//           new valCh in {
//             nodeGet!((map, nybList.slice(0, n)), *valCh) |
//             for (@val <- valCh) {
//               if (n == len) {
//                 ret!(val.contains(suffix))
//               } else {
//                 // See getter for explanation of formula
//                 if ((val/powers.nth(nybList.nth(n))) % 2 == 0) {
//                   ret!(false)
//                 } else {
//                   TreeHashMapContains!(map, nybList, n + 1, len, suffix, *ret)
//                 }
//               }
//             }
//           }
//         } |

//         contract TreeHashMap(@"contains", @map, @key, ret) = {
//           new hashCh, nybListCh, keccak256Hash(`rho:crypto:keccak256Hash`) in {
//             // Hash the key to get a 256-bit array
//             keccak256Hash!(key.toByteArray(), *hashCh) |
//             for (@hash <- hashCh) {
//               for (@depth <<- @(map, "depth")) {
//                 // Get the bit list
//                 ByteArrayToNybbleList!(hash, 0, depth, [], *nybListCh) |
//                 for (@nybList <- nybListCh) {
//                   TreeHashMapContains!(map, nybList, 0, 2 * depth, hash.slice(depth, 32), *ret)
//                 }
//               }
//             }
//           }
//         } |

//         contract TreeHashMapUpdater(@map, @nybList, @n, @len, update, @suffix, ret) = {
//           // Look up the value of the node at [map, nybList.slice(0, n + 1)
//           new valCh in {
//             match (map, nybList.slice(0, n)) {
//               node => {
//                 for (@val <<- @[node, *storeToken]) {
//                   if (n == len) {
//                     // We're at the end of the path.
//                     if (val == 0) {
//                       // There's nothing here.
//                       // Return
//                       ret!(Nil)
//                     } else {
//                       new resultCh in {
//                         // Acquire the lock on this node
//                         for (@val <- @[node, *storeToken]) {
//                           // Update the current value
//                           update!(val.get(suffix), *resultCh) |
//                           for (@newVal <- resultCh) {
//                             // Release the lock
//                             @[node, *storeToken]!(val.set(suffix, newVal)) |
//                             // Return
//                             ret!(Nil)
//                           }
//                         }
//                       }
//                     }
//                   } else {
//                     // Otherwise try to reach the end of the path.
//                     // Bit k set means child node k exists.
//                     if ((val/powers.nth(nybList.nth(n))) % 2 == 0) {
//                       // If the path doesn't exist, there's no value to update.
//                       // Return
//                       ret!(Nil)
//                     } else {
//                       // Child node exists, loop
//                       TreeHashMapUpdater!(map, nybList, n + 1, len, *update, suffix, *ret)
//                     }
//                   }
//                 }
//               }
//             }
//           }
//         } |
//         contract TreeHashMap(@"update", @map, @key, update, ret) = {
//           new hashCh, nybListCh, keccak256Hash(`rho:crypto:keccak256Hash`) in {
//             // Hash the key to get a 256-bit array
//             keccak256Hash!(key.toByteArray(), *hashCh) |
//             for (@hash <- hashCh) {
//               for (@depth <<- @(map, "depth")) {
//                 // Get the bit list
//                 ByteArrayToNybbleList!(hash, 0, depth, [], *nybListCh) |
//                 for (@nybList <- nybListCh) {
//                   TreeHashMapUpdater!(map, nybList, 0, 2 * depth, *update, hash.slice(depth, 32), *ret)
//                 }
//               }
//             }
//           }
//         }
//       }
//     }
//   } |

//   // Use 4 * 8 = 32-bit paths to leaf nodes.
//   TreeHashMap!("init", 4, *_registryStore) |
//   new ack in {
//     for (@map <<- _registryStore) {
//       TreeHashMap!("set", map, `rho:lang:treeHashMap`, bundle+{*TreeHashMap}, *ack)
//     } |

//     for (_ <- ack) {
//       bootstrapLookup!(*lookupCh) | // this will work only once
//       for (lookup <- lookupCh) {
//         contract lookup(@uriOrShorthand, ret) = {
//           match {
//             `rho:lang:either` : `rho:id:qrh6mgfp5z6orgchgszyxnuonanz7hw3amgrprqtciia6astt66ypn`,
//             `rho:lang:listOps` : `rho:id:6fzorimqngeedepkrizgiqms6zjt76zjeciktt1eifequy4osz35ks`,
//             `rho:lang:nonNegativeNumber` : `rho:id:hxyadh1ffypra47ry9mk6b8r1i33ar1w9wjsez4khfe9huzrfcytx9`,
//             `rho:rchain:authKey` : `rho:id:1qw5ehmq1x49dey4eadr1h4ncm361w3536asho7dr38iyookwcsp6i`,
//             `rho:rchain:makeMint` : `rho:id:asysrwfgzf8bf7sxkiowp4b3tcsy4f8ombi3w96ysox4u3qdmn1wbc`,
//             `rho:rchain:pos` : `rho:id:m3xk7h8r54dtqtwsrnxqzhe81baswey66nzw6m533nyd45ptyoybqr`,
//             `rho:rchain:revVault` : `rho:id:6zcfqnwnaqcwpeyuysx1rm48ndr6sgsbbgjuwf45i5nor3io7dr76j`,
//             `rho:rchain:multiSigRevVault` : `rho:id:b9s6j3xeobgset4ndn64hje64grfcj7a43eekb3fh43yso5ujiecfn`
//           } {
//             shorthands => {
//               for (@map <<- _registryStore) {
//                 TreeHashMap!("get", map, shorthands.getOrElse(uriOrShorthand, uriOrShorthand), *ret)
//               }
//             }
//           }
//         }
//       }
//     }
//   } |

//   bootstrapInsertArbitrary!(*insertArbitraryCh) | // this will work only once
//   for (insertArbitrary <- insertArbitraryCh) {
//     contract insertArbitrary(@data, ret) = {

//       new seed, uriCh in {
//         ops!("buildUri", *seed.toByteArray(), *uriCh) |
//         for (@uri <- uriCh) {
//           for (@map <<- _registryStore) {
//             new ack in {
//               TreeHashMap!("set", map, uri, data, *ack) |
//               for (_ <- ack) {
//                 ret!(uri)
//               }
//             }
//           }
//         }
//       }
//     }
//   } |

//   bootstrapInsertSigned!(*insertSignedCh) | // this will work only once
//   for (insertSigned <- insertSignedCh) {

//     contract insertSigned(@pubKeyBytes, @value, @sig, ret) = {
//       match value {
//         (nonce, data) => {
//           new uriCh, hashCh, verifyCh in {
//             blake2b256!((nonce, data).toByteArray(), *hashCh) |
//             for (@hash <- hashCh) {
//               secpVerify!(hash, sig, pubKeyBytes, *verifyCh) |
//               for (@verified <- verifyCh) {
//                 if (verified) {
//                   ops!("buildUri", pubKeyBytes, *uriCh) |
//                   for (@uri <- uriCh) {
//                     for (@map <<- _registryStore) {
//                       new responseCh in {
//                         TreeHashMap!("get", map, uri, *responseCh) |
//                         for (@response <- responseCh) {
//                           match response {
//                             Nil => {
//                               new ack in {
//                                 TreeHashMap!("set", map, uri, (nonce, data), *ack) |
//                                 for (_ <- ack) {
//                                   ret!(uri)
//                                 }
//                               }
//                             }
//                             (oldNonce, _) => {
//                               if (nonce > oldNonce) {
//                                 new ack in {
//                                   TreeHashMap!("set", map, uri, (nonce, data), *ack) |
//                                   for (_ <- ack) {
//                                     ret!(uri)
//                                   }
//                                 }
//                               } else {
//                                 ret!(Nil)
//                               }
//                             }
//                           }
//                         }
//                       }
//                     }
//                   }
//                 } else {
//                   ret!(Nil)
//                 }
//               }
//             }
//           }
//         }
//       }
//     }
//   }
// }
//   "#;

//     let result = parse_rholang_code_to_proc(&input_code);
//     assert!(result.is_ok());
// }
