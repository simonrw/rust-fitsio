#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

for toml in $(find . -maxdepth 2 -name "Cargo.toml"); do
    cargo test --manifest-path $toml
done
