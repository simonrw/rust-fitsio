//! `fitsio` - a thin wrapper around the [`cfitsio`][1] C library.
//!
//! * [HDU access](#hdu-access)
//! * [Header keys](#header-keys)
//! * [Reading file data](#reading-file-data)
//!     * [Images](#images)
//!     * [Tables](#tables)
//!
//! This library wraps the low level `cfitsio` bindings: [`fitsio-sys`][2] and provides a more
//! native experience for rust users.
//!
//! The main interface to a fits file is [`FitsFile`](struct.FitsFile.html). All file manipulation
//! and reading starts with this class.
//!
//! Opening a file:
//!
//! ```rust
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! use fitsio::FitsFile;
//!
//! // let filename = ...;
//! let fptr = FitsFile::open(filename).unwrap();
//! # }
//! ```
//!
//! Alternatively a new file can be created on disk with the companion method
//! [`create`](struct.FitsFile.html#method.create):
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use fitsio::FitsFile;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let _filename = tdir_path.join("test.fits");
//! # let filename = _filename.to_str().unwrap();
//! use fitsio::FitsFile;
//!
//! // let filename = ...;
//! let fptr = FitsFile::create(filename).unwrap();
//! # }
//! ```
//!
//! From this point, the current HDU can be queried and changed, or fits header cards can be read
//! or file contents can be read.
//!
//! ## HDU access
//!
//! HDU information belongs to the [`FitsHdu`](struct.FitsHdu.html) object. HDUs can be fetched by
//! `String`/`str` or integer (0-indexed). The HDU object contains information about the current
//! HDU:
//!
//! ```rust
//! # extern crate fitsio;
//! # extern crate fitsio_sys;
//! # use fitsio::FitsFile;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = FitsFile::open(filename).unwrap();
//! use fitsio_sys::HduType;
//! let hdu = fptr.hdu(0).unwrap();
//!
//! match hdu.hdu_type() {
//!     Ok(HduType::IMAGE_HDU) => println!("Found image"),
//!     Ok(HduType::BINARY_TBL) => println!("Found table"),
//!     _ => {},
//! }
//! # }
//! ```
//!
//! or fetching metadata about the current HDU:
//!
//! ```rust
//! # extern crate fitsio;
//! # extern crate fitsio_sys;
//! # use fitsio::{FitsFile, HduInfo};
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = FitsFile::open(filename).unwrap();
//! let hdu = fptr.hdu(0).unwrap();
//! // image HDU
//! if let HduInfo::ImageInfo { dimensions, shape } = hdu.info {
//!    println!("Image is {}-dimensional", dimensions);
//!    println!("Found image with shape {:?}", shape);
//! }
//! # let hdu = fptr.hdu("TESTEXT").unwrap();
//!
//! // tables
//! if let HduInfo::TableInfo { column_descriptions, num_rows, .. } = hdu.info {
//!     println!("Table contains {} rows", num_rows);
//!     println!("Table has {} columns", column_descriptions.len());
//! }
//! # }
//! ```
//!
//! The current HDU can be selected either by absolute number (0-indexed) or string-like:
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! fptr.change_hdu(1).unwrap();
//! assert_eq!(fptr.hdu_number(), 1);
//!
//! # fptr.change_hdu(0).unwrap();
//! fptr.change_hdu("TESTEXT").unwrap();
//! assert_eq!(fptr.hdu_number(), 1);
//! # }
//! ```
//!
//! ## Header keys
//!
//! Header keys are read through the [`read_key`](struct.FitsFile.html#method.read_key) function,
//! and is generic over types that implement the [`ReadsKey`](trait.ReadsKey.html) trait:
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! # {
//! let int_value: i64 = fptr.hdu(0).unwrap().read_key("INTTEST").unwrap();
//! # }
//!
//! // Alternatively
//! # {
//! let int_value = fptr.hdu(0).unwrap().read_key::<i64>("INTTEST").unwrap();
//! # }
//!
//! // Or let the compiler infer the types (if possible)
//! # }
//! ```
//!
//! Header cards can be written through the method
//! [`write_key`](struct.FitsFile.html#method.write_key). It takes a key name and value. See [the
//! `WritesKey`](trait.WritesKey.html) trait for supported data types.
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # {
//! # let fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! fptr.hdu(0).unwrap().write_key("foo", 1i64).unwrap();
//! assert_eq!(fptr.hdu(0).unwrap().read_key::<i64>("foo").unwrap(), 1i64);
//! # }
//! ```
//!
//! ## Reading file data
//!
//! ### Images
//!
//! Image data can be read through either
//! [`read_section`](struct.FitsHdu.html#method.read_section) which reads contiguous pixels
//! between a start index and end index, or
//! [`read_region`](struct.FitsHdu.html#method.read_region) which reads rectangular chunks from
//! the image.
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! // Read the first 100 pixels
//! let first_row: Vec<i32> = hdu.read_section(0, 100).unwrap();
//!
//! // Read a square section of the image
//! use fitsio::positional::Coordinate;
//!
//! let lower_left = Coordinate { x: 0, y: 0 };
//! let upper_right = Coordinate { x: 10, y: 10 };
//! let chunk: Vec<i32> = hdu.read_region(&lower_left, &upper_right).unwrap();
//! # }
//! ```
//!
//! ### Tables
//!
//! Columns can be read using the [`read_col`](struct.FitsFile.html#method.read_col) function,
//! which can convert data types on the fly. See the [`ReadsCol`](trait.ReadsCol.html) trait for
//! supported data types.
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! # fptr.change_hdu(1).unwrap();
//! let integer_data: Vec<i32> = fptr.hdu(1).and_then(|hdu| hdu.read_col("intcol")).unwrap();
//! # }
//! ```
//!
//! The [`columns`](struct.FitsFile.html#method.columns) method returns an iterator over all of the
//! columns in a table.
//!
//! [1]: http://heasarc.gsfc.nasa.gov/fitsio/fitsio.html
//! [2]: https://crates.io/crates/fitsio-sys

extern crate fitsio_sys as sys;
extern crate libc;

#[macro_use]
pub mod fitserror;
mod stringutils;
mod columndescription;
pub mod positional;
mod fitsfile;
mod fitshdu;
mod conversions;

pub use self::fitsfile::{FitsFile, HduInfo};
pub use self::fitshdu::FitsHdu;
