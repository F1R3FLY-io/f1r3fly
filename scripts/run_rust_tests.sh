#!/bin/bash

set -e

cd rspace++/
cargo test

cd ../rholang
cargo test
