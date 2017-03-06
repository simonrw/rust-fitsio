//! `fitsio` - a thin wrapper around the [`cfitsio`][1] C library.
//!
//! * [HDU access](#hdu-access)
//! * [Creating new HDUs](#creating-new-hdus)
//! * [Header keys](#header-keys)
//! * [Reading file data](#reading-file-data)
//!     * [Images](#images)
//!     * [Tables](#tables)
//! * [Writing file data](#writing-file-data)
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
//! To open a fits file in read/write mode (to allow changes to the file), the
//! [`edit`](struct.FitsFile.html#method.edit) must be used. This opens a file which already exists
//! on disk for editing.
//!
//! ## HDU access
//!
//! HDU information belongs to the [`FitsHdu`](struct.FitsHdu.html) object. HDUs can be fetched by
//! `String`/`str` or integer (0-indexed).
//! The `HduInfo` object contains information about the current HDU:
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
//! if let HduInfo::ImageInfo { shape } = hdu.info {
//!    println!("Image is {}-dimensional", shape.len());
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
//! ## Creating new HDUs
//!
//! ### Images
//!
//! New fits images are created with the [`create_image`](struct.FitsFile.html#method.create_image)
//! method. This method requires the extension name, and an
//! [`ImageDescription`](struct.ImageDescription.html) object, which defines the shape and type of
//! the desired image:
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use fitsio::fitsfile::ImageDescription;
//! # use fitsio::types::ImageType;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! let image_description = ImageDescription {
//!     data_type: ImageType::FLOAT_IMG,
//!     dimensions: vec![100, 100],
//! };
//! let mut hdu = fptr.create_image("EXTNAME".to_string(), &image_description).unwrap();
//! # }
//! ```
//!
//! ### Tables
//!
//! Similar to creating new images, new tables are created with the
//! [`create_table`](struct.FitsFile.html#method.create_table) method. This requires an extension
//! name, and a slice of [`ColumnDescription`](columndescription/struct.ColumnDescription.html)s:
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use fitsio::columndescription::ColumnDescription;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! let first_description = ColumnDescription {
//!     name: "A".to_string(),
//!     data_type: "1J".to_string(),
//! };
//! let second_description = ColumnDescription {
//!     name: "B".to_string(),
//!     data_type: "1K".to_string(),
//! };
//! let descriptions = [first_description, second_description];
//! let mut hdu = fptr.create_table("EXTNAME".to_string(), &descriptions).unwrap();
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
//!
//! let xcoord = 0..10;
//! let ycoord = 0..10;
//! let chunk: Vec<i32> = hdu.read_region(&vec![&ycoord, &xcoord]).unwrap();
//! # }
//! ```
//!
//! Some convenience methods are available for reading rows of the image. This is
//! typically useful as it's an efficient access method:
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! let start_row = 0;
//! let num_rows = 10;
//! let first_few_rows: Vec<f32> = hdu.read_rows(start_row, num_rows).unwrap();
//!
//! // 10 rows of 100 columns
//! assert_eq!(first_few_rows.len(), 1000);
//! # }
//! ```
//!
//! The whole image can also be read into memory:
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! let image_data: Vec<f32> = hdu.read_image().unwrap();
//!
//! // 100 rows of 100 columns
//! assert_eq!(image_data.len(), 10_000);
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
//! # let hdu = fptr.hdu(1);
//! let integer_data: Vec<i32> = hdu.and_then(|hdu| hdu.read_col("intcol")).unwrap();
//! # }
//! ```
//!
//! The [`columns`](struct.FitsFile.html#method.columns) method returns an iterator over all of the
//! columns in a table.
//!
//! ## Writing file data
//!
//! When writing to the file, all methods are attached to the `FitsHdu` object to which data is to
//! be written. As these methods manipulate the underlying file information, the `FitsHdu` object
//! must be `mut`.
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! let mut hdu = fptr.hdu(1);
//! # }
//! ```
//!
//! ### Images
//!
//! Image data is written through two methods on the HDU object:
//! [`write_section`](struct.FitsHdu.html#method.write_section) and
//! [`write_region`](struct.FitsHdu.html#method.write_region):o
//!
//! [`write_section`](struct.FitsHdu.html#method.write_section) requires a start index and
//! end index and data to write. The data parameter needs to be a slice, meaning any contiguous
//! memory storage method (e.g. `Vec`) can be passed.
//!
//! ```rust
//! # extern crate fitsio;
//! # extern crate tempdir;
//! # use fitsio::fitsfile::ImageDescription;
//! # use fitsio::types::ImageType;
//! #
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let desc = ImageDescription {
//! #    data_type: ImageType::FLOAT_IMG,
//! #    dimensions: vec![100, 100],
//! # };
//! # let mut hdu = fptr.create_image("".to_string(), &desc).unwrap();
//! let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0];
//! hdu.write_section(0, data_to_write.len(), &data_to_write).unwrap();
//! # }
//! ```
//!
//! [`write_region`](struct.FitsHdu.html#method.write_region) takes a slice of ranges with which
//! the data is to be written, and the data to write.
//!
//! ```rust
//! # extern crate fitsio;
//! # extern crate tempdir;
//! # use fitsio::fitsfile::ImageDescription;
//! # use fitsio::types::ImageType;
//! #
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let desc = ImageDescription {
//! #    data_type: ImageType::FLOAT_IMG,
//! #    dimensions: vec![100, 100],
//! # };
//! # let mut hdu = fptr.create_image("".to_string(), &desc).unwrap();
//! let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
//! let ranges = [&(0..1), &(0..1)];
//! hdu.write_region(&ranges, &data_to_write).unwrap();
//! # }
//! ```
//!
//! ### Tables
//!
//! [1]: http://heasarc.gsfc.nasa.gov/fitsio/fitsio.html
//! [2]: https://crates.io/crates/fitsio-sys

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate fitsio_sys as sys;
extern crate libc;

#[macro_use]
pub mod fitserror;
mod stringutils;
pub mod types;
pub mod columndescription;
pub mod fitsfile;
mod conversions;

pub use self::fitsfile::{FitsFile, FitsHdu};
pub use self::types::HduInfo;
