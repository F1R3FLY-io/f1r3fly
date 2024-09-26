use tree_sitter::{Parser, Tree};

pub fn parse_rholang_code(code: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_rholang::LANGUAGE.into())
        .expect("Error loading Rholang grammar");
    println!("Language {:?}", parser.language());
    parser.parse(code, None).expect("Failed to parse code")
}
