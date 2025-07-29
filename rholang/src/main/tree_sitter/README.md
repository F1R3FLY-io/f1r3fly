# f1r3fly-tree-sitter-rholang

F1r3fly Rholang grammar for tree-sitter - incremental parsing for Rholang smart contracts.

## Overview

This crate provides a [tree-sitter](https://tree-sitter.github.io/) grammar for the Rholang programming language, enabling fast incremental parsing for editors, IDEs, and development tools.

Tree-sitter is a parser generator tool and an incremental parsing library that creates concrete syntax trees for source files and efficiently updates them as the source file is edited.

## Features

- **Incremental Parsing**: Fast, incremental parsing of Rholang source code
- **Editor Integration**: Support for syntax highlighting and code analysis
- **Rust Bindings**: Native Rust integration for F1r3fly ecosystem
- **Grammar Definition**: Complete Rholang language grammar

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
f1r3fly-tree-sitter-rholang = "0.1.0"
```

Basic usage:

```rust
use tree_sitter::Parser;
use f1r3fly_tree_sitter_rholang::language;

let mut parser = Parser::new();
parser.set_language(language()).expect("Error loading Rholang grammar");

let source_code = "new stdout(`rho:io:stdout`) in { stdout!(\"Hello, Rholang!\") }";
let tree = parser.parse(source_code, None).unwrap();
```

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## License

Licensed under the Apache License, Version 2.0.