# rust-cfitsio

[![Build Status](https://travis-ci.org/mindriot101/rust-cfitsio.svg?branch=master)](https://travis-ci.org/mindriot101/rust-cfitsio)
[![Coverage Status](https://coveralls.io/repos/github/mindriot101/rust-cfitsio/badge.svg?branch=master)](https://coveralls.io/github/mindriot101/rust-cfitsio?branch=master)

FFI wrapper around cfitsio in Rust



## Api roadmap

### Images

* Change HDU
* Read image data
* Get image metadata

### Tables

## Examples

Open a fits file

```rust
let f = fitsio::FitsFile::open("test.fits");
```
