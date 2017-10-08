# rust-fitsio

FFI wrapper around cfitsio in Rust


[![Join the chat at https://gitter.im/mindriot101/rust-fitsio](https://badges.gitter.im/mindriot101/rust-fitsio.svg)](https://gitter.im/mindriot101/rust-fitsio?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
[![Build Status](https://travis-ci.org/mindriot101/rust-fitsio.svg?branch=master)](https://travis-ci.org/mindriot101/rust-fitsio)

## Installation

_There is a problem with `cfitsio` version 3.410. Please ensure version
3.420+ is installed._

For the time being, it's best to stick to the development version from github.
The code is tested before being pushed and is relatively stable. Add this to
your `Cargo.toml` file:

```toml
[dependencies]
fitsio = { git = "https://github.com/mindriot101/rust-fitsio" }
```

If you want the latest release from `crates.io` then add the following:

```toml
[dependencies]
fitsio = "*"
```

Or pin a specific version:

```toml
[dependencies]
fitsio = "*"
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
fitsio = { version = "*", default-features = false, features = ["bindgen"] }
```


## Documentation

`fitsio` [![`fitsio` documentation](https://docs.rs/fitsio/badge.svg)](https://docs.rs/fitsio/)<br />
`fitsio-sys` [![`fitsio-sys` documentation](https://docs.rs/fitsio-sys/badge.svg)](https://docs.rs/fitsio-sys)<br />
`fitsio-sys-bindgen` [![`fitsio-sys-bindgen` documentation](https://docs.rs/fitsio-sys-bindgen/badge.svg)](https://docs.rs/fitsio-sys-bindgen)<br />

## Feature support

Supported features of the underlying `cfitsio` library that _are_ available in `fitsio` are detailed in [this tracking issue](https://github.com/mindriot101/rust-fitsio/issues/15). If a particular function is not implemented in `fitsio`, then the underlying `fitsfile` pointer can be accessed through an unsafe API.

## Examples

Open a fits file

```rust
let f = fitsio::FitsFile::open("test.fits");
```

Accessing the underlying `fitsfile` object

```rust
extern crate fitsio;
extern crate fitsio_sys;

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
