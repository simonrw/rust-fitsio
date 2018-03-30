FROM rust:1.24.1

RUN apt-get update && \
    apt-get install -y libcfitsio-dev pkg-config libclang-3.8-dev build-essential clang

VOLUME ["/project"]
WORKDIR "/project/fitsio"
