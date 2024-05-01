#!/bin/bash

cd rspace++/
rustup update
cargo build --profile dev -p rspace_plus_plus_rhotypes

# This is needed for debugging through VS Code 'Metals' extension on M2
# Make sure Docker is running
cargo install cross --git https://github.com/cross-rs/cross
rustup target add x86_64-apple-darwin
cross build --profile dev --target x86_64-apple-darwin -p rspace_plus_plus_rhotypes
