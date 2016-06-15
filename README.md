# rust-cfitsio

[![Join the chat at https://gitter.im/mindriot101/rust-cfitsio](https://badges.gitter.im/mindriot101/rust-cfitsio.svg)](https://gitter.im/mindriot101/rust-cfitsio?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

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
