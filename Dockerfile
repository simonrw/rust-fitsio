FROM rust:1.60.0-slim-bullseye

RUN apt-get update && \
    apt-get -yq dist-upgrade && \
    apt-get install -yq --no-install-recommends \
        libcfitsio-dev \
        pkg-config \
        libclang-19-dev \
        build-essential \
        cmake \
        clang \
        gdb \
        python3 \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN rustup update && \
    rustup install stable && \
    rustup install nightly && \
    rustup component add --toolchain stable clippy

RUN cargo +stable install --locked cargo-nextest

VOLUME ["/project"]
WORKDIR "/project"

RUN apt-get update && \
    apt-get install -yq --no-install-recommends \
        valgrind \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
