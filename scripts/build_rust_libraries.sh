#!/bin/bash

set -e

mkdir -p ./rust_libraries

cd rspace++/
cargo build --profile dev -p rspace_plus_plus_rhotypes
cp -r ./target/debug/librspace_plus_plus_rhotypes.* ../rust_libraries

cd ../models
cargo build --profile dev -p models

cd ../rholang
cargo build --profile dev -p rholang
cp -r ./target/debug/librholang.* ../rust_libraries

cd ../crypto
cargo build --profile dev -p crypto

# This is needed for debugging through VS Code 'Metals' extension on M2
# ARCH_TYPE=$(uname -m)
# if [ "$ARCH_TYPE" == "arm64" ]; then
# 	cargo build --profile dev --target x86_64-apple-darwin -p rspace_plus_plus_rhotypes
# fi
