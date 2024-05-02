#!/bin/bash

cd rspace++/
cargo build --profile dev -p rspace_plus_plus_rhotypes

# This is needed for debugging through VS Code 'Metals' extension on M2
ARCH_TYPE=$(uname -m)
if [ "$ARCH_TYPE" == "arm64" ]; then
	cargo build --profile dev --target x86_64-apple-darwin -p rspace_plus_plus_rhotypes
fi
