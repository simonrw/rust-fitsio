# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
* Support for vector data-types in reading and writing tables.
### Changed
### Removed

## [0.21.2]
### Added

* Support for n-d arrays (rather than just 2-d arrays) [#233](https://github.com/simonrw/rust-fitsio/pull/233)

### Changed

* Updated `bindgen` to version 0.63.0
* The `bindgen` feature is now implemented in `fitsio-sys` rather than `fitsio`. This is done as the bindings generation is really part of `fitsio-sys` rather than `fitsio`. There should be no external difference noticed.

### Removed

* `fitsio-sys-bindgen` is no longer used. It has been merged with `fitsio-sys`.

## [0.21.1]
### Added
### Changed

* `fitsio`: update versions of `fitsio-sys` and `fitsio-sys-bindgen` to pick up latest changes
* `fitsio-sys`: do not specify the version for msys2 since it is not well represented

### Removed

## [0.21.0]
### Added

* The abillity to create a `FitsFile` struct from a `fitsio_sys::fitsfile` pointer [#195](https://github.com/simonrw/rust-fitsio/pull/195)
* Support for boolean header card values
* `fitsio-sys` (whichever feature is used) is exposed as `fitsio::sys` to make sure that only one crate that links to the system library exists [#195](https://github.com/simonrw/rust-fitsio/pull/174)

### Changed

* **BREAKING CHANGE** the previously public field `filename` on `FitsFile` has been removed, since:
  * it is now optional,
  * it is not used internally for much, and
  * it is state that does not need to be part of `fitsio`. [#195](https://github.com/simonrw/rust-fitsio/pull/174)
* Some more types are deriving `Eq` thanks to a clippy lint
* Fixed broken tests on m1 macos [#174](https://github.com/simonrw/rust-fitsio/pull/174)
* Minimum cfitsio version of 3.37 specified for compilation [#184](https://github.com/simonrw/rust-fitsio/pull/184)

## [0.20.0]
### Added

* Initial support for windows msys2 [#150](https://github.com/simonrw/rust-fitsio/pull/150)

### Changed

* Specify version dependencies more specifically which should make installations more reliable, see [this post](https://users.rust-lang.org/t/psa-please-specify-precise-dependency-versions-in-cargo-toml/71277/9) [#151](https://github.com/simonrw/rust-fitsio/pull/151)
* Pin `ndarray` against versions 0.15.*. This prevents downstream users from
  having interoperability problems when `ndarray` updates to 0.16.0.

## [0.19.0]
### Added

* Added support for 16 bit images and tables thanks @emaadparacha
  [#148](https://github.com/simonrw/rust-fitsio/pull/148).

### Changed

* arm32 architectures are tested on CI so consider arm32 a tier 1 platform -
  `fitsio` will not release a new version without arm32 tests passing on CI.

### Removed

## [0.18.0]
### Added

* Added support for arm32 architectures (armv7-unknown-linux-gnueabihf - e.g. Raspberry Pi). This is not tested on CI so it's not maintained as such.

### Changed
### Removed

## [0.17.0]

### Added

* Added support for compiling `cfitsio` from source rather than using the
  bundled version. This requires the installer needing to include the
  dependencies that are required for installing `cfitsio`, namely `make` and
  `gcc`, thanks @cjordan. [#130](https://github.com/simonrw/rust-fitsio/pull/130)

### Changed

* (`fitsio-derive`) corrected path to `FitsHdu`, release v0.2.0

### Removed

## [0.16.0]

### Added

* (`fitsio`) expose the HDU number (`number`) in the `FitsHdu` struct
* (`fitsio`) handle the 'X' fits column data type. This may not match the
  behaviour with cfitsio, but that behaviour is also seemingly complicated and
  suboptimal; see [#122](https://github.com/simonrw/rust-fitsio/pull/122)
  for discussion.

### Changed

* (`fitsio`) many small things, including using Rust 2018 edition.

### Removed

## [0.15.0]

### Added

### Changed

* (`fitsio`) **BREAKING CHANGE**: implementing underlying raw `fitsfile` pointer as a `std::ptr::NonNull`, meaning the pointer is guaranteed to never be null, increasing the safety of the API. The breaking change is that more methods require a mutable (exclusive) reference (due to the method of converting the `NonNull` to mutable pointer, required by some lower level methods). The upside is that the `FitsFile` object *should* have exclusive access as it wraps the state of the fits file on disk. Safe concurrent (though not parallel) access is given by the `threadsafe` method.

### Removed


## [0.14.1]

### Added

* (`fitsio`) add boolean column type
* (`fitsio`) add threadsafe version of a `FitsFile` [#99](https://github.com/simonrw/rust-fitsio/pull/99)
* (`fitsio`) add benchmarking [#98](https://github.com/simonrw/rust-fitsio/pull/98)
* (`fitsio`) add support for writing all integer header keys

### Changed

* (`fitsio`) fix errors with the system allocator, ensuring the package will run on the latest nightly and beta compilers [#100](https://github.com/simonrw/rust-fitsio/pull/100)

### Removed

## [0.14.0] - 2018-04-21

### Added

* (`fitsio`) add `overwrite` method to `NewFitsFile` [#94](https://github.com/simonrw/rust-fitsio/pull/94)
* (`fitsio`) add `ndarray` support [#92](https://github.com/simonrw/rust-fitsio/pull/92)
* (`fitsio`) add lots of documentation

### Changed

* (`fitsio`) **BREAKING CHANGE**: move code into more logical submodule arrangement [#95](https://github.com/simonrw/rust-fitsio/pull/95)
* (`fitsio`) add long cfitsio function names internally [#88](https://github.com/simonrw/rust-fitsio/pull/88)

### Removed

## [0.13.0] - 2018-03-10

### Addded

* (`fitsio`) add `row` method to read single row. This allows the user to declare a custom struct representing the row values [#86](https://github.com/simonrw/rust-fitsio/pull/86)
* (`fitsio`) add pretty-printing support [#83](https://github.com/simonrw/rust-fitsio/pull/83)
* (`fitsio`) add `primary_hdu` method [#77](https://github.com/simonrw/rust-fitsio/pull/77)

### Changed

* (`fitsio`) **BREAKING CHANGE**: changed ImageType variants to be more rusty [#84](https://github.com/simonrw/rust-fitsio/pull/84)
* (`fitsio`) `create_image` and `create_table` take `Into<String>` [#81](https://github.com/simonrw/rust-fitsio/pull/81)
* (`fitsio`) `WritesKey::write_key` now accepts &str's as well as `String`s [#80](https://github.com/simonrw/rust-fitsio/pull/80)
* (`fitsio`) **BREAKING CHANGE**: all ranges are now exclusive of the upper value, matching Rust's default behaviour [#61](https://github.com/simonrw/rust-fitsio/pull/61)
* (`fitsio`) **BREAKING CHANGE**: inverted the order of image/region axes to match the C row-major convention [#59](https://github.com/simonrw/rust-fitsio/pull/59)

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

[Unreleased]: https://github.com/simonrw/rust-fitsio/compare/v0.21.2...HEAD
[0.9.0]: https://github.com/simonrw/rust-fitsio/compare/v0.8.0...v0.9.0
[pull-34]: https://github.com/simonrw/rust-fitsio/pull/34
[pull-32]: https://github.com/simonrw/rust-fitsio/pull/32
[pull-31]: https://github.com/simonrw/rust-fitsio/pull/31
[pull-43]: https://github.com/simonrw/rust-fitsio/pull/43
[pull-44]: https://github.com/simonrw/rust-fitsio/pull/44
[pull-45]: https://github.com/simonrw/rust-fitsio/pull/45
[pull-46]: https://github.com/simonrw/rust-fitsio/pull/46
[0.10.0]: https://github.com/simonrw/rust-fitsio/compare/v0.9.0...v0.10.0
[0.11.0]: https://github.com/simonrw/rust-fitsio/compare/v0.10.0...v0.11.0
[0.11.1]: https://github.com/simonrw/rust-fitsio/compare/v0.10.0...v0.11.1
[0.12.0]: https://github.com/simonrw/rust-fitsio/compare/v0.11.1...v0.12.0
[0.12.1]: https://github.com/simonrw/rust-fitsio/compare/v0.12.0...v0.12.1
[0.13.0]: https://github.com/simonrw/rust-fitsio/compare/v0.12.1...v0.13.0
[0.14.0]: https://github.com/simonrw/rust-fitsio/compare/v0.13.0...v0.14.0
[0.14.1]: https://github.com/simonrw/rust-fitsio/compare/v0.14.0...v0.14.1
[0.15.0]: https://github.com/simonrw/rust-fitsio/compare/v0.14.1...v0.15.0
[0.16.0]: https://github.com/simonrw/rust-fitsio/compare/v0.15.0...v0.16.0
[0.17.0]: https://github.com/simonrw/rust-fitsio/compare/v0.16.0...v0.17.0
[0.18.0]: https://github.com/simonrw/rust-fitsio/compare/v0.17.0...v0.18.0
[0.19.0]: https://github.com/simonrw/rust-fitsio/compare/v0.18.0...v0.19.0
[0.20.0]: https://github.com/simonrw/rust-fitsio/compare/v0.19.0...v0.20.0
[0.21.0]: https://github.com/simonrw/rust-fitsio/compare/v0.20.0...v0.21.0
[0.21.1]: https://github.com/simonrw/rust-fitsio/compare/v0.21.0...v0.21.1
[0.21.2]: https://github.com/simonrw/rust-fitsio/compare/v0.21.1...v0.21.2

---

vim: ft=markdown:textwidth=0:nocindent
