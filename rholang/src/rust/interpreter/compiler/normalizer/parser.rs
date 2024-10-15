use tree_sitter::{Node, Parser, Tree};

use crate::rust::interpreter::{
    compiler::rholang_ast::{
        Block, Collection, Conjunction, Disjunction, Eval, Ground, GroundExpression, Name,
        Negation, Proc, ProcList, ProcVar, Quotable, Quote, SimpleType, SyncSendCont, UriLiteral,
    },
    errors::InterpreterError,
};

pub fn parse_rholang_code(code: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rholang::LANGUAGE.into())
        .expect("Error loading Rholang grammar");
    println!("Language {:?}", parser.language());
    parser.parse(code, None).expect("Failed to parse code")
}

pub fn parse_rholang_code_to_proc(code: &str) -> Result<Proc, InterpreterError> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rholang::LANGUAGE.into())
        .expect("Error loading Rholang grammar");

    let tree = parser.parse(code, None).expect("Failed to parse code");
    // println!("{:#?}", tree.root_node().to_sexp());

    let root_node = tree.root_node();
    if root_node.kind() != "source_file" {
        return Err(InterpreterError::ParserError(
            "Incorrent root kind".to_string(),
        ));
    }

    let start_node = match root_node.child(0) {
        Some(node) => Ok(node),
        None => Err(InterpreterError::ParserError(
            "The code does not contain any valid Rholang process. Expected a child node"
                .to_string(),
        )),
    }?;

    parse_proc(&start_node, code);

    // loop {
    //     let node = cursor.node();
    //     let node_name = node.kind();
    //     let node_value = node.utf8_text(code.as_bytes()).unwrap_or("");

    //     println!("Node: {}, Value: {}", node_name, node_value);

    //     if cursor.goto_first_child() {
    //         continue;
    //     }

    //     if cursor.goto_next_sibling() {
    //         continue;
    //     }

    //     while !cursor.goto_next_sibling() {
    //         if !cursor.goto_parent() {
    //             return Err(InterpreterError::ParserError(
    //                 "Traversal completed".to_string(),
    //             ));
    //         }
    //     }
    // }

    todo!()
}

