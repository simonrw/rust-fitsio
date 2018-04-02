# FITSIO

`fitsio` is a [Rust] FFI wrapper around [`cfitsio`], enabling Rust users
to read and write astronomy-specific files.

## Links

* [Documentation]
* [Issue tracker]
* [Changelog]
* [Contributing guide]
* [News]

## Example

```rust
extern crate fitsio;

use std::error::Error;
use fitsio::FitsFile;

fn try_main() -> Result<(), Box<Error>> {
    let filename = "example.fits";
    let mut fptr = FitsFile::open(filename)?;
}

fn main() { try_main().unwrap() }
```

[`cfitsio`]: http://heasarc.gsfc.nasa.gov/fitsio/fitsio.html
[Rust]: https://rust-lang.org/
[Documentation]: https://docs.rs/fitsio
[Changelog]: https://github.com/mindriot101/rust-fitsio/blob/master/CHANGELOG.md
[Contributing guide]: https://github.com/mindriot101/rust-fitsio/blob/master/CONTRIBUTING.md
[Issue tracker]: https://github.com/mindriot101/rust-fitsio/issues
[News]: news/index.html
