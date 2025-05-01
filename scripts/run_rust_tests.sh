#!/bin/bash

set -e

cd rspace++/
cargo test --release

cd ../rholang
cargo test --release

cd ../models
cargo test --release

cd ../crypto
cargo test --release

cd ../shared
cargo test --release