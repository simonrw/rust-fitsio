#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

for toml in $(find . -maxdepth 2 -name "Cargo.toml"); do
    echo $toml | grep -q bindgen || cargo test --manifest-path $toml
done
