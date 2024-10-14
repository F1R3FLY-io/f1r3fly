use tree_sitter::{Parser, Tree};

use crate::rust::interpreter::{compiler::rholang_ast::Proc, errors::InterpreterError};

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
            "Expected a child node".to_string(),
        )),
    }?;

    let mut cursor = start_node.walk();
    loop {
        let node = cursor.node();
        let node_name = node.kind();
        let node_value = node.utf8_text(code.as_bytes()).unwrap_or("");

        println!("Node: {}, Value: {}", node_name, node_value);

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return Err(InterpreterError::ParserError(
                    "Traversal completed".to_string(),
                ));
            }
        }
    }
}

pub fn parse_rholang_code(code: &str) -> Proc {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_rholang() };
    parser
        .set_language(language)
        .expect("Error loading Rholang grammar");

    let tree = parser.parse(code, None).expect("Failed to parse code");
    let root_node = tree.root_node();
    let mut cursor = root_node.walk();

    if let Some(first_child) = root_node.child(0) {
        cursor.goto_first_child();
        parse_proc(&mut cursor, code)
    } else {
        panic!("The code does not contain any valid Rholang process");
    }
}

fn parse_proc(cursor: &mut TreeCursor, source: &str) -> Proc {
    let node = cursor.node();
    match node.kind() {
        "par" => {
            cursor.goto_first_child();
            let left = parse_proc(cursor, source);
            cursor.goto_next_sibling();
            let right = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::Par(Box::new(left), Box::new(right))
        }
        "send_sync" => {
            cursor.goto_first_child();
            let name = node
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_string();
            cursor.goto_next_sibling();
            let messages = parse_proc_list(cursor, source);
            cursor.goto_next_sibling();
            let cont = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::SendSync(name, messages, Box::new(cont))
        }
        "new" => {
            cursor.goto_first_child();
            let decls = parse_decls(cursor, source);
            cursor.goto_next_sibling();
            let proc = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::New(decls, Box::new(proc))
        }
        "ifElse" => {
            cursor.goto_first_child();
            let condition = parse_proc(cursor, source);
            cursor.goto_next_sibling();
            let if_true = parse_proc(cursor, source);
            let alternative = if cursor.goto_next_sibling() {
                Some(Box::new(parse_proc(cursor, source)))
            } else {
                None
            };
            cursor.goto_parent();
            Proc::IfElse(Box::new(condition), Box::new(if_true), alternative)
        }
        "let" => {
            cursor.goto_first_child();
            let decls = parse_decls(cursor, source);
            cursor.goto_next_sibling();
            let body = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::Let(decls, Box::new(body))
        }
        "bundle" => {
            cursor.goto_first_child();
            let bundle_type = node
                .child_by_field_name("bundle_type")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_string();
            cursor.goto_next_sibling();
            let proc = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::Bundle(bundle_type, Box::new(proc))
        }
        "match" => {
            cursor.goto_first_child();
            let expression = parse_proc(cursor, source);
            cursor.goto_next_sibling();
            let cases = parse_cases(cursor, source);
            cursor.goto_parent();
            Proc::Match(Box::new(expression), cases)
        }
        "choice" => {
            cursor.goto_first_child();
            let branches = parse_branches(cursor, source);
            cursor.goto_parent();
            Proc::Choice(branches)
        }
        "contract" => {
            cursor.goto_first_child();
            let name = node
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_string();
            cursor.goto_next_sibling();
            let formals = parse_names(cursor, source);
            cursor.goto_next_sibling();
            let proc = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::Contract(name, formals, Box::new(proc))
        }
        "input" => {
            cursor.goto_first_child();
            let formals = parse_names(cursor, source);
            cursor.goto_next_sibling();
            let proc = parse_proc(cursor, source);
            cursor.goto_parent();
            Proc::Input(formals, Box::new(proc))
        }
        "send" => {
            cursor.goto_first_child();
            let name = node
                .child_by_field_name("name")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_string();
            cursor.goto_next_sibling();
            let inputs = parse_proc_list(cursor, source);
            cursor.goto_parent();
            Proc::Send(name, inputs)
        }
        "var_ref" => {
            let var = node
                .child_by_field_name("var")
                .unwrap()
                .utf8_text(source.as_bytes())
                .unwrap()
                .to_string();
            Proc::VarRef(var)
        }
        "nil" => Proc::Nil,
        _ => Proc::Ground(node.utf8_text(source.as_bytes()).unwrap().to_string()),
    }
}

fn parse_proc_list(cursor: &mut TreeCursor, source: &str) -> Vec<Proc> {
  let mut procs = Vec::new();
  cursor.goto_first_child();
  loop {
      procs.push(parse_proc(cursor, source));
      if !cursor.goto_next_sibling() {
          break;
      }
  }
  cursor.goto_parent();
  procs
}

fn parse_decls(cursor: &mut TreeCursor, source: &str) -> Vec<String> {
  let mut decls = Vec::new();
  cursor.goto_first_child();
  loop {
      let decl = cursor.node().utf8_text(source.as_bytes()).unwrap().to_string();
      decls.push(decl);
      if !cursor.goto_next_sibling() {
          break;
      }
  }
  cursor.goto_parent();
  decls
}

fn parse_cases(cursor: &mut TreeCursor, source: &str) -> Vec<(Proc, Proc)> {
  let mut cases = Vec::new();
  cursor.goto_first_child();
  loop {
      let pattern = parse_proc(cursor, source);
      cursor.goto_next_sibling();
      let proc = parse_proc(cursor, source);
      cases.push((pattern, proc));
      if !cursor.goto_next_sibling() {
          break;
      }
  }
  cursor.goto_parent();
  cases
}

fn parse_branches(cursor: &mut TreeCursor, source: &str) -> Vec<(Proc, Proc)> {
  let mut branches = Vec::new();
  cursor.goto_first_child();
  loop {
      let pattern = parse_proc(cursor, source);
      cursor.goto_next_sibling();
      let proc = parse_proc(cursor, source);
      branches.push((pattern, proc));
      if !cursor.goto_next_sibling() {
          break;
      }
  }
  cursor.goto_parent();
  branches
}

fn parse_names(cursor: &mut TreeCursor, source: &str) -> Vec<String> {
  let mut names = Vec::new();
  cursor.goto_first_child();
  loop {
      let name = cursor.node().utf8_text(source.as_bytes()).unwrap().to_string();
      names.push(name);
      if !cursor.goto_next_sibling() {
          break;
      }
  }
  cursor.goto_parent();
  names
}