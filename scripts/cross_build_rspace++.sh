#!/bin/bash

cd rspace++/
cargo build --release -p rspace_plus_plus_rhotypes

# This is needed for debugging through VS Code 'Metals' extension
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target x86_64-apple-darwin -p rspace_plus_plus_rhotypes