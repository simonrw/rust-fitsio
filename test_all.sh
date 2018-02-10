#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

# Hack for OSX: we cannot run the tests in parallel
case $OSTYPE in
    darwin*)
        __TESTEXTRA=""
        ;;
    default)
        __TESTEXTRA=""
        ;;
esac

for toml in $(find . -maxdepth 2 -name "Cargo.toml"); do
    cargo test --manifest-path $toml ${__TESTEXTRA}
done
