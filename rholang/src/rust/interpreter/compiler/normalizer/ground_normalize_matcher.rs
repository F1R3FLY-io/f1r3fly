use tree_sitter::Node;
use super::exports::parse_rholang_code;
use super::exports::normalize_bool;

/*
  This normalizer works with various types of "ground" (primitive) values, such as Bool, Int, String, and Uri.
 */

#[derive(Debug, PartialEq)]
pub enum Ground {
  Bool(bool),
  Int(i64),
  String(String),
  Uri(String),
}

pub fn normalize_ground(node: Node, source_code: &[u8]) -> Option<Ground> {
  match node.kind() {
    "bool_literal" => {
      if let Some(value) = normalize_bool(node, source_code) {
        return Some(Ground::Bool(value));
      }
      None
    },
    "long_literal" => {
      let text = node.utf8_text(source_code).unwrap();
      text.parse::<i64>().ok().map(Ground::Int)
    }
    "string_literal" => {
      let text = node.utf8_text(source_code).unwrap();
      Some(Ground::String(strip_string(text)))
    }
    "uri_literal" => {
      let text = node.utf8_text(source_code).unwrap();
      Some(Ground::Uri(strip_uri(text)))
    }
    _ => None,
  }
}

fn strip_uri(raw: &str) -> String {
  raw[1..raw.len() - 1].to_string()
}

fn strip_string(raw: &str) -> String {
  raw[1..raw.len() - 1].to_string()
}

// first 3 tests based on src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/GroundMatcherSpec.scala
#[test]
fn test_normalize_ground_int() {
  let rholang_code = "42";

  let tree = parse_rholang_code(rholang_code);
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let root = tree.root_node();

  let literal_node = root
    .child(0)
    .expect("Expected a long_literal node");

  assert_eq!(literal_node.kind(), "long_literal");

  let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
    .expect("Expected to normalize an int");

  assert_eq!(normalized_ground, Ground::Int(42));
}

#[test]
fn test_normalize_ground_string() {
  let rholang_code = r#""Hello, Rholang!""#;

  let tree = parse_rholang_code(rholang_code);
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let root = tree.root_node();
  let literal_node = root
    .child(0)
    .expect("Expected a string_literal node");
  let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
    .expect("Expected to normalize a string");

  assert_eq!(normalized_ground, Ground::String("Hello, Rholang!".to_string()));
}

#[test]
fn test_normalize_ground_uri() {
  let rholang_code = "`http://example.com`";

  let tree = parse_rholang_code(rholang_code);
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let root = tree.root_node();
  let literal_node = root
    .child(0)
    .expect("Expected a uri_literal node");
  let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
    .expect("Expected to normalize a uri");

  assert_eq!(normalized_ground, Ground::Uri("http://example.com".to_string()));
}

#[test]
fn test_normalize_ground_bool() {
  let rholang_code = r#"true"#;

  let tree = parse_rholang_code(rholang_code);
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let root = tree.root_node();
  let literal_node = root
    .child(0)
    .expect("Expected a bool_literal node");
  let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
    .expect("Expected to normalize a bool");

  assert_eq!(normalized_ground, Ground::Bool(true));
}