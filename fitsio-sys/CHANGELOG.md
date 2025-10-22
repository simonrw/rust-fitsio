# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.5.7](https://github.com/simonrw/rust-fitsio/compare/fitsio-sys-v0.5.6...fitsio-sys-v0.5.7) - 2025-10-22

### Added

- allow use of CMake instead of Autotools for fitsio-sys ([#398](https://github.com/simonrw/rust-fitsio/pull/398))

- Option to use CMake instead of Autotools when building CFITSIO (`fitsio-src` feature + `src-cmake` feature)

## [0.5.6](https://github.com/simonrw/rust-fitsio/compare/fitsio-sys-v0.5.5...fitsio-sys-v0.5.6) - 2025-09-15

### Other

- *(deps)* update bindgen requirement from 0.71 to 0.72 in /fitsio-sys in the cargo-packages group ([#407](https://github.com/simonrw/rust-fitsio/pull/407))
- Generate and provide fitsio "long names" from longnam.h ([#382](https://github.com/simonrw/rust-fitsio/pull/382))

## [0.5.5](https://github.com/simonrw/rust-fitsio/compare/fitsio-sys-v0.5.4...fitsio-sys-v0.5.5) - 2025-01-02

### Other

- Simplify build.rs of fitsio-sys ([#377](https://github.com/simonrw/rust-fitsio/pull/377))
- Include function to get cfitsio version ([#379](https://github.com/simonrw/rust-fitsio/pull/379))
- *(deps)* update bindgen requirement from 0.70 to 0.71 in /fitsio-sys in the cargo-packages group (#372)

## [0.5.4](https://github.com/simonrw/rust-fitsio/compare/fitsio-sys-v0.5.3...fitsio-sys-v0.5.4) - 2024-10-31

### Other

- *(deps)* update bindgen requirement from 0.69 to 0.70 in /fitsio-sys in the cargo-packages group ([#357](https://github.com/simonrw/rust-fitsio/pull/357))

## [0.5.3](https://github.com/simonrw/rust-fitsio/compare/fitsio-sys-v0.5.2...fitsio-sys-v0.5.3) - 2024-07-26

### Other
- Fix clippy warnings
- Update bindgen requirement from 0.66 to 0.69 in /fitsio-sys
- Include new changelog for fitsio-sys
- Provide aliases to function "long names".
- Update bindgen requirement from 0.63 to 0.66 in /fitsio-sys
### Added

* Added aliases for cfitsio short names ([#258](https://github.com/simonrw/rust-fitsio/pull/258))

### Changed
### Removed
