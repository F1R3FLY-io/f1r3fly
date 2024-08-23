use std::process::Command;
use std::path::Path;

fn main() {
  let grammar_js_path = Path::new("src/main/tree_sitter/grammar.js");
  let parser_c_path = Path::new("src/main/tree_sitter/src/parser.c");

  // Ensure the grammar.js file exists before running tree-sitter generate
  if !grammar_js_path.exists() {
    panic!("`grammar.js` does not exist. Please ensure your grammar file is in the correct path.");
  }

  // If the parser.c file doesn't exist, run the tree-sitter generate command
  if !parser_c_path.exists() {
    println!("`parser.c` not found, running `tree-sitter generate`...");

    // Run the tree-sitter generate command
    let output = Command::new("tree-sitter")
      .arg("generate")
      .current_dir("src/main/tree_sitter") // Change to the correct directory
      .output()
      .expect("Failed to execute tree-sitter generate");

    // Check if the command was successful
    if !output.status.success() {
      panic!(
        "tree-sitter generate failed: {}",
        String::from_utf8_lossy(&output.stderr)
      );
    }

    // Verify if parser.c is generated
    if !parser_c_path.exists() {
      panic!("`parser.c` was not generated. Check your grammar or Tree-Sitter setup.");
    }
  }

  // Compile the generated parser.c file
  cc::Build::new()
    .file(&parser_c_path)
    .compile("tree-sitter-rholang");
}


// This script checks if the  parser.c  file exists in the  src/main/tree_sitter/src  directory. If it doesn't exist, it runs the  tree-sitter generate  command to generate the parser.
// It then compiles the generated  parser.c  file using the  cc  crate.
// The  tree-sitter generate  command generates the parser based on the grammar file in the  src/main/tree_sitter/grammar.js  file.
// The  cc  crate is a build dependency that provides a simple C compiler wrapper. It is used to compile the generated  parser.c  file.
// The  build.rs  script is executed before the build process starts. It generates the parser if it doesn't exist and compiles it.