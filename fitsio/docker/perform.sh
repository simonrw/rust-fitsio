#!/bin/sh

set -eux

analyse_log() {
    grep -q cfitsio /tmp/logfile
}

reset() {
    cargo clean
}

test_default() {
    reset
    cargo test -j 1 --verbose -- --test-threads 1 2>&1 | tee /tmp/logfile
    analyse_log
}

test_bindgen() {
    reset
    cargo test -j 1 --verbose --features bindgen --no-default-features -- --test-threads 1 2>&1 | tee /tmp/logfile
    analyse_log
}

main() {
    test_default
    test_bindgen
}

main
