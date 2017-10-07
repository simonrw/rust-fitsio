# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* cfitsio license

### Changed

* Include `SBYTE_IMG`, `USHORT_IMG` and `ULONG_IMG` data types

### Removed

## [0.9.0] - 2017-07-15

### Added

* Created unified error type `fitsio::errors::Error`
* Official (i.e. tested) support for the extended filename syntax
* Implemented support for generating the ffi bindings "live" using `bindgen` [#34][pull-34]
* Support (unsafely) accessing the underlying `fitsfile` pointer [#32][pull-32]
* Implement resizing images [#31][pull-31]

### Changed

* Removed _most_ unneeded `unwrap`s from the code
* Simplified the implementation of `buf_to_string`
* Include image data type in hdu info struct

### Removed

Nothing

[Unreleased]: https://github.com/mindriot101/rust-fitsio/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/mindriot101/rust-fitsio/compare/v0.8.0...v0.9.0
[pull-34]: https://github.com/mindriot101/rust-fitsio/pull/34
[pull-32]: https://github.com/mindriot101/rust-fitsio/pull/32
[pull-31]: https://github.com/mindriot101/rust-fitsio/pull/31
