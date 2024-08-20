fn main() {
  cc::Build::new()
    .file("src/main/tree_sitter/src/parser.c") // Update this path to the correct location of parser.c
    .compile("tree-sitter-rholang");
}