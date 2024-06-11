#!/bin/bash

cd rspace++/
# cargo clean
# cargo build --profile dev -p rspace_plus_plus_rhotypes
cargo install cross --git https://github.com/cross-rs/cross

OS_TYPE=$(uname -s)
ARCH_TYPE=$(uname -m)
TARGET=""

if [ "$ARCH_TYPE" == "x86_64" ]; then
	TARGET="x86_64-unknown-linux-gnu"
elif [ "$ARCH_TYPE" == "arm64" ]; then
	TARGET="aarch64-unknown-linux-gnu"
else
	echo "Unsupported architecture: $ARCH_TYPE"
	exit 1
fi

cross build --profile dev --target $TARGET -p rspace_plus_plus_rhotypes
# cargo build --profile dev --target $TARGET -p rspace_plus_plus_rhotypes --locked

BUILD_ARTIFACTS_PATH="../rspace++/target/$TARGET/debug"
COMMON_RELEASE_PATH="../rspace++/target/docker/debug"

mkdir -p "$COMMON_RELEASE_PATH"
cp -r "$BUILD_ARTIFACTS_PATH"/* "$COMMON_RELEASE_PATH"/
