FROM rust:latest

RUN apt-get update && \
    apt-get -yq dist-upgrade && \
    apt-get install -yq --no-install-recommends \
        libcfitsio-dev \
        pkg-config \
        build-essential \
        clang \
        gdb \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

RUN rustup update && \
    rustup install stable && \
    rustup install nightly

VOLUME ["/project"]
WORKDIR "/project/fitsio"

RUN apt-get update && \
    apt-get install -yq --no-install-recommends \
        valgrind \
        && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
