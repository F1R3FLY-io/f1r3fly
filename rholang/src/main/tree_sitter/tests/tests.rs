#[cfg(test)]
mod tests {
    use tree_sitter::{Parser, Tree};
    use tree_sitter_rholang::language; // Import the language function from the generated bindings

    // This function parses the provided Rholang code and returns the Tree
    fn parse_rholang_code(code: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&language()).expect("Error loading Rholang grammar");
        let tree = parser.parse(code, None).expect("Failed to parse code");
        tree // Return the Tree itself
    }

    #[test]
    fn test_hello_world_contract() {
        let code = r#"new helloWorld in { helloWorld!("Hey, deploy") }"#;

        let tree = parse_rholang_code(code); // Parse the code and get the Tree
        println!("{}", tree.root_node().to_sexp());
        let root = tree.root_node(); // Get the root node from the Tree
        assert_eq!(root.kind(), "source_file"); // Ensure the root node is of type "source_file"

        // Navigate the syntax tree to find specific nodes and ensure they match expectations
        let first_child = root.child(0).expect("Expected a child node");

        // This is where the test checks if the first child is of the expected kind
        assert_eq!(first_child.kind(), "proc16");

        // Further checks on the parsed structure, such as method calls or other constructs
        let proc_call = first_child.child(0).expect("Expected a procedure call");
        assert_eq!(proc_call.kind(), "ground");

        // let identifier = proc_call.child(0).expect("Expected an identifier");
        // assert_eq!(identifier.kind(), "uri_literal");
        // assert_eq!(identifier.utf8_text(code.as_bytes()).unwrap(), "helloWorld");
        //
        // let argument = proc_call.child(1).expect("Expected an argument");
        // assert_eq!(argument.kind(), "string_literal");
        // assert_eq!(argument.utf8_text(code.as_bytes()).unwrap(), r#""Hey, deploy""#);
    }
}
