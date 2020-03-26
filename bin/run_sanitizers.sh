#!/bin/sh

set -e

for sanitizer in address memory; do
    RUSTFLAGS="-Z sanitizer=$sanitizer" cargo +nightly run \
        --manifest-path fitsio/Cargo.toml \
        --example full_example \
        --target x86_64-unknown-linux-gnu || true
done
