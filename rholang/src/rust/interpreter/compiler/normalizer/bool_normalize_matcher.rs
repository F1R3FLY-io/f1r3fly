use crate::rust::interpreter::{compiler::rholang_ast::Proc, errors::InterpreterError};

pub fn normalize_bool(proc: &Proc) -> Result<bool, InterpreterError> {
    match proc {
        Proc::BoolLiteral { value, .. } => match value {
            true => Ok(true),
            false => Ok(false),
        },

        _ => Err(InterpreterError::BugFoundError(format!(
            "Expected boolean literal proc, found {:?}",
            proc
        ))),
    }
}

// based on src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/BoolMatcherSpec.scala
#[test]
fn bool_true_should_compile_as_proc_bool() {
    let b_true = Proc::BoolLiteral {
        value: true,
        line_num: 0,
        col_num: 0,
    };

    let res = normalize_bool(&b_true);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), true);
}

// #[test]
// fn test_bool_false_normalization() {
//   let rholang_code = r#"false"#;

//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let root = tree.root_node();
//   let first_child = root.child(0).expect("Expected a child node");

//   let normalized_bool = normalize_bool(first_child, rholang_code.as_bytes())
//     .expect("Expected to normalize a bool");
//   assert_eq!(normalized_bool, false);
// }
