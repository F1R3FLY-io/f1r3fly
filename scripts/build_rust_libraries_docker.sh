#!/bin/bash

set -e

mkdir -p ./rust_libraries/debug

# RSpace++

cd rspace++/
cargo build --profile dev -p rspace_plus_plus_rhotypes
cargo install cross --git https://github.com/cross-rs/cross

AMD64_TARGET="x86_64-unknown-linux-gnu"
ARM64_TARGET="aarch64-unknown-linux-gnu"

cross build --profile dev --target $AMD64_TARGET -p rspace_plus_plus_rhotypes
cross build --profile dev --target $ARM64_TARGET -p rspace_plus_plus_rhotypes

AMD64_BUILD_ARTIFACTS_PATH="../rust_libraries/$AMD64_TARGET/debug"
ARM64_BUILD_ARTIFACTS_PATH="../rust_libraries/$ARM64_TARGET/debug"

mkdir -p "$AMD64_BUILD_ARTIFACTS_PATH"
mkdir -p "$ARM64_BUILD_ARTIFACTS_PATH"

cp -r ./target/$AMD64_TARGET/debug/librspace_plus_plus_rhotypes.* $AMD64_BUILD_ARTIFACTS_PATH
cp -r ./target/$ARM64_TARGET/debug/librspace_plus_plus_rhotypes.* $ARM64_BUILD_ARTIFACTS_PATH

AMD64_RELEASE_PATH="../rust_libraries/docker/amd64/debug"
ARM64_RELEASE_PATH="../rust_libraries/docker/arm64/debug"

mkdir -p "$AMD64_RELEASE_PATH"
mkdir -p "$ARM64_RELEASE_PATH"

cp -r "$AMD64_BUILD_ARTIFACTS_PATH"/* "$AMD64_RELEASE_PATH"/
cp -r "$ARM64_BUILD_ARTIFACTS_PATH"/* "$ARM64_RELEASE_PATH"/

# Rholang

cd ../rholang/
cargo build --profile dev -p rholang

AMD64_TARGET="x86_64-unknown-linux-gnu"
ARM64_TARGET="aarch64-unknown-linux-gnu"

cross build --profile dev --target $AMD64_TARGET -p rholang
cross build --profile dev --target $ARM64_TARGET -p rholang

AMD64_BUILD_ARTIFACTS_PATH="../rust_libraries/$AMD64_TARGET/debug"
ARM64_BUILD_ARTIFACTS_PATH="../rust_libraries/$ARM64_TARGET/debug"

mkdir -p "$AMD64_BUILD_ARTIFACTS_PATH"
mkdir -p "$ARM64_BUILD_ARTIFACTS_PATH"

cp -r ./target/$AMD64_TARGET/debug/librholang.* $AMD64_BUILD_ARTIFACTS_PATH
cp -r ./target/$ARM64_TARGET/debug/librholang.* $ARM64_BUILD_ARTIFACTS_PATH

AMD64_RELEASE_PATH="../rust_libraries/docker/amd64/debug"
ARM64_RELEASE_PATH="../rust_libraries/docker/arm64/debug"

mkdir -p "$AMD64_RELEASE_PATH"
mkdir -p "$ARM64_RELEASE_PATH"

cp -r "$AMD64_BUILD_ARTIFACTS_PATH"/* "$AMD64_RELEASE_PATH"/
cp -r "$ARM64_BUILD_ARTIFACTS_PATH"/* "$ARM64_RELEASE_PATH"/
