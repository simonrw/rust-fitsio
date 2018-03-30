# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* (`fitsio`) add `ndarray` support [#92](https://github.com/mindriot101/rust-fitsio/pull/92)
* (`fitsio`) add lots of documentation

### Changed

* (`fitsio`) add long cfitsio function names internally [#88](https://github.com/mindriot101/rust-fitsio/pull/88)

### Removed

## [0.13.0] - 2018-03-10

### Addded

* (`fitsio`) add `primary_hdu` method [#77](https://github.com/mindriot101/rust-fitsio/pull/77)
* (`fitsio`) add pretty-printing support [#83](https://github.com/mindriot101/rust-fitsio/pull/83)
* (`fitsio`) add `row` method to read single row. This allows the user to declare a custom struct representing the row values [#86](https://github.com/mindriot101/rust-fitsio/pull/86)

### Changed

* (`fitsio`) **BREAKING CHANGE**: inverted the order of image/region axes to match the C row-major convention [#59](https://github.com/mindriot101/rust-fitsio/pull/59)
* (`fitsio`) **BREAKING CHANGE**: all ranges are now exclusive of the upper value, matching Rust's default behaviour [#61](https://github.com/mindriot101/rust-fitsio/pull/61)
* (`fitsio`) `WritesKey::write_key` now accepts &str's as well as `String`s [#80](https://github.com/mindriot101/rust-fitsio/pull/80)
* (`fitsio`) `create_image` and `create_table` take `Into<String>` [#81](https://github.com/mindriot101/rust-fitsio/pull/81)
* (`fitsio`) **BREAKING CHANGE**: changed ImageType variants to be more rusty [#84](https://github.com/mindriot101/rust-fitsio/pull/84)

### Removed

## [0.12.1] - 2018-02-23

_Fix issue with uploading crate_

### Addded
### Changed
### Removed

## [0.12.0] - 2018-02-23

### Addded

* (`fitsio`) add support for images which are not 2d
* (`fitsio`) add support for customising the primary HDU when a file is created (Thanks [@astrojhgu](https://users.rust-lang.org/u/astrojhgu) from the users discourse, for the suggestion)
* (`fitsio`) add support for non-2d images
* (`fitsio`) more friendly errors when image data is requested outside of the range of the image

### Changed

* (`fitsio`) change function arguments that were previously `start` and `end` to a `range` `Range<usize>`
* (`fitsio`) removed builder pattern for construction of `ColumnDataDescription`
* (`fitsio`) some implementations which were previously nightly-only have been used. These features involve the `cloned` method on an `Iterator`. Therefore the version of rust is therefore restricted by this.

### Removed

## [0.11.1] - 2017-11-24

### Addded
### Changed

* (`fitsio`) Fixed problem with writing string columns

### Removed

## [0.11.0] - 2017-11-24

### Addded
### Changed

* (`fitsio`) Updated the documentation to be feature complete as of this version

### Removed

## [0.10.0] - 2017-11-07

### Added

* (`fitsio`) add `iter` [#46][pull-46]
* (`fitsio`) add `hdu_name`, `hdu_names`, `num_hdus`, and `delete` [#45][pull-45]
* (`fitsio`) add `copy_to` [#44][pull-44]
* (`fitsio`) add `insert_column`, `append_column`, and `delete_column` methods to `FitsHdu` [#43][pull-43]
* add contribution guide
* cfitsio license

### Changed

* (`fitsio`) **BREAKING CHANGE**: most methods require passing a mutable `FitsFile` to perform work
* (`fitsio`) Include `SBYTE_IMG`, `USHORT_IMG` and `ULONG_IMG` data types

### Removed

Nothing

## [0.9.0] - 2017-07-15

### Added

* (`fitsio`) Created unified error type `fitsio::errors::Error`
* (`fitsio`) Official (i.e. tested) support for the extended filename syntax
* (`fitsio`) Implemented support for generating the ffi bindings "live" using `bindgen` [#34][pull-34]
* (`fitsio`) Support (unsafely) accessing the underlying `fitsfile` pointer [#32][pull-32]
* (`fitsio`) Implement resizing images [#31][pull-31]

### Changed

* (`fitsio`) Removed _most_ unneeded `unwrap`s from the code
* (`fitsio`) Simplified the implementation of `buf_to_string`
* (`fitsio`) Include image data type in hdu info struct

### Removed

Nothing

[Unreleased]: https://github.com/mindriot101/rust-fitsio/compare/v0.11.1...HEAD
[0.9.0]: https://github.com/mindriot101/rust-fitsio/compare/v0.8.0...v0.9.0
[pull-34]: https://github.com/mindriot101/rust-fitsio/pull/34
[pull-32]: https://github.com/mindriot101/rust-fitsio/pull/32
[pull-31]: https://github.com/mindriot101/rust-fitsio/pull/31
[pull-43]: https://github.com/mindriot101/rust-fitsio/pull/43
[pull-44]: https://github.com/mindriot101/rust-fitsio/pull/44
[pull-45]: https://github.com/mindriot101/rust-fitsio/pull/45
[pull-46]: https://github.com/mindriot101/rust-fitsio/pull/46
[0.10.0]: https://github.com/mindriot101/rust-fitsio/compare/v0.9.0...v0.10.0
[0.11.0]: https://github.com/mindriot101/rust-fitsio/compare/v0.10.0...v0.11.0
[0.11.1]: https://github.com/mindriot101/rust-fitsio/compare/v0.10.0...v0.11.1
[0.12.0]: https://github.com/mindriot101/rust-fitsio/compare/v0.11.1...v0.12.0
[0.12.1]: https://github.com/mindriot101/rust-fitsio/compare/v0.12.0...v0.12.1
[0.13.0]: https://github.com/mindriot101/rust-fitsio/compare/v0.12.1...v0.13.0

---

vim: ft=markdown:textwidth=0:wrap:nocindent