fn parse_proc(node: &Node, source: &str) -> Result<Proc, InterpreterError> {
    match node.kind() {
        "par" => {
            let left_node = get_child_by_field_name(node, "left")?;
            let left_proc = parse_proc(&left_node, source)?;

            let right_node = get_child_by_field_name(node, "right")?;
            let right_proc = parse_proc(&right_node, source)?;

            Ok(Proc::Par {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num: node.start_position().row,
                col_num: node.start_position().column,
            })
        }

        "send_sync" => {
            let name_node = get_child_by_field_name(node, "name")?;
            let name_proc = parse_name(&name_node, source)?;
            let messages_node = get_child_by_field_name(node, "messages")?;
            let messages_proc = parse_proc_list(&messages_node, source)?;

            let cont: SyncSendCont = {
                let cont_node = get_child_by_field_name(node, "cont")?;
                match cont_node.kind() {
                    "non_empty_cont" => SyncSendCont::NonEmpty {
                        proc: Box::new(parse_proc(&cont_node, source)?),
                        line_num: cont_node.start_position().row,
                        col_num: cont_node.start_position().column,
                    },

                    "empty_cont" => SyncSendCont::Empty {
                        line_num: cont_node.start_position().row,
                        col_num: cont_node.start_position().column,
                    },

                    _ => {
                        return Err(InterpreterError::ParserError(format!(
                            "Unexpected choice node kind: {:?} of sync_send_cont",
                            node.kind(),
                        )))
                    }
                }
            };

            Ok(Proc::SendSync {
                name: name_proc,
                messages: messages_proc,
                cont,
                line_num: node.start_position().row,
                col_num: node.start_position().column,
            })
        } // "new" => {
        //     cursor.goto_first_child();
        //     let decls = parse_decls(cursor, source);
        //     cursor.goto_next_sibling();
        //     let proc = parse_proc(cursor, source);
        //     cursor.goto_parent();
        //     Proc::New(decls, Box::new(proc))
        // }
        // "ifElse" => {
        //     cursor.goto_first_child();
        //     let condition = parse_proc(cursor, source);
        //     cursor.goto_next_sibling();
        //     let if_true = parse_proc(cursor, source);
        //     let alternative = if cursor.goto_next_sibling() {
        //         Some(Box::new(parse_proc(cursor, source)))
        //     } else {
        //         None
        //     };
        //     cursor.goto_parent();
        //     Proc::IfElse(Box::new(condition), Box::new(if_true), alternative)
        // }
        // "let" => {
        //     cursor.goto_first_child();
        //     let decls = parse_decls(cursor, source);
        //     cursor.goto_next_sibling();
        //     let body = parse_proc(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Let(decls, Box::new(body))
        // }
        // "bundle" => {
        //     cursor.goto_first_child();
        //     let bundle_type = node
        //         .child_by_field_name("bundle_type")
        //         .unwrap()
        //         .utf8_text(source.as_bytes())
        //         .unwrap()
        //         .to_string();
        //     cursor.goto_next_sibling();
        //     let proc = parse_proc(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Bundle(bundle_type, Box::new(proc))
        // }
        // "match" => {
        //     cursor.goto_first_child();
        //     let expression = parse_proc(cursor, source);
        //     cursor.goto_next_sibling();
        //     let cases = parse_cases(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Match(Box::new(expression), cases)
        // }
        // "choice" => {
        //     cursor.goto_first_child();
        //     let branches = parse_branches(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Choice(branches)
        // }
        // "contract" => {
        //     cursor.goto_first_child();
        //     let name = node
        //         .child_by_field_name("name")
        //         .unwrap()
        //         .utf8_text(source.as_bytes())
        //         .unwrap()
        //         .to_string();
        //     cursor.goto_next_sibling();
        //     let formals = parse_names(cursor, source);
        //     cursor.goto_next_sibling();
        //     let proc = parse_proc(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Contract(name, formals, Box::new(proc))
        // }
        // "input" => {
        //     cursor.goto_first_child();
        //     let formals = parse_names(cursor, source);
        //     cursor.goto_next_sibling();
        //     let proc = parse_proc(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Input(formals, Box::new(proc))
        // }
        // "send" => {
        //     cursor.goto_first_child();
        //     let name = node
        //         .child_by_field_name("name")
        //         .unwrap()
        //         .utf8_text(source.as_bytes())
        //         .unwrap()
        //         .to_string();
        //     cursor.goto_next_sibling();
        //     let inputs = parse_proc_list(cursor, source);
        //     cursor.goto_parent();
        //     Proc::Send(name, inputs)
        // }
        // "var_ref" => {
        //     let var = node
        //         .child_by_field_name("var")
        //         .unwrap()
        //         .utf8_text(source.as_bytes())
        //         .unwrap()
        //         .to_string();
        //     Proc::VarRef(var)
        // }
        // "nil" => Proc::Nil,
        _ => Err(InterpreterError::ParserError(
            "Unrecognizable process".to_string(),
        )),
    }
}

fn parse_name(node: &Node, source: &str) -> Result<Name, InterpreterError> {
    match node.kind() {
        "_proc_var" => Ok(Name::NameProcVar(parse_proc_var(node, source)?)),

        "quote" => Ok(Name::NameQuote(Box::new(parse_quote(node, source)?))),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of name",
            node.kind(),
        ))),
    }
}

fn parse_proc_var(node: &Node, source: &str) -> Result<ProcVar, InterpreterError> {
    match node.kind() {
        "wildcard" => Ok(ProcVar::Wildcard {
            line_num: node.start_position().row,
            col_num: node.start_position().column,
        }),

        "var" => Ok(ProcVar::Var {
            name: get_node_value(node, source.as_bytes())?,
            line_num: node.start_position().row,
            col_num: node.start_position().column,
        }),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of _proc_var",
            node.kind(),
        ))),
    }
}

