use rspace_plus_plus::rspace::history::Either;
use tree_sitter::{Node, Parser, Tree};

use crate::rust::interpreter::{
    compiler::rholang_ast::{
        Block, Branch, Bundle, Case, Collection, Conjunction, Decl, Decls, DeclsChoice,
        Disjunction, Eval, KeyValuePair, LinearBind, Name, NameDecl, Names, Negation, Proc,
        ProcList, Quotable, Quote, Receipt, ReceiptBindings, SendRule, SendType, SimpleType,
        Source, SyncSendCont, UriLiteral, Var, VarRef, VarRefKind,
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
    println!("\nTree: {:#?}", tree.root_node().to_sexp());

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

    // println!("\n{:?}", root_node.child(0).unwrap().kind());

    parse_proc(&start_node, code)
}

/*
 * TODO: How to handle '_line_comment' and '_block_comment'
 * TODO: VarRef
 */
fn parse_proc(node: &Node, source: &str) -> Result<Proc, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "par" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Par {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
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
                    "empty_cont" => SyncSendCont::Empty {
                        line_num: cont_node.start_position().row,
                        col_num: cont_node.start_position().column,
                    },

                    "non_empty_cont" => {
                        let proc_node = match cont_node.child(1) {
                            Some(_proc_node) => Ok(_proc_node),
                            None => Err(InterpreterError::ParserError(
                                "Expected a _proc node in non_empty_cont at index 1".to_string(),
                            )),
                        }?;

                        SyncSendCont::NonEmpty {
                            proc: Box::new(parse_proc(&proc_node, source)?),
                            line_num: cont_node.start_position().row,
                            col_num: cont_node.start_position().column,
                        }
                    }

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
                line_num,
                col_num,
            })
        }

        "new" => {
            let decls_node = get_child_by_field_name(node, "decls")?;
            let decls_proc = parse_decls(&decls_node, source)?;

            let proc_node = get_child_by_field_name(node, "proc")?;
            let proc = parse_proc(&proc_node, source)?;
            Ok(Proc::New {
                decls: decls_proc,
                proc: Box::new(proc),
                line_num,
                col_num,
            })
        }

        "ifElse" => {
            let condition_node = get_child_by_field_name(node, "condition")?;
            let condition_proc = parse_proc(&condition_node, source)?;

            let if_true_node = get_child_by_field_name(node, "ifTrue")?;
            let if_true_proc = parse_proc(&if_true_node, source)?;

            let alternative_proc = match node.child_by_field_name("alternative") {
                Some(alternative_node) => {
                    let proc_node = match alternative_node.child(1) {
                        Some(_proc_node) => Ok(_proc_node),
                        None => Err(InterpreterError::ParserError(
                            "Expected a _proc node in alternative at index 1".to_string(),
                        )),
                    }?;

                    Some(Box::new(parse_proc(&proc_node, source)?))
                }
                None => None,
            };

            Ok(Proc::IfElse {
                condition: Box::new(condition_proc),
                if_true: Box::new(if_true_proc),
                alternative: alternative_proc,
                line_num,
                col_num,
            })
        }

        "let" => {
            let decls_node = get_child_by_field_name(node, "decls")?;
            let mut decls = Vec::new();
            let mut cursor = decls_node.walk();
            cursor.goto_first_child(); // 'semiSep1' and `conc1` ensures there is at least one child

            loop {
                let current_node = cursor.node();
                if current_node.kind() == "decl" {
                    decls.push(parse_decl(&current_node, source)?);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }

            let decls_row = decls_node.start_position().row;
            let decls_col = decls_node.start_position().column;

            let decls_proc = match decls_node.kind() {
                "linear_decls" => Ok(DeclsChoice::LinearDecls {
                    decls,
                    line_num: decls_row,
                    col_num: decls_col,
                }),

                "conc_decls" => Ok(DeclsChoice::ConcDecls {
                    decls,
                    line_num: decls_row,
                    col_num: decls_col,
                }),

                _ => Err(InterpreterError::ParserError(format!(
                    "Unexpected choice node kind: {:?} of _decls",
                    node.kind(),
                ))),
            }?;

            let body_node = get_child_by_field_name(node, "body")?;
            let body_proc = parse_block(&body_node, source)?;

            Ok(Proc::Let {
                decls: decls_proc,
                body: Box::new(body_proc),
                line_num,
                col_num,
            })
        }

        "bundle" => {
            let bundle_node = get_child_by_field_name(node, "bundle_type")?;
            let bundle = parse_bundle_choice(&bundle_node)?;

            let proc_node = get_child_by_field_name(node, "proc")?;
            let proc = parse_block(&proc_node, source)?;

            Ok(Proc::Bundle {
                bundle_type: bundle,
                proc: Box::new(proc),
                line_num,
                col_num,
            })
        }

        "match" => {
            let expression_node = get_child_by_field_name(node, "expression")?;
            let expression_proc = parse_proc(&expression_node, source)?;

            let cases_node = get_child_by_field_name(node, "cases")?;
            let mut cases = Vec::new();
            let mut cursor = cases_node.walk();
            cursor.goto_first_child(); // 'repeat1' ensures there is at least one child

            loop {
                let current_node = cursor.node();
                if current_node.kind() == "case" {
                    cases.push(parse_case(&current_node, source)?);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }

            Ok(Proc::Match {
                expression: Box::new(expression_proc),
                cases,
                line_num,
                col_num,
            })
        }

        "choice" => {
            let branches_node = get_child_by_field_name(node, "branches")?;
            let mut branches = Vec::new();
            let mut cursor = branches_node.walk();
            cursor.goto_first_child(); // 'repeat1' ensures there is at least one child

            loop {
                let current_node = cursor.node();
                if current_node.kind() == "branch" {
                    branches.push(parse_branch(&current_node, source)?);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }

            Ok(Proc::Choice {
                branches,
                line_num,
                col_num,
            })
        }

        "contract" => {
            let name_node = get_child_by_field_name(node, "name")?;
            let name_proc = parse_name(&name_node, source)?;

            let formals_node = get_child_by_field_name(node, "formals")?;
            let formals_proc = parse_names(&formals_node, source)?;

            let proc_node = get_child_by_field_name(node, "proc")?;
            let proc = parse_block(&proc_node, source)?;

            Ok(Proc::Contract {
                name: name_proc,
                formals: formals_proc,
                proc: Box::new(proc),
                line_num,
                col_num,
            })
        }

        "input" => {
            let formals_node = get_child_by_field_name(node, "formals")?;
            let formals_proc = parse_receipts(&formals_node, source)?;

            let proc_node = get_child_by_field_name(node, "proc")?;
            let proc = parse_block(&proc_node, source)?;

            Ok(Proc::Input {
                formals: formals_proc,
                proc: Box::new(proc),
                line_num,
                col_num,
            })
        }

        "send" => {
            let name_node = get_child_by_field_name(node, "name")?;
            let name_proc = parse_name(&name_node, source)?;

            let send_type_node = get_child_by_field_name(node, "send_type")?;
            let send_type_proc = parse_send_type(&send_type_node)?;

            let inputs_node = get_child_by_field_name(node, "inputs")?;
            let inputs_proc = parse_proc_list(&inputs_node, source)?;

            Ok(Proc::Send {
                name: name_proc,
                send_type: send_type_proc,
                inputs: inputs_proc,
                line_num,
                col_num,
            })
        }

        // _proc_expression
        "or" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Or {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "and" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::And {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "matches" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Matches {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "eq" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Eq {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "neq" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Neq {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "lt" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Lt {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "lte" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Lte {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "gt" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Gt {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "gte" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Gte {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "concat" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Concat {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "minus_minus" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::MinusMinus {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "minus" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Minus {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "add" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Add {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "percent_percent" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::PercentPercent {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "mult" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Mult {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "div" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Div {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "mod" => {
            let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

            Ok(Proc::Mod {
                left: Box::new(left_proc),
                right: Box::new(right_proc),
                line_num,
                col_num,
            })
        }

        "not" => {
            let proc_node = get_child_by_field_name(node, "proc")?;
            let proc = parse_proc(&proc_node, source)?;

            Ok(Proc::Not {
                proc: Box::new(proc),
                line_num,
                col_num,
            })
        }

        "neg" => {
            let proc_node = get_child_by_field_name(node, "proc")?;
            let proc = parse_proc(&proc_node, source)?;

            Ok(Proc::Neg {
                proc: Box::new(proc),
                line_num,
                col_num,
            })
        }

        "method" => {
            let receiver_node = get_child_by_field_name(node, "receiver")?;
            let receiver_proc = parse_proc(&receiver_node, source)?;

            let name_node = get_child_by_field_name(node, "name")?;
            let name_proc = parse_var(&name_node, source)?;

            let args_node = get_child_by_field_name(node, "args")?;
            let args_proc = parse_proc_list(&args_node, source)?;

            Ok(Proc::Method {
                receiver: Box::new(receiver_proc),
                name: name_proc,
                args: args_proc,
                line_num,
                col_num,
            })
        }

        // TODO: Figure this case out
        "_parenthesized" => {
            todo!()
        }

        "eval" => Ok(Proc::Eval(parse_eval(node, source)?)),

        "quote" => Ok(Proc::Quote(parse_quote(node, source)?)),

        "disjunction" => Ok(Proc::Disjunction(parse_disjunction(node, source)?)),

        "conjuction" => Ok(Proc::Conjunction(parse_conjuction(node, source)?)),

        "negation" => Ok(Proc::Negation(parse_negation(node, source)?)),

        // _ground_expression
        "block" => Ok(Proc::Block(Box::new(parse_block(node, source)?))),

        "collection" => Ok(Proc::Collection(parse_collection(node, source)?)),

        "simple_type" => Ok(Proc::SimpleType(parse_simple_type(node)?)),

        //_ground
        "bool_literal" => Ok(Proc::BoolLiteral {
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

        "long_literal" => Ok(Proc::LongLiteral {
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

        "string_literal" => Ok(Proc::StringLiteral {
            value: {
                let value_with_quotes = get_node_value(node, source.as_bytes())?;
                let value = value_with_quotes.trim_matches('"').to_string();
                value
            },
            line_num,
            col_num,
        }),

        "uri_literal" => Ok(Proc::UriLiteral(parse_uri_literal(node, source)?)),

        "nil" => Ok(Proc::Nil { line_num, col_num }),

        // _proc_var
        "wildcard" => Ok(Proc::Wildcard {
            line_num: node.start_position().row,
            col_num: node.start_position().column,
        }),

        "var" => Ok(Proc::Var(parse_var(node, source)?)),

        // var_ref
        "var_ref" => Ok(Proc::VarRef(parse_var_ref(node, source)?)),

        _ => Err(InterpreterError::ParserError(format!(
            "Unrecognizable process. Node: {:?}",
            node.kind()
        ))),
    }
}

fn parse_name(node: &Node, source: &str) -> Result<Name, InterpreterError> {
    // println!("\nproc_var node: {:?}", node.to_sexp());

    match node.kind() {
        "quote" => Ok(Name::Quote(Box::new(parse_quote(node, source)?))),

        _ => {
            let proc_result = parse_proc(node, source);
            match proc_result {
                Ok(proc) => Ok(Name::ProcVar(Box::new(proc))),
                Err(proc_err) => Err(InterpreterError::ParserError(format!(
                    "{:?}. Unexpected choice node kind: {:?} of name.",
                    proc_err,
                    node.kind(),
                ))),
            }
        }
    }
}

fn parse_var(node: &Node, source: &str) -> Result<Var, InterpreterError> {
    let var_value = get_node_value(node, source.as_bytes())?;
    Ok(Var {
        name: var_value,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_var_ref(node: &Node, source: &str) -> Result<VarRef, InterpreterError> {
    let var_ref_kind_node = node
        .child(0)
        .filter(|n| n.kind() == "var_ref_kind")
        .ok_or_else(|| {
            InterpreterError::ParserError(
                "Expected a var_ref_kind node in var_ref at index 0".to_string(),
            )
        })?;

    let var_ref_kind = match var_ref_kind_node.kind() {
        "=" => VarRefKind::Proc,
        "=*" => VarRefKind::Name,
        _ => {
            return Err(InterpreterError::ParserError(format!(
                "Unexpected choice node kind: {:?} of var_ref_kind.",
                node.kind(),
            )))
        }
    };

    let var_node = node.child(1).filter(|n| n.kind() == "var").ok_or_else(|| {
        InterpreterError::ParserError("Expected a var node in var_ref at index 1".to_string())
    })?;

    let var = parse_var(&var_node, source)?;

    Ok(VarRef {
        var_ref_kind,
        var,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
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
        quotable: Box::new(parse_quotable(&quotable_node, source)?),
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

        _ => {
            let proc_result = parse_proc(node, source);
            match proc_result {
                Ok(proc) => Ok(Quotable::GroundExpression(proc)),
                Err(proc_err) => Err(InterpreterError::ParserError(format!(
                    "{:?}. Unexpected choice node kind: {:?} of quotable.",
                    proc_err,
                    node.kind(),
                ))),
            }
        }
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
    let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

    Ok(Disjunction {
        left: Box::new(left_proc),
        right: Box::new(right_proc),
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_conjuction(node: &Node, source: &str) -> Result<Conjunction, InterpreterError> {
    let (left_proc, right_proc) = parse_left_and_right_nodes(node, source)?;

    Ok(Conjunction {
        left: Box::new(left_proc),
        right: Box::new(right_proc),
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_negation(node: &Node, source: &str) -> Result<Negation, InterpreterError> {
    let proc_node = get_child_by_field_name(node, "proc")?;
    let proc = parse_proc(&proc_node, source)?;

    Ok(Negation {
        proc: Box::new(proc),
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
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

fn parse_uri_literal(node: &Node, source: &str) -> Result<UriLiteral, InterpreterError> {
    Ok(UriLiteral {
        value: {
            let value_with_quotes = get_node_value(node, source.as_bytes())?;
            let value = value_with_quotes.trim_matches('"').to_string();
            value
        },
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_collection(node: &Node, source: &str) -> Result<Collection, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "list" => Ok(Collection::List {
            elements: {
                match node.child(1) {
                    Some(comma_sep_procs_node) => {
                        parse_comma_sep_procs(&comma_sep_procs_node, source)?
                    }
                    None => {
                        return Err(InterpreterError::ParserError(
                            "Expected a commaSep node of procs in list at index 1".to_string(),
                        ))
                    }
                }
            },
            cont: {
                match node.child_by_field_name("cont") {
                    Some(proc_remainder_node) => {
                        Some(Box::new(parse_proc(&proc_remainder_node, source)?))
                    }
                    None => None,
                }
            },
            line_num,
            col_num,
        }),

        "set" => Ok(Collection::Set {
            elements: {
                match node.child(1) {
                    Some(comma_sep_procs_node) => {
                        parse_comma_sep_procs(&comma_sep_procs_node, source)?
                    }
                    None => {
                        return Err(InterpreterError::ParserError(
                            "Expected a commaSep node of procs in set at index 1".to_string(),
                        ))
                    }
                }
            },
            cont: {
                match node.child_by_field_name("cont") {
                    Some(proc_remainder_node) => {
                        Some(Box::new(parse_proc(&proc_remainder_node, source)?))
                    }
                    None => None,
                }
            },
            line_num,
            col_num,
        }),

        "map" => Ok(Collection::Map {
            pairs: {
                let mut kvs = Vec::new();
                let mut cursor = node.walk();

                cursor.goto_first_child(); // '{'

                while cursor.goto_next_sibling() {
                    let current_node = cursor.node();
                    match current_node.kind() {
                        "key_value_pair" => kvs.push(parse_key_value_pair(&current_node, source)?),
                        "," => continue,
                        _ => break,
                    }
                }

                kvs
            },
            cont: {
                match node.child_by_field_name("cont") {
                    Some(proc_remainder_node) => {
                        Some(Box::new(parse_proc(&proc_remainder_node, source)?))
                    }
                    None => None,
                }
            },
            line_num,
            col_num,
        }),

        "tuple" => Ok(Collection::Tuple {
            elements: {
                match node.child(1) {
                    Some(comma_sep_procs_node) => {
                        parse_comma_sep_procs(&comma_sep_procs_node, source)?
                    }
                    None => {
                        return Err(InterpreterError::ParserError(
                            "Expected a commaSep node of procs in tuple at index 1".to_string(),
                        ))
                    }
                }
            },
            line_num,
            col_num,
        }),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of collection",
            node.kind(),
        ))),
    }
}

fn parse_comma_sep_procs(node: &Node, source: &str) -> Result<Vec<Proc>, InterpreterError> {
    let mut procs = Vec::new();
    let mut cursor = node.walk();

    loop {
        let current_node = cursor.node();
        if current_node.kind() == "," {
            continue;
        }
        procs.push(parse_proc(&current_node, source)?);
        if !cursor.goto_next_sibling() {
            break;
        }
    }

    Ok(procs)
}

fn parse_key_value_pair(node: &Node, source: &str) -> Result<KeyValuePair, InterpreterError> {
    let key_node = get_child_by_field_name(node, "key")?;
    let key_proc = parse_proc(&key_node, source)?;

    let value_node = get_child_by_field_name(node, "value")?;
    let value_proc = parse_proc(&value_node, source)?;

    Ok(KeyValuePair {
        key: key_proc,
        value: value_proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_simple_type(node: &Node) -> Result<SimpleType, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "Bool" => Ok(SimpleType::Bool { line_num, col_num }),

        "Int" => Ok(SimpleType::Int { line_num, col_num }),

        "String" => Ok(SimpleType::String { line_num, col_num }),

        "Uri" => Ok(SimpleType::Uri { line_num, col_num }),

        "ByteArray" => Ok(SimpleType::ByteArray { line_num, col_num }),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of simple_type",
            node.kind(),
        ))),
    }
}

fn parse_proc_list(node: &Node, source: &str) -> Result<ProcList, InterpreterError> {
    // println!("\nproc_list node: {:?}", node.to_sexp());

    let mut cursor = node.walk();
    cursor.goto_first_child(); // '('
    cursor.goto_next_sibling();
    let procs = parse_comma_sep_procs(&cursor.node(), source)?;

    Ok(ProcList {
        procs,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_decls(node: &Node, source: &str) -> Result<Decls, InterpreterError> {
    let mut decls = Vec::new();
    let mut cursor = node.walk();

    if cursor.goto_first_child() {
        loop {
            let current_node = cursor.node();
            if current_node.kind() == "," {
                continue;
            }
            decls.push(parse_name_decl(&current_node, source)?);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    Ok(Decls {
        decls,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_name_decl(node: &Node, source: &str) -> Result<NameDecl, InterpreterError> {
    let var_node = node.child(0).filter(|n| n.kind() == "var").ok_or_else(|| {
        InterpreterError::ParserError("Expected a var node in name_decl at index 0".to_string())
    })?;
    let var_proc = parse_var(&var_node, source)?;

    let uri = match node.child_by_field_name("uri") {
        Some(uri_literal_node) => Some(parse_uri_literal(&uri_literal_node, source)?),
        None => None,
    };

    Ok(NameDecl {
        var: var_proc,
        uri,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_decl(node: &Node, source: &str) -> Result<Decl, InterpreterError> {
    let names_node = get_child_by_field_name(node, "names")?;
    let names_proc = parse_names(&names_node, source)?;

    let procs_node = get_child_by_field_name(node, "procs")?;
    let procs = parse_comma_sep_procs(&procs_node, source)?;

    Ok(Decl {
        names: names_proc,
        procs,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_bundle_choice(node: &Node) -> Result<Bundle, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "bundle_write" => Ok(Bundle::BundleWrite { line_num, col_num }),

        "bundle_read" => Ok(Bundle::BundleRead { line_num, col_num }),

        "bundle_equiv" => Ok(Bundle::BundleEquiv { line_num, col_num }),

        "bundle_read_write" => Ok(Bundle::BundleReadWrite { line_num, col_num }),

        _ => Err(InterpreterError::ParserError(format!(
            "Unexpected choice node kind: {:?} of _bundle",
            node.kind(),
        ))),
    }
}

fn parse_case(node: &Node, source: &str) -> Result<Case, InterpreterError> {
    let pattern_node = get_child_by_field_name(node, "pattern")?;
    let pattern_proc = parse_proc(&pattern_node, source)?;

    let proc_node = get_child_by_field_name(node, "proc")?;
    let proc = parse_proc(&proc_node, source)?;

    Ok(Case {
        pattern: pattern_proc,
        proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_branch(node: &Node, source: &str) -> Result<Branch, InterpreterError> {
    let pattern_node = get_child_by_field_name(node, "pattern")?;
    let mut pattern_proc = Vec::new();
    let mut cursor = pattern_node.walk();

    if cursor.goto_first_child() {
        loop {
            let current_node = cursor.node();
            if current_node.kind() == "linear_bind" {
                pattern_proc.push(parse_linear_bind(&current_node, source)?);
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let proc_node = get_child_by_field_name(node, "proc")?;
    let proc = match proc_node.kind() {
        "send" => Either::Left(parse_send(&proc_node, source)?),

        _ => {
            let proc_result = parse_proc(node, source);
            match proc_result {
                Ok(proc) => Either::Right(proc),
                Err(proc_err) => {
                    return Err(InterpreterError::ParserError(format!(
                        "{:?}. Unexpected choice node kind: {:?} of _proc in branch",
                        proc_err,
                        node.kind(),
                    )))
                }
            }
        }
    };

    Ok(Branch {
        pattern: pattern_proc,
        proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_send(node: &Node, source: &str) -> Result<SendRule, InterpreterError> {
    let name_node = get_child_by_field_name(node, "name")?;
    let name_proc = parse_name(&name_node, source)?;

    let send_type_node = get_child_by_field_name(node, "send_type")?;
    let send_type_proc = parse_send_type(&send_type_node)?;

    let inputs_node = get_child_by_field_name(node, "inputs")?;
    let inputs_proc = parse_proc_list(&inputs_node, source)?;

    Ok(SendRule {
        name: name_proc,
        send_type: send_type_proc,
        inputs: inputs_proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_send_type(node: &Node) -> Result<SendType, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "send_single" => Ok(SendType::Single { line_num, col_num }),

        "send_multiple" => Ok(SendType::Multiple { line_num, col_num }),

        _ => {
            return Err(InterpreterError::ParserError(format!(
                "Unexpected choice node kind: {:?} of send_type",
                node.kind(),
            )))
        }
    }
}

fn parse_linear_bind(node: &Node, source: &str) -> Result<LinearBind, InterpreterError> {
    let names_node = get_child_by_field_name(node, "names")?;
    let names_proc = parse_names(&names_node, source)?;

    let input_node = get_child_by_field_name(node, "input")?;
    let input_proc = parse_source(&input_node, source)?;

    Ok(LinearBind {
        names: names_proc,
        input: input_proc,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_names(node: &Node, source: &str) -> Result<Names, InterpreterError> {
    let mut names = Vec::new();
    let mut cursor = node.walk();

    if cursor.goto_first_child() {
        loop {
            let current_node = cursor.node();
            if current_node.kind() == "name" {
                names.push(parse_name(&current_node, source)?);
            } else if current_node.kind() == "," {
                continue;
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let name_remainder = match node.child_by_field_name("cont") {
        Some(name_remainder_node) => Some(Box::new(parse_proc(&name_remainder_node, source)?)),
        None => None,
    };

    Ok(Names {
        names,
        cont: name_remainder,
        line_num: node.start_position().row,
        col_num: node.start_position().column,
    })
}

fn parse_source(node: &Node, source: &str) -> Result<Source, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    match node.kind() {
        "simple_source" => Ok(Source::Simple {
            name: parse_name(node, source)?,
            line_num,
            col_num,
        }),

        "receive_send_source" => Ok(Source::ReceiveSend {
            name: {
                let name_node = node
                    .child(0)
                    .filter(|n| n.kind() == "name")
                    .ok_or_else(|| {
                        InterpreterError::ParserError(
                            "Expected a name node in receive_send_source at index 0".to_string(),
                        )
                    })?;
                parse_name(&name_node, source)?
            },
            line_num,
            col_num,
        }),

        "send_receive_source" => Ok(Source::SendReceive {
            name: {
                let name_node = node
                    .child(0)
                    .filter(|n| n.kind() == "name")
                    .ok_or_else(|| {
                        InterpreterError::ParserError(
                            "Expected a name node in send_receive_source at index 0".to_string(),
                        )
                    })?;
                parse_name(&name_node, source)?
            },
            inputs: {
                let inputs_node = get_child_by_field_name(node, "inputs")?;
                parse_proc_list(&inputs_node, source)?
            },
            line_num,
            col_num,
        }),

        _ => {
            return Err(InterpreterError::ParserError(format!(
                "Unexpected choice node kind: {:?} of _source",
                node.kind(),
            )))
        }
    }
}

fn parse_receipts(node: &Node, source: &str) -> Result<Vec<Receipt>, InterpreterError> {
    let mut receipts = Vec::new();
    let mut cursor = node.walk();

    if cursor.goto_first_child() {
        loop {
            let current_node = cursor.node();
            if current_node.kind() == ";" {
                continue;
            }
            receipts.push(parse_receipt(&current_node, source)?);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    Ok(receipts)
}

fn parse_receipt(node: &Node, source: &str) -> Result<Receipt, InterpreterError> {
    let line_num = node.start_position().row;
    let col_num = node.start_position().column;

    let mut bindings = Vec::new();
    let mut cursor = node.walk();

    // Check which 'conc1' we are dealing with and then loop through
    match node.kind() {
        "linear_bind" => loop {
            let current_node = cursor.node();
            match current_node.kind() {
                "linear_bind" => bindings.push(ReceiptBindings::LinearBind(parse_linear_bind(
                    &current_node,
                    source,
                )?)),
                "&" => continue,
                _ => break,
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        },

        "repeated_bind" => loop {
            let current_node = cursor.node();
            match current_node.kind() {
                "repeated_bind" => {
                    let names_node = get_child_by_field_name(&current_node, "names")?;
                    let names_proc = parse_names(&names_node, source)?;

                    let input_node = get_child_by_field_name(&current_node, "input")?;
                    let input_proc = parse_name(&input_node, source)?;

                    bindings.push(ReceiptBindings::RepeatedBind {
                        names: names_proc,
                        input: input_proc,
                        line_num,
                        col_num,
                    });
                }
                "&" => continue,
                _ => break,
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        },

        "peek_bind" => loop {
            let current_node = cursor.node();
            match current_node.kind() {
                "peek_bind" => {
                    let names_node = get_child_by_field_name(&current_node, "names")?;
                    let names_proc = parse_names(&names_node, source)?;

                    let input_node = get_child_by_field_name(&current_node, "input")?;
                    let input_proc = parse_name(&input_node, source)?;

                    bindings.push(ReceiptBindings::PeekBind {
                        names: names_proc,
                        input: input_proc,
                        line_num,
                        col_num,
                    });
                }
                "&" => continue,
                _ => break,
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        },

        _ => {
            return Err(InterpreterError::ParserError(format!(
                "Unexpected choice node kind: {:?} of _receipt",
                node.kind(),
            )))
        }
    }

    Ok(Receipt {
        bindings,
        line_num,
        col_num,
    })
}

fn parse_left_and_right_nodes(node: &Node, source: &str) -> Result<(Proc, Proc), InterpreterError> {
    let left_node = get_child_by_field_name(node, "left")?;
    let left_proc = parse_proc(&left_node, source)?;

    let right_node = get_child_by_field_name(node, "right")?;
    let right_proc = parse_proc(&right_node, source)?;

    Ok((left_proc, right_proc))
}

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
    // println!("\nnode in get_node_value: {:?}", node.to_sexp());

    match node.utf8_text(bytes) {
        Ok(str) => Ok(str.to_owned()),
        Err(e) => Err(InterpreterError::ParserError(format!(
            "Failed to get node value. Error: {:?}",
            e,
        ))),
    }
}
