use crate::rust::interpreter::compiler::normalizer::exports::normalize_bool;
use crate::rust::interpreter::compiler::rholang_ast::{Proc, UriLiteral};
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::Expr;
use models::rust::utils::{new_gbool_expr, new_gint_expr, new_gstring_expr, new_guri_expr};

/*
 This normalizer works with various types of "ground" (primitive) values, such as Bool, Int, String, and Uri.
*/
pub fn normalize_ground(proc: &Proc) -> Result<Expr, InterpreterError> {
    match proc.clone() {
        Proc::BoolLiteral { .. } => Ok(new_gbool_expr(normalize_bool(proc)?)),

        Proc::LongLiteral { value, .. } => Ok(new_gint_expr(value)),

        // The 'value' here is already stripped. This happens in custom parser.
        Proc::StringLiteral { value, .. } => Ok(new_gstring_expr(value)),

        // The 'value' here is already stripped. This happens in custom parser.
        Proc::UriLiteral(UriLiteral { value, .. }) => Ok(new_guri_expr(value)),

        _ => Err(InterpreterError::BugFoundError(format!(
            "Expected a ground type, found: {:?}",
            proc
        ))),
    }
}

// first 3 tests based on src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/GroundMatcherSpec.scala
// #[test]
// fn test_normalize_ground_int() {
//   let rholang_code = "42";

//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let root = tree.root_node();

//   let literal_node = root
//     .child(0)
//     .expect("Expected a long_literal node");

//   assert_eq!(literal_node.kind(), "long_literal");

//   let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
//     .expect("Expected to normalize an int");

//   assert_eq!(normalized_ground, Ground::Int(42));
// }

// #[test]
// fn test_normalize_ground_string() {
//   let rholang_code = r#""Hello, Rholang!""#;

//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let root = tree.root_node();
//   let literal_node = root
//     .child(0)
//     .expect("Expected a string_literal node");
//   let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
//     .expect("Expected to normalize a string");

//   assert_eq!(normalized_ground, Ground::String("Hello, Rholang!".to_string()));
// }

// #[test]
// fn test_normalize_ground_uri() {
//   let rholang_code = "`http://example.com`";

//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let root = tree.root_node();
//   let literal_node = root
//     .child(0)
//     .expect("Expected a uri_literal node");
//   let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
//     .expect("Expected to normalize a uri");

//   assert_eq!(normalized_ground, Ground::Uri("http://example.com".to_string()));
// }

// #[test]
// fn test_normalize_ground_bool() {
//   let rholang_code = r#"true"#;

//   let tree = parse_rholang_code(rholang_code);
//   println!("Tree S-expression: {}", tree.root_node().to_sexp());
//   let root = tree.root_node();
//   let literal_node = root
//     .child(0)
//     .expect("Expected a bool_literal node");
//   let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
//     .expect("Expected to normalize a bool");

//   assert_eq!(normalized_ground, Ground::Bool(true));
// }
