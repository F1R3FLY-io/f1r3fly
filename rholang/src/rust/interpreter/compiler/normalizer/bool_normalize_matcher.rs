use tree_sitter::Node;
use super::exports::parse_rholang_code;

pub fn normalize_bool(node: Node, source_code: &[u8]) -> Option<bool> {
  match node.kind() {
    "bool_literal" => {
      let text = node.utf8_text(source_code).unwrap();
      match text {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
      }
    }
    _ => {
      // recursively search for a bool literal in the children
      for i in 0..node.child_count() {
        if let Some(result) = normalize_bool(node.child(i).unwrap(), source_code) {
          return Some(result);
        }
      }
      None
    }
  }
}

#[test]
/*
proc16 — This is a node that covers various types of constructs, including ground.
ground — This is a node that covers basic value types such as bool_literal, long_literal, string_literal, etc.
bool_literal — This is an actual true or false value that is a child of ground.
 */
fn test_hello_world_contract() {
  let rholang_code = r#"true"#;

  let tree = parse_rholang_code(rholang_code);
  println!("Tree S-expression: {}", tree.root_node().to_sexp());
  let root = tree.root_node();
  assert_eq!(root.kind(), "source_file");


  let first_child = root.child(0).expect("Expected a child node");
  assert_eq!(first_child.kind(), "proc16");

  let recursive_case_proc16 = normalize_bool(first_child, rholang_code.as_bytes()).expect("Expected to normalize a bool");
  println!("Normalized bool under proc16: {}", recursive_case_proc16);
  assert_eq!(recursive_case_proc16, true);

  let ground_node = first_child.child(0).expect("Expected a ground node");
  assert_eq!(ground_node.kind(), "ground");


  let bool_node = ground_node.child(0).expect("Expected a bool literal node");
  assert_eq!(bool_node.kind(), "bool_literal");

  let normalized_bool = normalize_bool(bool_node, rholang_code.as_bytes()).expect("Expected to normalize a bool");
  println!("Normalized bool: {}", normalized_bool);
  assert_eq!(normalized_bool, true);
}

