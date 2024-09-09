use tree_sitter::Node;
use super::exports::parse_rholang_code;

/*
  This normalizer works with various types of "ground" (primitive) values, such as Bool, Int, String, and Uri.
 */

#[derive(Debug, PartialEq)]
enum Ground {
  Bool(bool),
  Int(i64),
  String(String),
  Uri(String),
}

fn normalize_ground(node: Node, source_code: &[u8]) -> Option<Ground> {
  match node.kind() {
    "bool_literal" => {
      let text = node.utf8_text(source_code).unwrap();
      match text {
        "true" => Some(Ground::Bool(true)),
        "false" => Some(Ground::Bool(false)),
        _ => None,
      }
    }
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
  raw[1..raw.len() - 1].to_string()  // Видаляємо зворотні лапки з URI
}

fn strip_string(raw: &str) -> String {
  raw[1..raw.len() - 1].to_string()  // Видаляємо лапки зі строки
}

#[test]
fn test_normalize_ground() {
  let cases = vec![
    (r#""Hello, Rholang!""#, "string_literal", Ground::String("Hello, Rholang!".to_string())),
    ("true", "bool_literal", Ground::Bool(true)),
    ("42", "long_literal", Ground::Int(42)),
    ("`http://example.com`", "uri_literal", Ground::Uri("http://example.com".to_string())),
  ];

  for (rholang_code, expected_kind, expected_ground) in cases {
    let tree = parse_rholang_code(rholang_code);
    println!("Tree S-expression: {}", tree.root_node().to_sexp());

    let root = tree.root_node();
    assert_eq!(root.kind(), "source_file");

    let literal_node = root
      .child(0)  // proc16
      .and_then(|n| n.child(0))  // ground
      .and_then(|n| n.child(0))  // literal (string, bool, int, uri)
      .expect(&format!("Expected a {} node", expected_kind));

    assert_eq!(literal_node.kind(), expected_kind);

    let normalized_ground = normalize_ground(literal_node, rholang_code.as_bytes())
      .expect(&format!("Expected to normalize a {}", expected_kind));
    println!("Normalized ground for {}: {:?}", rholang_code, normalized_ground);
    assert_eq!(normalized_ground, expected_ground);
  }
}