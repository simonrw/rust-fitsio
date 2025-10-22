# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.21.9](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.8...fitsio-v0.21.9) - 2025-10-22

### Added

- allow use of CMake instead of Autotools for fitsio-sys ([#398](https://github.com/simonrw/rust-fitsio/pull/398))

- Option to use CMake instead of Autotools when building CFITSIO (`fitsio-src` feature + `src-cmake` feature)

## [0.21.8](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.7...fitsio-v0.21.8) - 2025-09-15

### Added

- set  public fitsio::fitfile::FitsFile::num_hdus function. ([#417](https://github.com/simonrw/rust-fitsio/pull/417))

### Other

- Remove locked flag ([#414](https://github.com/simonrw/rust-fitsio/pull/414))
- *(deps)* update criterion requirement from 0.6.0 to 0.7.0 in /fitsio in the cargo-packages group ([#412](https://github.com/simonrw/rust-fitsio/pull/412))
- Fix clippy lints to fix CI
- *(deps)* update criterion requirement from 0.5.1 to 0.6.0 in /fitsio in the cargo-packages group ([#403](https://github.com/simonrw/rust-fitsio/pull/403))
- Fix latest nightly clippy warnings

## [0.21.7](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.6...fitsio-v0.21.7) - 2025-01-02

### Other

- Include function to get cfitsio version ([#379](https://github.com/simonrw/rust-fitsio/pull/379))
- Fix macos tests ([#380](https://github.com/simonrw/rust-fitsio/pull/380))

## [0.21.6](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.5...fitsio-v0.21.6) - 2024-10-31

### Added

- add `FitsFile::file_path` method to access `FitsFile` file path ([#367](https://github.com/simonrw/rust-fitsio/pull/367))

### Other

- Allow needless-lifetimes ([#363](https://github.com/simonrw/rust-fitsio/pull/363))

## [0.21.5](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.4...fitsio-v0.21.5) - 2024-08-06

### Added
- add vector type support ([#330](https://github.com/simonrw/rust-fitsio/pull/330))

### Other
- *(deps)* update ndarray requirement from 0.15.0 to 0.16.0 in /fitsio ([#347](https://github.com/simonrw/rust-fitsio/pull/347))

## [0.21.4](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.3...fitsio-v0.21.4) - 2024-07-26

### Added
- support reading boolean columns ([#342](https://github.com/simonrw/rust-fitsio/pull/342))

## [0.21.3](https://github.com/simonrw/rust-fitsio/compare/fitsio-v0.21.2...fitsio-v0.21.3) - 2024-07-26

### Added
- POC header comment API design ([#332](https://github.com/simonrw/rust-fitsio/pull/332))

### Other
- Add TSHORT types for i16 and u16
- Add clippy feature
- Merge branch 'main' into main
- Pin minimal serde version
- Update criterion requirement from 0.3.5 to 0.5.1 in /fitsio
- Fix nightly compile errors
- Use TBYTE for *8 reads ([#277](https://github.com/simonrw/rust-fitsio/pull/277))
- Allow/fix warnings that are blocking CI
