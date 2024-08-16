#!/bin/bash

set -e

cd rspace++/
cargo build --profile dev -p rspace_plus_plus_rhotypes

cd ../models
cargo build --profile dev -p models

cd ../rholang
cargo build --profile dev -p rholang

cd ../crypto
cargo build --profile dev -p crypto

# This is needed for debugging through VS Code 'Metals' extension on M2
# ARCH_TYPE=$(uname -m)
# if [ "$ARCH_TYPE" == "arm64" ]; then
# 	cargo build --profile dev --target x86_64-apple-darwin -p rspace_plus_plus_rhotypes
# fi
