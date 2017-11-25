//! `fitsio` - a thin wrapper around the [`cfitsio`][1] C library.
//!
//! * [HDU access](#hdu-access)
//! * [Creating new HDUs](#creating-new-hdus)
//!     * [Creating a new image](#creating-a-new-image)
//!     * [Creating a new table](#creating-a-new-table)
//!         * [Column descriptions](#column-descriptions)
//!     * [Copying HDUs to another file](#copying-hdus-to-another-file)
//!     * [Deleting a HDU](#deleting-a-hdu)
//!     * [Iterating over the HDUs in a file](#iterating-over-the-hdus-in-a-file)
//!     * [General calling behaviour](#general-calling-behaviour)
//! * [Header keys](#header-keys)
//! * [Reading file data](#reading-file-data)
//!     * [Reading images](#reading-images)
//!     * [Reading tables](#reading-tables)
//!     * [Iterating over columns](#iterating-over-columns)
//! * [Writing file data](#writing-file-data)
//!     * [Writing images](#writing-images)
//!         * [Resizing an image](#resizing-an-image)
//!     * [Writing tables](#writing-tables)
//!         * [Writing table data](#writing-table-data)
//!         * [Inserting columns](#inserting-columns)
//!         * [Deleting columns](#deleting-columns)
//! * [Raw fits file access](#raw-fits-file-access)
//!
//! This library wraps the low level `cfitsio` bindings: [`fitsio-sys`][2] and provides a more
//! native experience for rust users.
//!
//! The main interface to a fits file is [`FitsFile`][fits-file]. All file manipulation
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
//! [`create`][fits-file-create]:
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
//! [`edit`][fits-file-edit] must be used. This opens a file which already exists
//! on disk for editing.
//!
//! ```rust
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! use fitsio::FitsFile;
//!
//! // let filename = ...;
//! let fptr = FitsFile::edit(filename).unwrap();
//! # }
//! ```
//!
//! # HDU access
//!
//! HDU information belongs to the [`FitsHdu`][fits-hdu] object. HDUs can be fetched by
//! `String`/`str` or integer (0-indexed).
//! The `HduInfo` object contains information about the current HDU:
//!
//! ```rust
//! # extern crate fitsio;
//! # #[cfg(feature = "default")]
//! # extern crate fitsio_sys as sys;
//! # #[cfg(feature = "bindgen")]
//! # extern crate fitsio_sys_bindgen as sys;
//! # use fitsio::{FitsFile, HduInfo};
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let mut fptr = FitsFile::open(filename).unwrap();
//! let hdu = fptr.hdu(0).unwrap();
//! // image HDU
//! if let HduInfo::ImageInfo { shape, .. } = hdu.info {
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
//! # Creating new HDUs
//!
//! ## Creating a new image
//!
//! New fits images are created with the [`create_image`][fits-file-create-image]
//! method. This method requires the extension name, and an
//! [`ImageDescription`][image-description] object, which defines the shape and type of
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
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! let image_description = ImageDescription {
//!     data_type: ImageType::FLOAT_IMG,
//!     dimensions: &[100, 100],
//! };
//! let hdu = fptr.create_image("EXTNAME".to_string(), &image_description).unwrap();
//! # }
//! ```
//!
//! ## Creating a new table
//!
//! Similar to creating new images, new tables are created with the
//! [`create_table`][fits-file-create-table] method. This requires an extension
//! name, and a slice of [`ColumnDescription`][column-description]s:
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use fitsio::columndescription::*;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! let first_description = ColumnDescription::new("A")
//!     .with_type(ColumnDataType::Int)
//!     .create().unwrap();
//! let second_description = ColumnDescription::new("B")
//!     .with_type(ColumnDataType::Long)
//!     .create().unwrap();
//! let descriptions = [first_description, second_description];
//! let hdu = fptr.create_table("EXTNAME".to_string(), &descriptions).unwrap();
//! # }
//! ```
//!
//! ### Column descriptions
//!
//! Columns are described with the
//! [`ColumnDescription`][column-description] struct. This
//! encapsulates: the name of the column, and the data format.
//!
//! The fits specification allows scalar or vector columns, and the data format is described the
//! [`ColumnDataDescription`][column-data-description] struct, which in
//! turn encapsulates the number of elements per row element (typically 1), the width of the
//! column (for strings), and the data type, which is one of the
//! [`ColumnDataType`][column-data-type] members
//!
//! For the common case of a scalar column, a `ColumnDataDescription` object can be constructed
//! with the `scalar` method:
//!
//! ```rust
//! # extern crate fitsio;
//! # use fitsio::columndescription::*;
//! # fn main() {
//! let desc = ColumnDataDescription::scalar(ColumnDataType::Int);
//! assert_eq!(desc.repeat, 1);
//! assert_eq!(desc.width, 1);
//! # }
//! ```
//!
//! Vector columns can be constructed with the `vector` method:
//!
//! ```rust
//! # extern crate fitsio;
//! # use fitsio::columndescription::*;
//! # fn main() {
//! let desc = ColumnDataDescription::vector(ColumnDataType::Int, 100);
//! assert_eq!(desc.repeat, 100);
//! assert_eq!(desc.width, 1);
//! # }
//! ```
//!
//! These impl `From<...> for String` such that the traditional fits column description string can
//! be obtained:
//!
//! ```rust
//! # extern crate fitsio;
//! # use fitsio::columndescription::*;
//! # fn main() {
//! let desc = ColumnDataDescription::scalar(ColumnDataType::Int);
//! assert_eq!(String::from(desc), "1J".to_string());
//! # }
//! ```
//!
//! ## Copying HDUs to another file
//!
//! A HDU can be copied to another open file with the [`copy_to`][fits-hdu-copy-to] method. This
//! requires another open [`FitsFile`][fits-file] object to copy to:
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let mut src_fptr = fitsio::FitsFile::open(filename).unwrap();
//! #
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut dest_fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! #
//! # let hdu = src_fptr.hdu(1).unwrap();
//! hdu.copy_to(&mut src_fptr, &mut dest_fptr).unwrap();
//! # }
//! ```
//!
//! ## Deleting a HDU
//!
//! The current HDU can be deleted using the [`delete`][fits-hdu-delete] method. Note: this method
//! takes ownership of `self`, and as such the [`FitsHdu`][fits-hdu] object cannot be used after
//! this is called.
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
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let image_description = ImageDescription {
//! #     data_type: ImageType::FLOAT_IMG,
//! #     dimensions: &[100, 100],
//! # };
//! # let hdu = fptr.create_image("EXTNAME".to_string(), &image_description).unwrap();
//! // let fptr = FitsFile::open(...).unwrap();
//! // let hdu = fptr.hdu(0).unwrap();
//! fptr.delete(hdu).unwrap();
//! // Cannot use hdu after this
//! # }
//! ```
//!
//! ## Iterating over the HDUs in a file
//!
//! The [`iter`][fits-hdu-iter] method allows for iteration over the HDUs of a fits file.
//!
//! ```rust
//! # extern crate fitsio;
//! # fn main() {
//! #     let mut fptr = fitsio::FitsFile::open("../testdata/full_example.fits").unwrap();
//! for hdu in fptr.iter() {
//!     // Do something with hdu
//! }
//! # }
//! ```
//!
//! ## General calling behaviour
//!
//! All subsequent data acess is performed through the [`FitsFile`][fits-file] object. Most methods
//! take an [`FitsHdu`][fits-hdu] object as the first parameter.
//!
//! # Header keys
//!
//! Header keys are read through the [`read_key`][fits-hdu-read-key] function,
//! and is generic over types that implement the [`ReadsKey`][reads-key] trait:
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let mut fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! # {
//! let int_value: i64 = fptr.read_key(&hdu, "INTTEST").unwrap();
//! # }
//!
//! // Alternatively
//! # {
//! let int_value = fptr.read_key::<i64>(&hdu, "INTTEST").unwrap();
//! # }
//!
//! // Or let the compiler infer the types (if possible)
//! # }
//! ```
//!
//! Header cards can be written through the method
//! [`write_key`][fits-hdu-write-key]. It takes a key name and value. See [the
//! `WritesKey`][writes-key] trait for supported data types.
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # {
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! fptr.write_key(&hdu, "foo", 1i64).unwrap();
//! assert_eq!(fptr.read_key::<i64>(&hdu, "foo").unwrap(), 1i64);
//! # }
//! ```
//!
//! # Reading file data
//!
//! ## Reading images
//!
//! Image data can be read through either
//! [`read_section`][fits-hdu-read-section] which reads contiguous pixels
//! between a start index and end index, or
//! [`read_region`][fits-hdu-read-region] which reads rectangular chunks from
//! the image.
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let mut fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! // Read the first 100 pixels
//! let first_row: Vec<i32> = fptr.read_section(&hdu, 0, 100).unwrap();
//!
//! // Read a square section of the image
//!
//! let xcoord = 0..10;
//! let ycoord = 0..10;
//! let chunk: Vec<i32> = fptr.read_region(&hdu, &[&ycoord, &xcoord]).unwrap();
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
//! # let mut fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! let start_row = 0;
//! let num_rows = 10;
//! let first_few_rows: Vec<f32> = fptr.read_rows(&hdu, start_row, num_rows).unwrap();
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
//! # let mut fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! let image_data: Vec<f32> = fptr.read_image(&hdu).unwrap();
//!
//! // 100 rows of 100 columns
//! assert_eq!(image_data.len(), 10_000);
//! # }
//! ```
//!
//! ## Reading tables
//!
//! Columns can be read using the [`read_col`][fits-hdu-read-col] function,
//! which can convert data types on the fly. See the [`ReadsCol`][reads-col] trait for
//! supported data types.
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let mut fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu(1).unwrap();
//! let integer_data: Vec<i32> = fptr.read_col(&hdu, "intcol").unwrap();
//! # }
//! ```
//!
//! ## Iterating over columns
//!
//! Iterate over the columns with [`columns`][fits-hdu-columns].
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let mut fptr = fitsio::FitsFile::open(filename).unwrap();
//! # let hdu = fptr.hdu("TESTEXT").unwrap();
//! for column in fptr.columns(&hdu) {
//!     // Do something with column
//! }
//! # }
//! ```
//!
//! # Writing file data
//!
//! ## Writing images
//!
//! Image data is written through two methods on the HDU object:
//! [`write_section`][fits-hdu-write-section] and
//! [`write_region`][fits-hdu-write-region].
//!
//! [`write_section`][fits-hdu-write-section] requires a start index and
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
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let desc = ImageDescription {
//! #    data_type: ImageType::FLOAT_IMG,
//! #    dimensions: &[100, 100],
//! # };
//! # let hdu = fptr.create_image("".to_string(), &desc).unwrap();
//! let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0];
//! fptr.write_section(&hdu, 0, data_to_write.len(), &data_to_write).unwrap();
//! # }
//! ```
//!
//! [`write_region`][fits-hdu-write-region] takes a slice of ranges with which
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
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let desc = ImageDescription {
//! #    data_type: ImageType::FLOAT_IMG,
//! #    dimensions: &[100, 100],
//! # };
//! # let hdu = fptr.create_image("".to_string(), &desc).unwrap();
//! let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
//! let ranges = [&(0..1), &(0..1)];
//! fptr.write_region(&hdu, &ranges, &data_to_write).unwrap();
//! # }
//! ```
//!
//! ### Resizing an image
//!
//! Images can be resized to a new shape using the [`resize`][fits-hdu-resize] method.
//!
//! The method takes the open [`FitsFile`][fits-file], and an slice of `usize` values. Note:
//! currently `fitsio` only supports slices with length 2, i.e. a 2D image.
//! [`resize`][fits-hdu-resize] takes ownership `self` to force the user to fetch the HDU object
//! again. This ensures the image changes are reflected in the hew HDU object.
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use std::fs::copy;
//! # use fitsio::HduInfo;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # copy("../testdata/full_example.fits", &filename).unwrap();
//! # let filename = filename.to_str().unwrap();
//! # let mut fptr = fitsio::FitsFile::edit(filename).unwrap();
//! # let hdu = fptr.hdu(0).unwrap();
//! fptr.resize(hdu, &[1024, 1024]).unwrap();
//! #
//! // Have to get the HDU again, to reflect the latest changes
//! let hdu = fptr.hdu(0).unwrap();
//! match hdu.info {
//!     HduInfo::ImageInfo { shape, .. } => {
//!         assert_eq!(shape, [1024, 1024]);
//!     }
//!     _ => panic!("Unexpected hdu type"),
//! }
//! # }
//! ```
//!
//! ## Writing tables
//!
//! ### Writing table data
//!
//! Tablular data can either be written with [`write_col`][fits-hdu-write-col] or
//! [`write_col_range`][fits-hdu-write-col-range].
//!
//! [`write_col`][fits-hdu-write-col] writes an entire column's worth of data to the file. It does
//! not check how many rows are in the file, but extends the table if the length of data is longer
//! than the table length.
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use std::fs::copy;
//! # use fitsio::HduInfo;
//! # use fitsio::columndescription::*;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let table_description = vec![
//! #     ColumnDescription::new("bar")
//! #         .with_type(ColumnDataType::Int)
//! #         .create()
//! #         .unwrap(),
//! # ];
//! # let hdu = fptr.create_table("foo".to_string(), &table_description)
//! #     .unwrap();
//! let data_to_write: Vec<i32> = vec![10101; 5];
//! fptr.write_col(&hdu, "bar", &data_to_write).unwrap();
//! let data: Vec<i32> = fptr.read_col(&hdu, "bar").unwrap();
//! assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
//! # }
//! ```
//!
//! [`write_col_range`][fits-hdu-write-col-range] writes data to a range of rows in a table. The
//! range is inclusive of both the upper and lower bounds, so `0..4` writes 5 elements.
//!
//! ```rust
//! # extern crate tempdir;
//! # extern crate fitsio;
//! # use std::fs::copy;
//! # use fitsio::HduInfo;
//! # use fitsio::columndescription::*;
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let table_description = vec![
//! #     ColumnDescription::new("bar")
//! #         .with_type(ColumnDataType::Int)
//! #         .create()
//! #         .unwrap(),
//! # ];
//! # let hdu = fptr.create_table("foo".to_string(), &table_description)
//! #     .unwrap();
//! let data_to_write: Vec<i32> = vec![10101; 10];
//! fptr.write_col_range(&hdu, "bar", &data_to_write, &(0..4)).unwrap();
//! let data: Vec<i32> = fptr.read_col(&hdu, "bar").unwrap();
//! assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
//! # }
//! ```
//!
//! ### Inserting columns
//!
//! Two methods on the HDU object allow for adding new columns:
//! [`append_column`][fits-hdu-append-column]
//! and [`insert_column`][fits-hdu-insert-column].
//! [`append_column`][fits-hdu-append-column] adds a new column as the last column member, and is
//! generally
//! preferred as it does not require shifting of data within the file.
//!
//! ```rust
//! # extern crate fitsio;
//! # extern crate tempdir;
//! # use fitsio::fitsfile::ImageDescription;
//! # use fitsio::types::ImageType;
//! # use fitsio::columndescription::{ColumnDescription, ColumnDataType};
//! #
//! # fn main() {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let table_description = &[
//! #     ColumnDescription::new("bar")
//! #         .with_type(ColumnDataType::Int)
//! #         .create()
//! #         .unwrap(),
//! # ];
//! # let hdu = fptr.create_table("foo".to_string(), table_description)
//! #     .unwrap();
//! let column_description = ColumnDescription::new("abcdefg")
//! .with_type(ColumnDataType::Int)
//! .create().unwrap();
//! fptr.append_column(&hdu, &column_description).unwrap();
//! # }
//!
//! ```
//!
//! ### Deleting columns
//!
//! The HDU object has the method [`delete_column`][fits-hdu-delete-column] which removes a column.
//! The column can either be accessed by integer or name
//!
//! ```rust
//! # extern crate fitsio;
//! # extern crate tempdir;
//! # use fitsio::fitsfile::ImageDescription;
//! # use fitsio::types::ImageType;
//! # use fitsio::columndescription::{ColumnDescription, ColumnDataType};
//! #
//! # fn main() {
//! # {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let table_description = &[
//! #     ColumnDescription::new("bar")
//! #         .with_type(ColumnDataType::Int)
//! #         .create()
//! #         .unwrap(),
//! # ];
//! # let hdu = fptr.create_table("foo".to_string(), table_description)
//! #     .unwrap();
//! let newhdu = fptr.delete_column(&hdu, "bar").unwrap();
//! # }
//! # {
//! # let tdir = tempdir::TempDir::new("fitsio-").unwrap();
//! # let tdir_path = tdir.path();
//! # let filename = tdir_path.join("test.fits");
//! # let mut fptr = fitsio::FitsFile::create(filename.to_str().unwrap()).unwrap();
//! # let table_description = &[
//! #     ColumnDescription::new("bar")
//! #         .with_type(ColumnDataType::Int)
//! #         .create()
//! #         .unwrap(),
//! # ];
//! # let hdu = fptr.create_table("foo".to_string(), table_description)
//! #     .unwrap();
//! // or
//! let newhdu = fptr.delete_column(&hdu, 0).unwrap();
//! # }
//! # }
//! ```
//!
//! # Raw fits file access
//!
//! If this library does not support the particular use case that is needed, the raw `fitsfile`
//! pointer can be accessed:
//!
//! ```rust
//! # extern crate fitsio;
//! # #[cfg(not(feature="bindgen"))]
//! extern crate fitsio_sys;
//! # #[cfg(feature="bindgen")]
//! # extern crate fitsio_sys_bindgen as fitsio_sys;
//!
//! # use fitsio::FitsFile;
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! let fptr = FitsFile::open(filename).unwrap();
//!
//! /* Find out the number of HDUs in the file */
//! let mut num_hdus = 0;
//! let mut status = 0;
//!
//! unsafe {
//!     let fitsfile = fptr.as_raw();
//!
//!     /* Use the unsafe fitsio-sys low level library to call a function that is possibly not
//!     implemented in this crate */
//!     fitsio_sys::ffthdu(fitsfile, &mut num_hdus, &mut status);
//! }
//! assert_eq!(num_hdus, 2);
//! # }
//! ```
//!
//! This (unsafe) pointer can then be used with the underlying [`fitsio-sys`][2] library directly.
//!
//! [1]: http://heasarc.gsfc.nasa.gov/fitsio/fitsio.html
//! [2]: https://crates.io/crates/fitsio-sys
//! [column-data-description]: columndescription/struct.ColumnDataDescription.html
//! [column-data-type]: columndescription/struct.ColumnDataType.html
//! [column-description]: columndescription/struct.ColumnDescription.html
//! [fits-file-create-image]: fitsfile/struct.FitsFile.html#method.create_image
//! [fits-file-create-table]: fitsfile/struct.FitsFile.html#method.create_table
//! [fits-file-create]: fitsfile/struct.FitsFile.html#method.create
//! [fits-file-edit]: fitsfile/struct.FitsFile.html#method.edit
//! [fits-file]: fitsfile/struct.FitsFile.html
//! [fits-hdu-append-column]: fitsfile/struct.FitsHdu.html#method.append_column
//! [fits-hdu-columns]: fitsfile/struct.FitsHdu.html#method.columns
//! [fits-hdu-delete-column]: fitsfile/struct.FitsHdu.html#method.delete_column
//! [fits-hdu-insert-column]: fitsfile/struct.FitsHdu.html#method.insert_column
//! [fits-hdu-read-col]: fitsfile/struct.FitsHdu.html#method.read_col
//! [fits-hdu-read-key]: fitsfile/struct.FitsHdu.html#method.read_key
//! [fits-hdu-read-region]: fitsfile/struct.FitsHdu.html#method.read_region
//! [fits-hdu-read-section]: fitsfile/struct.FitsHdu.html#method.read_section
//! [fits-hdu-write-key]: fitsfile/struct.FitsHdu.html#method.write_key
//! [fits-hdu-write-col]: fitsfile/struct.FitsHdu.html#method.write_col
//! [fits-hdu-write-col-range]: fitsfile/struct.FitsHdu.html#method.write_col_range
//! [fits-hdu-write-region]: fitsfile/struct.FitsHdu.html#method.write_region
//! [fits-hdu-write-section]: fitsfile/struct.FitsHdu.html#method.write_section
//! [fits-hdu-iter]: fitsfile/struct.FitsHdu.html#method.iter
//! [fits-hdu-copy-to]: fitsfile/struct.FitsHdu.html#method.copy_to
//! [fits-hdu-delete]: fitsfile/struct.FitsHdu.html#method.copy_to
//! [fits-hdu-resize]: fitsfile/struct.FitsHdu.html#method.copy_to
//! [fits-hdu]: fitsfile/struct.FitsHdu.html
//! [image-description]: fitsfile/struct.ImageDescription.html
//! [reads-col]: fitsfile/trait.ReadsCol.html
//! [reads-key]: fitsfile/trait.ReadsKey.html
//! [writes-key]: fitsfile/trait.ReadsKey.html

#![warn(missing_docs)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[cfg(feature = "default")]
extern crate fitsio_sys as sys;
#[cfg(feature = "bindgen")]
extern crate fitsio_sys_bindgen as sys;

extern crate libc;

#[macro_use]
mod fitserror;
pub mod errors;
mod stringutils;
pub mod types;
pub mod columndescription;
pub mod fitsfile;

pub use self::fitsfile::{FitsFile, FitsHdu};
pub use self::types::HduInfo;
pub use self::errors::{Error, Result};
