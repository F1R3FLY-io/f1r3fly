use tree_sitter::{Parser, Tree};
use tree_sitter_rholang::language;

pub fn parse_rholang_code(code: &str) -> Tree {
  let mut parser = Parser::new();
  parser.set_language(&language()).expect("Error loading Rholang grammar");
  println!("Language {:?}", parser.language());
  parser.parse(code, None).expect("Failed to parse code")
}