#!/bin/bash

AMD64_TARGET="x86_64-unknown-linux-gnu"
ARM64_TARGET="aarch64-unknown-linux-gnu"

RSPACE_PLUS_PLUS_AMD64_BUILD_ARTIFACTS_PATH="target/$AMD64_TARGET/debug"
RSPACE_PLUS_PLUS_ARM64_BUILD_ARTIFACTS_PATH="target/$ARM64_TARGET/debug"

RHOLANG_AMD64_BUILD_ARTIFACTS_PATH="target/$AMD64_TARGET/debug"
RHOLANG_ARM64_BUILD_ARTIFACTS_PATH="target/$ARM64_TARGET/debug"

RUST_LIBRARIES_AMD64_RELEASE_PATH="rust_libraries/docker/debug/amd64"
RUST_LIBRARIES_ARM64_RELEASE_PATH="rust_libraries/docker/debug/arm64"

set -e

mkdir -p "$RUST_LIBRARIES_AMD64_RELEASE_PATH"
mkdir -p "$RUST_LIBRARIES_ARM64_RELEASE_PATH"

cargo install cross --git https://github.com/cross-rs/cross

cd rspace++/
cross build --profile dev --target $AMD64_TARGET -p rspace_plus_plus_rhotypes
cross build --profile dev --target $ARM64_TARGET -p rspace_plus_plus_rhotypes

cp -r "$RSPACE_PLUS_PLUS_AMD64_BUILD_ARTIFACTS_PATH"/librspace_plus_plus_rhotypes.* "../$RUST_LIBRARIES_AMD64_RELEASE_PATH"/
cp -r "$RSPACE_PLUS_PLUS_ARM64_BUILD_ARTIFACTS_PATH"/librspace_plus_plus_rhotypes.* "../$RUST_LIBRARIES_ARM64_RELEASE_PATH"/

cd ../rholang/
cross build --profile dev --target $AMD64_TARGET -p rholang
cross build --profile dev --target $ARM64_TARGET -p rholang

cp -r "$RHOLANG_AMD64_BUILD_ARTIFACTS_PATH"/librholang.* "../$RUST_LIBRARIES_AMD64_RELEASE_PATH"/
cp -r "$RHOLANG_ARM64_BUILD_ARTIFACTS_PATH"/librholang.* "../$RUST_LIBRARIES_ARM64_RELEASE_PATH"/
