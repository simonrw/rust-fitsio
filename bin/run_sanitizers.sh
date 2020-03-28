#!/bin/sh

set -e

main() {

    # Check the number of arguments. If the user supplies an argument, assume
    # they want to run a single sanitiser.
    if [[ $# -eq 0 ]]; then
        for sanitizer in address memory; do
            RUSTFLAGS="-Z sanitizer=$sanitizer" cargo +nightly run \
                --manifest-path fitsio/Cargo.toml \
                --example full_example \
                --target x86_64-unknown-linux-gnu || true
        done
    elif [[ $# -eq 1 ]]; then
        sanitizer=$1
        RUSTFLAGS="-Z sanitizer=$sanitizer" cargo +nightly run \
            --manifest-path fitsio/Cargo.toml \
            --example full_example \
            --target x86_64-unknown-linux-gnu
    else
        echo "Program usage: $0 [sanitiser]" >&2
        return 1
    fi
}

main "$@"
