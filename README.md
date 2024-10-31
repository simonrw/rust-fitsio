# rust-fitsio

FFI wrapper around cfitsio in Rust

[![Join the chat at https://gitter.im/simonrw/rust-fitsio](https://badges.gitter.im/simonrw/rust-fitsio.svg)](https://gitter.im/rust-fitsio/Lobby?utm_source=share-link&utm_medium=link&utm_campaign=share-link)
[![Build Status](https://travis-ci.org/simonrw/rust-fitsio.svg?branch=master)](https://travis-ci.org/simonrw/rust-fitsio)
[![Coverage Status](https://coveralls.io/repos/github/simonrw/rust-fitsio/badge.svg?branch=main)](https://coveralls.io/github/simonrw/rust-fitsio?branch=main)

## Platform support

| Platform      | Support level |
| ------------- | ------------- |
| Linux arm     | Tier 1        |
| Linux x86_64  | Tier 1        |
| macos x86_64  | Tier 1        |
| Linux arm64   | Tier 2        |
| Linux i386    | Tier 2        |
| macos arm64   | Tier 2        |
| Windows msys2 | Tier 3        |
| Windows msvc  | -             |

Where the tiers refer to:

- Tier 1: guaranteed to work, tested in CI
- Tier 2: should work but not tested by CI
- Tier 3: may work, and not tested by CI

## MSRV

The minimal version of rust we support is 1.58.0.

## Installation

`fitsio` supports versions of `cfitsio >= 3.37`.

`cfitsio` must be compiled with `reentrant` support (making it
thread-safe) if it is to be compiled with the `--enable-reentrant` flag
passed to `configure`. This affects developers of this library as the
tests by default are run in parallel.

For example on a mac with homebrew, install `cfitsio` with:

```sh
brew install cfitsio --with-reentrant
```

Alternatively, it is possible to automatically have `cargo` automatically
compile `cfitsio` from source. To do this, you are required to have a C
compiler, autotools (to run the `configure` script) and make (to run the
`Makefile`). This functionality is made available with the `fitsio-src` feature:

```sh
cargo build --features fitsio-src
```

For the time being, it's best to stick to the development version from
github. The code is tested before being pushed and is relatively
stable. Add this to your `Cargo.toml` file:

```toml,no_sync
[dependencies]
fitsio = { git = "https://github.com/simonrw/rust-fitsio" }
```

If you want the latest release from `crates.io` then add the following:

```toml
[dependencies]
fitsio = "*"
```

Or pin a specific version:

```toml
[dependencies]
fitsio = "0.21.6"
```

This repository contains `fitsio-sys-bindgen` which generates the C
wrapper using `bindgen` at build time. This requires clang to build, and
as this is likely to not be available in general, I do not recommend
using it. It is contained here but is not actively developed, and
untested. Use at your own peril. To opt in to building with `bindgen`,
compile as:

```sh
cargo build --no-default-features --features bindgen
```

or use from your `Cargo.toml` as such:

```toml
[dependencies]
fitsio = "0.21.6"
```

## Documentation

`fitsio` [![`fitsio` documentation](https://docs.rs/fitsio/badge.svg)](https://docs.rs/fitsio/)<br />
`fitsio-sys` [![`fitsio-sys` documentation](https://docs.rs/fitsio-sys/badge.svg)](https://docs.rs/fitsio-sys)<br />
`fitsio-sys-bindgen` [![`fitsio-sys-bindgen` documentation](https://docs.rs/fitsio-sys-bindgen/badge.svg)](https://docs.rs/fitsio-sys-bindgen)<br />

## Feature support

Supported features of the underlying `cfitsio` library that _are_ available in `fitsio` are detailed in [this tracking issue](https://github.com/simonrw/rust-fitsio/issues/15). If a particular function is not implemented in `fitsio`, then the underlying `fitsfile` pointer can be accessed through an unsafe API.

## Examples

Open a fits file

```rust
let f = fitsio::FitsFile::open("test.fits");
```

Accessing the underlying `fitsfile` object

```rust
fn main() {
    let filename = "../testdata/full_example.fits";
    let fptr = fitsio::FitsFile::open(filename).unwrap();

    /* Find out the number of HDUs in the file */
    let mut num_hdus = 0;
    let mut status = 0;

    unsafe {
        let fitsfile = fptr.as_raw();

        /* Use the unsafe fitsio-sys low level library to call a function that is possibly not
       implemented in this crate */
        fitsio_sys::ffthdu(fitsfile, &mut num_hdus, &mut status);
    }
    assert_eq!(num_hdus, 2);
}
```

## Development

See [./CONTRIBUTING.md](./CONTRIBUTING.md)
