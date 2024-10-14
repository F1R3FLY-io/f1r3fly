use tree_sitter::{Node, Parser, Tree, TreeCursor};

use crate::rust::interpreter::{
    compiler::rholang_ast::{Name, Proc, ProcVar, Quote, SyncSendCont},
    errors::InterpreterError,
    unwrap_option_safe,
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
                if cont_node.kind() == "non_empty_cont" {
                    SyncSendCont::NonEmpty {
                        proc: Box::new(parse_proc(&cont_node, source)?),
                        line_num: cont_node.start_position().row,
                        col_num: cont_node.start_position().column,
                    }
                } else if node.kind() == "empty_cont" {
                    SyncSendCont::Empty {
                        line_num: cont_node.start_position().row,
                        col_num: cont_node.start_position().column,
                    }
                } else {
                    return Err(InterpreterError::ParserError(format!(
                        "Unexpected choice node kind: {:?} on sync_send_cont",
                        node.kind(),
                    )));
                }
            };

            Ok(Proc::SendSync {
                name: name_val,
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
          // _ => Proc::Ground(node.utf8_text(source.as_bytes()).unwrap().to_string()),
    }
}

fn parse_name(node: &Node, source: &str) -> Result<Name, InterpreterError> {
    if node.kind() == "_proc_var" {
        Ok(Name::NameProcVar(parse_proc_var(node, source)?))
    } else if node.kind() == "quote" {
        Ok(Name::NameQuote(Box::new(parse_quote(node, source)?)))
    } else {
        return Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} on name",
            node.kind(),
        )));
    }
}

fn parse_proc_var(node: &Node, source: &str) -> Result<ProcVar, InterpreterError> {
    if node.kind() == "wildcard" {
        Ok(ProcVar::Wildcard {
            line_num: node.start_position().row,
            col_num: node.start_position().column,
        })
    } else if node.kind() == "var" {
        Ok(Name::NameQuote(Box::new(parse_quote(node, source)?)))
    } else {
        return Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} on _proc_var",
            node.kind(),
        )));
    }
}

fn parse_quote(node: &Node, source: &str) -> Result<Quote, InterpreterError> {
    todo!()
}

fn parse_proc_list(node: &Node, source: &str) -> Result<Vec<Proc>, InterpreterError> {
    let mut procs = Vec::new();
    let comma_sep_node = get_child_by_field_name(node, "commaSep")?;
    let mut cursor = comma_sep_node.walk();
    if cursor.goto_first_child() {
        loop {
            procs.push(parse_proc(&cursor.node(), source)?);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    Ok(procs)
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
