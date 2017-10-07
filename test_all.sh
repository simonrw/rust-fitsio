#!/bin/bash
set -e

export CARGO_TARGET_DIR=target

# Hack for OSX: we cannot run the tests in parallel
case $OSTYPE in
    darwin*)
        __TESTEXTRA="-- --test-threads 1"
        ;;
    default)
        __TESTEXTRA=""
        ;;
esac

for toml in $(find . -maxdepth 2 -name "Cargo.toml"); do
    echo $toml | grep -q bindgen || cargo test --manifest-path $toml ${__TESTEXTRA}
done
