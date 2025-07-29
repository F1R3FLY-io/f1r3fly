fn main() {
    // The tree-sitter parser is now provided by the f1r3fly-tree-sitter-rholang dependency
    // No need to generate or compile parser.c here since it's handled by the separate crate
    println!("cargo:rerun-if-changed=build.rs");
}

// This script checks if the  parser.c  file exists in the  src/main/tree_sitter/src  directory. If it doesn't exist, it runs the  tree-sitter generate  command to generate the parser.
// It then compiles the generated  parser.c  file using the  cc  crate.
// The  tree-sitter generate  command generates the parser based on the grammar file in the  src/main/tree_sitter/grammar.js  file.
// The  cc  crate is a build dependency that provides a simple C compiler wrapper. It is used to compile the generated  parser.c  file.
// The  build.rs  script is executed before the build process starts. It generates the parser if it doesn't exist and compiles it.
