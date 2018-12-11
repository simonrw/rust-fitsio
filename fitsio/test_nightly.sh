#!/bin/bash

set -e

IMAGE=srwalker101/rust-fitsio

main() {
    docker run --rm -v $(pwd)/..:/project ${IMAGE} cargo +nightly test --lib
}

main