fn parse_quote(node: &Node, source: &str) -> Result<Quote, InterpreterError> {
    let quotable_node = node
        .child(1)
        .filter(|n| n.kind() == "quotable")
        .ok_or_else(|| {
            InterpreterError::ParserError(
                "Expected a quotable node in quote at index 1".to_string(),
            )
        })?;

    Ok(Quote {
        quotable: parse_quotable(&quotable_node, source)?,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_quotable(node: &Node, source: &str) -> Result<Quotable, InterpreterError> {
    match node.kind() {
        "eval" => Ok(Quotable::Eval(parse_eval(node, source)?)),

        "disjunction" => Ok(Quotable::Disjunction(parse_disjunction(node, source)?)),

        "conjunction" => Ok(Quotable::Conjunction(parse_conjuction(node, source)?)),

        "negation" => Ok(Quotable::Negation(parse_negation(node, source)?)),

        "_ground_expression" => Ok(Quotable::GroundExpression(parse_ground_expression(
            node, source,
        )?)),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of quotable",
            node.kind(),
        ))),
    }
}

fn parse_eval(node: &Node, source: &str) -> Result<Eval, InterpreterError> {
    let name_node = node
        .child(1)
        .filter(|n| n.kind() == "name")
        .ok_or_else(|| {
            InterpreterError::ParserError("Expected a name node in eval at index 1".to_string())
        })?;

    Ok(Eval {
        name: parse_name(&name_node, source)?,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_disjunction(node: &Node, source: &str) -> Result<Disjunction, InterpreterError> {
    let left_node = get_child_by_field_name(node, "left")?;
    let left_proc = parse_proc(&left_node, source)?;

    let right_node = get_child_by_field_name(node, "right")?;
    let right_proc = parse_proc(&right_node, source)?;

    Ok(Disjunction {
        left: left_proc,
        right: right_proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_conjuction(node: &Node, source: &str) -> Result<Conjunction, InterpreterError> {
    let left_node = get_child_by_field_name(node, "left")?;
    let left_proc = parse_proc(&left_node, source)?;

    let right_node = get_child_by_field_name(node, "right")?;
    let right_proc = parse_proc(&right_node, source)?;

    Ok(Conjunction {
        left: left_proc,
        right: right_proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_negation(node: &Node, source: &str) -> Result<Negation, InterpreterError> {
    let proc_node = get_child_by_field_name(node, "proc")?;
    let proc = parse_proc(&proc_node, source)?;

    Ok(Negation {
        proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_ground_expression(
    node: &Node,
    source: &str,
) -> Result<GroundExpression, InterpreterError> {
    match node.kind() {
        "block" => Ok(GroundExpression::Block(parse_block(node, source)?)),

        "_ground" => Ok(GroundExpression::Ground(parse_ground(node, source)?)),

        "collection" => Ok(GroundExpression::Collection(parse_collection(
            node, source,
        )?)),

        "_proc_var" => Ok(GroundExpression::ProcVar(parse_proc_var(node, source)?)),

        "simple_type" => Ok(GroundExpression::SimpleType(parse_simple_type(
            node, source,
        )?)),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of _ground_expression",
            node.kind(),
        ))),
    }
}

fn parse_block(node: &Node, source: &str) -> Result<Block, InterpreterError> {
    let body_node = get_child_by_field_name(node, "body")?;
    let body_proc = parse_proc(&body_node, source)?;

    Ok(Block {
        proc: body_proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_ground(node: &Node, source: &str) -> Result<Ground, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "bool_literal" => Ok(Ground::BoolLiteral {
            value: match node.kind() {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(InterpreterError::ParserError(format!(
                        "Invalid bool literal value: {}",
                        node.kind()
                    )));
                }
            },
            line_num,
            col_num,
        }),

        "long_literal" => Ok(Ground::LongLiteral {
            value: {
                let long_string = get_node_value(node, source.as_bytes())?;
                let long = long_string.parse::<i64>().or_else(|e| {
                    Err(InterpreterError::ParserError(format!(
                        "Failed to convert long_literal into i64. Error: {:?}",
                        e,
                    )))
                })?;
                long
            },
            line_num,
            col_num,
        }),

        "string_literal" => Ok(Ground::StringLiteral {
            value: get_node_value(node, source.as_bytes())?,
            line_num,
            col_num,
        }),

        "uri_literal" => Ok(Ground::UriLiteral(parse_uri_literal(node, source)?)),

        "nil" => Ok(Ground::Nil { line_num, col_num }),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of _ground",
            node.kind(),
        ))),
    }
}

fn parse_uri_literal(node: &Node, source: &str) -> Result<UriLiteral, InterpreterError> {
    Ok(UriLiteral {
        value: get_node_value(node, source.as_bytes())?,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_collection(node: &Node, source: &str) -> Result<Collection, InterpreterError> {
    todo!()
}

fn parse_simple_type(node: &Node, source: &str) -> Result<SimpleType, InterpreterError> {
    todo!()
}

fn parse_proc_list(node: &Node, source: &str) -> Result<ProcList, InterpreterError> {
    let mut procs = Vec::new();
    let mut cursor = node.walk();

    cursor.goto_first_child(); // '('

    while cursor.goto_next_sibling() {
        let current_node = cursor.node();
        match current_node.kind() {
            "_proc" => procs.push(parse_proc(&current_node, source)?),

            "," => continue,

            _ => break,
        }
    }

    Ok(ProcList {
        procs,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

// fn parse_decls(cursor: &mut TreeCursor, source: &str) -> Vec<String> {
//     let mut decls = Vec::new();
//     cursor.goto_first_child();
//     loop {
//         let decl = cursor
//             .node()
//             .utf8_text(source.as_bytes())
//             .unwrap()
//             .to_string();
//         decls.push(decl);
//         if !cursor.goto_next_sibling() {
//             break;
//         }
//     }
//     cursor.goto_parent();
//     decls
// }

// fn parse_cases(cursor: &mut TreeCursor, source: &str) -> Vec<(Proc, Proc)> {
//     let mut cases = Vec::new();
//     cursor.goto_first_child();
//     loop {
//         let pattern = parse_proc(cursor, source);
//         cursor.goto_next_sibling();
//         let proc = parse_proc(cursor, source);
//         cases.push((pattern, proc));
//         if !cursor.goto_next_sibling() {
//             break;
//         }
//     }
//     cursor.goto_parent();
//     cases
// }

// fn parse_branches(cursor: &mut TreeCursor, source: &str) -> Vec<(Proc, Proc)> {
//     let mut branches = Vec::new();
//     cursor.goto_first_child();
//     loop {
//         let pattern = parse_proc(cursor, source);
//         cursor.goto_next_sibling();
//         let proc = parse_proc(cursor, source);
//         branches.push((pattern, proc));
//         if !cursor.goto_next_sibling() {
//             break;
//         }
//     }
//     cursor.goto_parent();
//     branches
// }

// fn parse_names(cursor: &mut TreeCursor, source: &str) -> Vec<String> {
//     let mut names = Vec::new();
//     cursor.goto_first_child();
//     loop {
//         let name = cursor
//             .node()
//             .utf8_text(source.as_bytes())
//             .unwrap()
//             .to_string();
//         names.push(name);
//         if !cursor.goto_next_sibling() {
//             break;
//         }
//     }
//     cursor.goto_parent();
//     names
// }

fn get_child_by_field_name<'a>(
    node: &'a Node<'a>,
    name: &'a str,
) -> Result<Node<'a>, InterpreterError> {
    match node.child_by_field_name(name) {
        Some(child_node) => Ok(child_node),
        None => Err(InterpreterError::ParserError(format!(
            "Error: did not find expected field: {:?}, on node {:?}",
            name,
            node.kind(),
        ))),
    }
}

fn get_node_value(node: &Node, bytes: &[u8]) -> Result<String, InterpreterError> {
    match node.utf8_text(bytes) {
        Ok(str) => Ok(str.to_string()),
        Err(e) => Err(InterpreterError::ParserError(format!(
            "Failed to get node value. Error: {:?}",
            e,
        ))),
    }
}
