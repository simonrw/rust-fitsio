FROM alpine:3.7

RUN echo '@testing http://dl-cdn.alpinelinux.org/alpine/edge/testing' >> /etc/apk/repositories

RUN apk update && apk add --upgrade apk-tools@testing

RUN apk add \
    gcc \
    musl-dev \
    curl \
    git \
    rust@testing \
    cargo@testing \
    clang-dev \
    make

RUN curl -LO http://heasarc.gsfc.nasa.gov/FTP/software/fitsio/c/cfitsio3420.tar.gz && \
    tar xvf cfitsio3420.tar.gz && \
    cd cfitsio && \
    ./configure --prefix /usr/local && \
    make shared install && \
    cd .. && \
    rm -r cfitsio3420.tar.gz cfitsio

WORKDIR /fitsiotest/fitsio
VOLUME ["/fitsiotest"]
CMD ["sh", "docker/perform.sh"]
