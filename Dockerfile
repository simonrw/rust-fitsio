FROM rust:1.60.0-slim-buster

RUN apt-get update && \
    apt-get -yq dist-upgrade && \
    apt-get install -yq --no-install-recommends \
        libcfitsio-dev \
        pkg-config \
        libclang-3.8-dev \
        build-essential \
        clang \
        gdb \
        python3 \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN rustup update && \
    rustup install stable && \
    rustup install nightly && \
    rustup component add clippy --toolchain stable-x86_64-unknown-linux-gnu

VOLUME ["/project"]
WORKDIR "/project"

RUN apt-get update && \
    apt-get install -yq --no-install-recommends \
        valgrind \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
