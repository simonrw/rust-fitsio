#!/bin/sh

set -e

main() {

    # Assume we are in a subdirectory of `fitsio` and move upwards
    while [ ! -f fitsio/Cargo.toml ]; do
        if [[ $PWD = "/" ]]; then
            # Top of the file system
            echo "Cannot find fitsio dir" >&2
            exit 1
        fi

        cd ..
    done

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
