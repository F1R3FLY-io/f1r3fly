#!/bin/bash

cd rspace++/
# cargo clean
cargo build --profile dev -p rspace_plus_plus_rhotypes
cargo install cross --git https://github.com/cross-rs/cross

# OS_TYPE=$(uname -s)
# ARCH_TYPE=$(uname -m)
# TARGET=""

# if [ "$ARCH_TYPE" == "x86_64" ]; then
# 	TARGET="x86_64-unknown-linux-gnu"
# elif [ "$ARCH_TYPE" == "arm64" ]; then
# 	TARGET="aarch64-unknown-linux-gnu"
# else
# 	echo "Unsupported architecture: $ARCH_TYPE"
# 	exit 1
# fi

AMD64_TARGET="x86_64-unknown-linux-gnu"
ARM64_TARGET="aarch64-unknown-linux-gnu"

cross build --profile dev --target $AMD64_TARGET -p rspace_plus_plus_rhotypes
cross build --profile dev --target $ARM64_TARGET -p rspace_plus_plus_rhotypes

AMD64_BUILD_ARTIFACTS_PATH="../rspace++/target/$AMD64_TARGET/debug"
ARM64_BUILD_ARTIFACTS_PATH="../rspace++/target/$ARM64_TARGET/debug"

AMD64_RELEASE_PATH="../rspace++/target/docker/amd64/debug"
ARM64_RELEASE_PATH="../rspace++/target/docker/arm64/debug"

mkdir -p "$AMD64_RELEASE_PATH"
mkdir -p "$ARM64_RELEASE_PATH"

cp -r "$AMD64_BUILD_ARTIFACTS_PATH"/* "$AMD64_RELEASE_PATH"/
cp -r "$ARM64_BUILD_ARTIFACTS_PATH"/* "$ARM64_RELEASE_PATH"/
