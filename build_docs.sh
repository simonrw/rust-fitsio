#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

for toml in $(find . -maxdepth 2 -name "Cargo.toml"); do
    cargo doc --no-deps --manifest-path $toml
done
