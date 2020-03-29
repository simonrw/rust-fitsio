/*!
`fitsio` - a thin wrapper around the [`cfitsio`][cfitsio] C library.

* [File access](#file-access)
    * [Pretty printing](#pretty-printing)
* [HDU access](#hdu-access)
* [Creating new HDUs](#creating-new-hdus)
    * [Creating a new image](#creating-a-new-image)
    * [Creating a new table](#creating-a-new-table)
        * [Column descriptions](#column-descriptions)
    * [Copying HDUs to another file](#copying-hdus-to-another-file)
    * [Deleting a HDU](#deleting-a-hdu)
    * [Iterating over the HDUs in a file](#iterating-over-the-hdus-in-a-file)
    * [General calling behaviour](#general-calling-behaviour)
* [Header keys](#header-keys)
* [Reading file data](#reading-file-data)
    * [Reading images](#reading-images)
        * [`ndarray` support](#ndarray-support)
    * [Reading tables](#reading-tables)
        * [Reading cell values](#reading-cell-values)
        * [Reading rows](#reading-rows)
    * [Iterating over columns](#iterating-over-columns)
* [Writing file data](#writing-file-data)
    * [Writing images](#writing-images)
        * [Resizing an image](#resizing-an-image)
    * [Writing tables](#writing-tables)
        * [Writing table data](#writing-table-data)
        * [Inserting columns](#inserting-columns)
        * [Deleting columns](#deleting-columns)
* [Raw fits file access](#raw-fits-file-access)
* [Threadsafe access](#threadsafe-access)

This library wraps the low level `cfitsio` bindings: [`fitsio-sys`][fitsio-sys] and provides a more
native experience for rust users.

The main interface to a fits file is [`FitsFile`][fits-file]. All file manipulation
and reading starts with this class.

# File access

To open an existing file, use the [open][fitsfile-open] method.

```rust
use fitsio::FitsFile;
# use std::error::Error;

# fn try_main() -> Result<(), Box<dyn Error>> {
# let filename = "../testdata/full_example.fits";
// let filename = ...;
let fptr = FitsFile::open(filename)?;
# Ok(())
# }
# fn main() {
# try_main().unwrap();
# }
```

Alternatively a new file can be created on disk with the companion method
[`create`][fits-file-create]:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
use fitsio::FitsFile;

// let filename = ...;
let fptr = FitsFile::create(filename).open()?;
# Ok(())
# }
# fn main() {
# try_main().unwrap();
# }
```

The [`create`][fits-file-create] method returns a [`NewFitsFile`][new-fits-file], which is an
internal representation of a temporary fits file on disk, before the file is fully created.

This representation has two methods: [`open`][new-fits-file-open] and
[`with_custom_primary`][new-fits-file-with-custom-primary]. The [`open`][new-fits-file-open]
method actually creates the file on disk, but before calling this method, the
[`with_custom_primary`][new-fits-file-with-custom-primary] method can be used to add a custom
primary HDU. This is mostly useful for images. Otherwise, a default primary HDU is created.  An
example of not adding a custom primary HDU is shown above. Below we see an example of
[`with_custom_primary`][new-fits-file-with-custom-primary]:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
use fitsio::FitsFile;
use fitsio::images::{ImageType, ImageDescription};

// let filename = ...;
let description = ImageDescription {
    data_type: ImageType::Double,
    dimensions: &[52, 103],
};

let fptr = FitsFile::create(filename)
    .with_custom_primary(&description)
    .open()?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

From this point, the current HDU can be queried and changed, or fits header cards can be read
or file contents can be read.

To open a fits file in read/write mode (to allow changes to the file), the
[`edit`][fits-file-edit] must be used. This opens a file which already exists
on disk for editing.

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
use fitsio::FitsFile;

// let filename = ...;
let fptr = FitsFile::edit(filename)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

## Pretty printing

Fits files can be pretty-printed with [`pretty_print`][pretty-print], or its more powerful
cousin [`pretty_write`][pretty-write].

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# use std::io;
use fitsio::FitsFile;

let mut fptr = FitsFile::open(filename)?;
fptr.pretty_print()?;
// or
fptr.pretty_write(&mut io::stdout())?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

In the continuing tradition of releasing fits summary programs with each fits library, this
create contains a binary program [`fitssummary`] which can be installed with `cargo install`. This
takes fits files on the command line and prints their summaries to stdout.

```sh
$ fitssummary ../testdata/full_example.fits

  file: ../testdata/full_example.fits
  mode: READONLY
  extnum hdutype      hduname    details
  0      IMAGE_HDU               dimensions: [100, 100], type: Long
  1      BINARY_TBL   TESTEXT    num_cols: 4, num_rows: 50
```

# HDU access

HDU information belongs to the [`FitsHdu`][fits-hdu] object. HDUs can be fetched by
`String`/`str` or integer (0-indexed), with the [`hdu`][fitsfile-hdu] method.  The `HduInfo`
object contains information about the current HDU:

```rust
# #[cfg(feature = "default")]
# use fitsio_sys as sys;
# #[cfg(feature = "bindgen")]
# use fitsio_sys_bindgen as sys;
# use fitsio::FitsFile;
#
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = FitsFile::open(filename)?;
use fitsio::hdu::HduInfo;

let hdu = fptr.hdu(0)?;
// image HDU
if let HduInfo::ImageInfo { shape, .. } = hdu.info {
   println!("Image is {}-dimensional", shape.len());
   println!("Found image with shape {:?}", shape);
}
# let hdu = fptr.hdu("TESTEXT")?;

// tables
if let HduInfo::TableInfo { column_descriptions, num_rows, .. } = hdu.info {
    println!("Table contains {} rows", num_rows);
    println!("Table has {} columns", column_descriptions.len());
}
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

The primary HDU can always be accessed with the `FitsFile::primary_hdu` method.

# Creating new HDUs

## Creating a new image

New fits images are created with the [`create_image`][fits-file-create-image]
method. This method requires the extension name, and an
[`ImageDescription`][image-description] object, which defines the shape and type of
the desired image:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
use fitsio::images::{ImageDescription, ImageType};

let image_description = ImageDescription {
    data_type: ImageType::Float,
    dimensions: &[100, 100],
};
let hdu = fptr.create_image("EXTNAME".to_string(), &image_description)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

_Unlike cfitsio, the order of the dimensions of `new_size` follows the C convention, i.e.
[row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order)._

## Creating a new table

Similar to creating new images, new tables are created with the
[`create_table`][fits-file-create-table] method. This requires an extension
name, and a slice of [`ColumnDescription`][column-description]s:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
use fitsio::tables::{ColumnDescription, ColumnDataType};

let first_description = ColumnDescription::new("A")
    .with_type(ColumnDataType::Int)
    .create()?;
let second_description = ColumnDescription::new("B")
    .with_type(ColumnDataType::Long)
    .create()?;
let descriptions = [first_description, second_description];
let hdu = fptr.create_table("EXTNAME".to_string(), &descriptions)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### Column descriptions

Columns are described with the
[`ColumnDescription`][column-description] struct. This
encapsulates: the name of the column, and the data format.

The fits specification allows scalar or vector columns, and the data format is described the
[`ColumnDataDescription`][column-data-description] struct, which in
turn encapsulates the number of elements per row element (typically 1), the width of the
column (for strings), and the data type, which is one of the
[`ColumnDataType`][column-data-type] members

For the common case of a scalar column, a `ColumnDataDescription` object can be constructed
with the `scalar` method:

```rust
# fn main() {
use fitsio::tables::{ColumnDescription, ColumnDataDescription, ColumnDataType};

let desc = ColumnDataDescription::scalar(ColumnDataType::Int);
assert_eq!(desc.repeat, 1);
assert_eq!(desc.width, 1);
# }
```

Vector columns can be constructed with the `vector` method:

```rust
# fn main() {
use fitsio::tables::{ColumnDataDescription, ColumnDescription, ColumnDataType};

let desc = ColumnDataDescription::vector(ColumnDataType::Int, 100);
assert_eq!(desc.repeat, 100);
assert_eq!(desc.width, 1);
# }
```

These impl `From<...> for String` such that the traditional fits column description string can
be obtained:

```rust
# fn main() {
use fitsio::tables::{ColumnDataDescription, ColumnDescription, ColumnDataType};

let desc = ColumnDataDescription::scalar(ColumnDataType::Int);
assert_eq!(String::from(desc), "1J".to_string());
# }
```

## Copying HDUs to another file

A HDU can be copied to another open file with the [`copy_to`][fits-hdu-copy-to] method. This
requires another open [`FitsFile`][fits-file] object to copy to:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut src_fptr = fitsio::FitsFile::open(filename)?;
#
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut dest_fptr = fitsio::FitsFile::create(filename).open()?;
#
# let hdu = src_fptr.hdu(1)?;
hdu.copy_to(&mut src_fptr, &mut dest_fptr)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

## Deleting a HDU

The current HDU can be deleted using the [`delete`][fits-hdu-delete] method. Note: this method
takes ownership of `self`, and as such the [`FitsHdu`][fits-hdu] object cannot be used after
this is called.

```rust
# use fitsio::images::{ImageType, ImageDescription};
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let image_description = ImageDescription {
#     data_type: ImageType::Float,
#     dimensions: &[100, 100],
# };
# let hdu = fptr.create_image("EXTNAME".to_string(), &image_description)?;
// let fptr = FitsFile::open(...)?;
// let hdu = fptr.hdu(0)?;
hdu.delete(&mut fptr)?;
// Cannot use hdu after this
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

## Iterating over the HDUs in a file

The [`iter`][fits-hdu-iter] method allows for iteration over the HDUs of a fits file.

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
#     let mut fptr = fitsio::FitsFile::open("../testdata/full_example.fits")?;
for hdu in fptr.iter() {
    // Do something with hdu
}
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

## General calling behaviour

All subsequent data acess is performed through the [`FitsHdu`][fits-hdu] object. Most methods
take the currently open [`FitsFile`][fits-file] as the first parameter.

# Header keys

Header keys are read through the [`read_key`][fits-hdu-read-key] function,
and is generic over types that implement the [`ReadsKey`][reads-key] trait:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = fitsio::FitsFile::open(filename)?;
# {
let int_value: i64 = fptr.hdu(0)?.read_key(&mut fptr, "INTTEST")?;
# }

// Alternatively
# {
let int_value = fptr.hdu(0)?.read_key::<i64>(&mut fptr, "INTTEST")?;
# }

// Or let the compiler infer the types (if possible)
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

Header cards can be written through the method
[`write_key`][fits-hdu-write-key]. It takes a key name and value. See [the
`WritesKey`][writes-key] trait for supported data types.

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# {
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
fptr.hdu(0)?.write_key(&mut fptr, "foo", 1i64)?;
assert_eq!(fptr.hdu(0)?.read_key::<i64>(&mut fptr, "foo")?, 1i64);
# Ok(())
# }
# }
# fn main() { try_main().unwrap(); }
```

# Reading file data

Methods taking ranges are exclusive of the upper range value, reflecting the nature of Rust's
range type.

## Reading images

Image data can be read through either
[`read_section`][fits-hdu-read-section] which reads contiguous pixels
between a start index and end index, or
[`read_region`][fits-hdu-read-region] which reads rectangular chunks from
the image.

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = fitsio::FitsFile::open(filename)?;
# let hdu = fptr.hdu(0)?;
// Read the first 100 pixels
let first_row: Vec<i32> = hdu.read_section(&mut fptr, 0, 100)?;

// Read a square section of the image
let xcoord = 0..10;
let ycoord = 0..10;
let chunk: Vec<i32> = hdu.read_region(&mut fptr, &[&ycoord, &xcoord])?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

_Unlike cfitsio, the order of the the section ranges follows the C convention, i.e.
[row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order)._

Some convenience methods are available for reading rows of the image. This is
typically useful as it's an efficient access method:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = fitsio::FitsFile::open(filename)?;
# let hdu = fptr.hdu(0)?;
let start_row = 0;
let num_rows = 10;
let first_few_rows: Vec<f32> = hdu.read_rows(&mut fptr, start_row, num_rows)?;

// 10 rows of 100 columns
assert_eq!(first_few_rows.len(), 1000);
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

The whole image can also be read into memory:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = fitsio::FitsFile::open(filename)?;
# let hdu = fptr.hdu(0)?;
let image_data: Vec<f32> = hdu.read_image(&mut fptr, )?;

// 100 rows of 100 columns
assert_eq!(image_data.len(), 10_000);
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### [`ndarray`][ndarray] support

When `fitsio` is compiled with the `array` feature, images can be read into
the [`ndarray::ArrayD`][arrayd] type:

```rust
# #[cfg(feature = "array")]
# fn main() {
use fitsio::FitsFile;
# #[cfg(feature = "array")]
use ndarray::ArrayD;

let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
let hdu = f.primary_hdu().unwrap();

let data: ArrayD<u32> = hdu.read_image(&mut f).unwrap();
let dim = data.dim();
assert_eq!(dim[0], 100);
assert_eq!(dim[1], 100);
assert_eq!(data[[20, 5]], 152);
# }
#
# #[cfg(not(feature = "array"))]
# fn main() {}
```

For more details, see the [`ndarray_compat`](ndarray_compat/index.html) documentation (only
available if compiled with `array` feature).

## Reading tables

Columns can be read using the [`read_col`][fits-hdu-read-col] function,
which can convert data types on the fly. See the [`ReadsCol`][reads-col] trait for
supported data types.

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = fitsio::FitsFile::open(filename)?;
# let hdu = fptr.hdu(1);
let integer_data: Vec<i32> = hdu.and_then(|hdu| hdu.read_col(&mut fptr, "intcol"))?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### Reading cell values

Individual cell values can be read from FITS tables:

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits[TESTEXT]";
# let mut f = fitsio::FitsFile::open(filename)?;
# let tbl_hdu = f.hdu("TESTEXT")?;
let result: i64 = tbl_hdu.read_cell_value(&mut f, "intcol", 4)?;
assert_eq!(result, 16);

let result: String = tbl_hdu.read_cell_value(&mut f, "strcol", 4)?;
assert_eq!(result, "value4".to_string());
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### Reading rows

Single rows can be read from a fits table with the [`row`][fits-hdu-row] method. This requires
use of the [`fitsio-derive`][fitsio-derive] crate.

```rust
use fitsio::tables::FitsRow;
use fitsio_derive::FitsRow;

#[derive(Default, FitsRow)]
struct Row {
    #[fitsio(colname = "intcol")]
    intfoo: i32,
    #[fitsio(colname = "strcol")]
    foobar: String,
}
#
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits[TESTEXT]";
# let mut f = fitsio::FitsFile::open(filename)?;
# let hdu = f.hdu("TESTEXT")?;

// Pick the 4th row
let row: Row = hdu.row(&mut f, 4)?;
assert_eq!(row.intfoo, 16);
assert_eq!(row.foobar, "value4");
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

## Iterating over columns

Iterate over the columns with [`columns`][fits-hdu-columns].

```rust
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let filename = "../testdata/full_example.fits";
# let mut fptr = fitsio::FitsFile::open(filename)?;
# let hdu = fptr.hdu("TESTEXT")?;
for column in hdu.columns(&mut fptr) {
    // Do something with column
}
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

# Writing file data

Methods taking ranges are exclusive of the upper range value, reflecting the nature of Rust's
range type.

## Writing images

Image data is written through three methods on the HDU object:
[`write_section`][fits-hdu-write-section], [`write_region`][fits-hdu-write-region], and
[`write_image`][fits-hdu-write-image].

[`write_section`][fits-hdu-write-section] requires a start index and
end index and data to write. The data parameter needs to be a slice, meaning any contiguous
memory storage method (e.g. `Vec`) can be passed.

```rust
# use fitsio::images::{ImageType, ImageDescription};
#
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let desc = ImageDescription {
#    data_type: ImageType::Float,
#    dimensions: &[100, 100],
# };
# let hdu = fptr.create_image("".to_string(), &desc)?;
let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0];
hdu.write_section(&mut fptr, 0, data_to_write.len(), &data_to_write)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

[`write_region`][fits-hdu-write-region] takes a slice of ranges with which
the data is to be written, and the data to write.

```rust
# use fitsio::images::{ImageType, ImageDescription};
#
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let desc = ImageDescription {
#    data_type: ImageType::Float,
#    dimensions: &[100, 100],
# };
# let hdu = fptr.create_image("".to_string(), &desc)?;
let data_to_write: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
let ranges = [&(0..1), &(0..1)];
hdu.write_region(&mut fptr, &ranges, &data_to_write)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

_Unlike cfitsio, the order of the ranges follows the C convention, i.e.
[row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order)._

[`write_image`][fits-hdu-write-image] writes all of the data passed (if possible) into the
image. If more data is passed than pixels in the image, the method returns with an error.

```rust
# use fitsio::images::{ImageType, ImageDescription};
#
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let desc = ImageDescription {
#    data_type: ImageType::Float,
#    dimensions: &[3, 1],
# };
# let hdu = fptr.create_image("".to_string(), &desc)?;
// Image is 3x1
assert!(hdu.write_image(&mut fptr, &[1.0, 2.0, 3.0]).is_ok());
assert!(hdu.write_image(&mut fptr, &[1.0, 2.0, 3.0, 4.0]).is_err());
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### Resizing an image

Images can be resized to a new shape using the [`resize`][fits-hdu-resize] method.

The method takes the open [`FitsFile`][fits-file], and an slice of `usize` values. Note:
currently `fitsio` only supports slices with length 2, i.e. a 2D image.
[`resize`][fits-hdu-resize] takes ownership `self` to force the user to fetch the HDU object
again. This ensures the image changes are reflected in the hew HDU object.

```rust
# use std::fs::copy;
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# copy("../testdata/full_example.fits", &filename)?;
# let filename = filename.to_str().unwrap();
# let mut fptr = fitsio::FitsFile::edit(filename)?;
# let hdu = fptr.hdu(0)?;
use fitsio::hdu::HduInfo;

hdu.resize(&mut fptr, &[1024, 1024])?;

// Have to get the HDU again, to reflect the latest changes
let hdu = fptr.hdu(0)?;
match hdu.info {
    HduInfo::ImageInfo { shape, .. } => {
        assert_eq!(shape, [1024, 1024]);
    }
    _ => panic!("Unexpected hdu type"),
}
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

_Unlike cfitsio, the order of the dimensions of `new_size` follows the C convention, i.e.
[row-major order](https://en.wikipedia.org/wiki/Row-_and_column-major_order)._

## Writing tables

### Writing table data

Tablular data can either be written with [`write_col`][fits-hdu-write-col] or
[`write_col_range`][fits-hdu-write-col-range].

[`write_col`][fits-hdu-write-col] writes an entire column's worth of data to the file. It does
not check how many rows are in the file, but extends the table if the length of data is longer
than the table length.

```rust
# use std::fs::copy;
# use fitsio::hdu::HduInfo;
# use fitsio::tables::{ColumnDescription, ColumnDataType};
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let table_description = vec![
#     ColumnDescription::new("bar")
#         .with_type(ColumnDataType::Int)
#         .create()
#         ?,
# ];
# let hdu = fptr.create_table("foo".to_string(), &table_description)
#     ?;
let data_to_write: Vec<i32> = vec![10101; 5];
hdu.write_col(&mut fptr, "bar", &data_to_write)?;
let data: Vec<i32> = hdu.read_col(&mut fptr, "bar")?;
assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

[`write_col_range`][fits-hdu-write-col-range] writes data to a range of rows in a table. The
range is inclusive of both the upper and lower bounds, so `0..4` writes 5 elements.

```rust
# use std::fs::copy;
# use fitsio::hdu::HduInfo;
# use fitsio::tables::{ColumnDescription, ColumnDataType};
# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let table_description = vec![
#     ColumnDescription::new("bar")
#         .with_type(ColumnDataType::Int)
#         .create()?,
# ];
# let hdu = fptr.create_table("foo".to_string(), &table_description)?;
let data_to_write: Vec<i32> = vec![10101; 10];
hdu.write_col_range(&mut fptr, "bar", &data_to_write, &(0..5))?;
let data: Vec<i32> = hdu.read_col(&mut fptr, "bar")?;
assert_eq!(data, vec![10101, 10101, 10101, 10101, 10101]);
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### Inserting columns

Two methods on the HDU object allow for adding new columns:
[`append_column`][fits-hdu-append-column]
and [`insert_column`][fits-hdu-insert-column].
[`append_column`][fits-hdu-append-column] adds a new column as the last column member, and is
generally
preferred as it does not require shifting of data within the file.

```rust
use fitsio::tables::{ColumnDescription, ColumnDataType};

# fn try_main() -> Result<(), Box<std::error::Error>> {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let table_description = &[
#     ColumnDescription::new("bar")
#         .with_type(ColumnDataType::Int)
#         .create()?,
# ];
# let hdu = fptr.create_table("foo".to_string(), table_description)?;
let column_description = ColumnDescription::new("abcdefg")
    .with_type(ColumnDataType::Int)
    .create()?;
hdu.append_column(&mut fptr, &column_description)?;
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

### Deleting columns

The HDU object has the method [`delete_column`][fits-hdu-delete-column] which removes a column.
The column can either be accessed by integer or name

```rust
# use fitsio::tables::{ColumnDescription, ColumnDataType};
# fn try_main() -> Result<(), Box<std::error::Error>> {
# {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let table_description = &[
#     ColumnDescription::new("bar")
#         .with_type(ColumnDataType::Int)
#         .create()?,
# ];
# let hdu = fptr.create_table("foo".to_string(), table_description)?;
let newhdu = hdu.delete_column(&mut fptr, "bar")?;
# }
# {
# let tdir = tempdir::TempDir::new("fitsio-")?;
# let tdir_path = tdir.path();
# let filename = tdir_path.join("test.fits");
# let mut fptr = fitsio::FitsFile::create(filename).open()?;
# let table_description = &[
#     ColumnDescription::new("bar")
#         .with_type(ColumnDataType::Int)
#         .create()?,
# ];
# let hdu = fptr.create_table("foo".to_string(), table_description)?;
// or
let newhdu = hdu.delete_column(&mut fptr, 0)?;
# }
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

# Raw fits file access

If this library does not support the particular use case that is needed, the raw `fitsfile`
pointer can be accessed:

```rust
# #[cfg(not(feature="bindgen"))]
# use fitsio_sys;
# #[cfg(feature="bindgen")]
# use fitsio_sys_bindgen as fitsio_sys;

use fitsio::FitsFile;

# fn try_main() -> Result<(), Box<dyn std::error::Error>> {
# let filename = "../testdata/full_example.fits";
let mut fptr = FitsFile::open(filename)?;

/* Find out the number of HDUs in the file */
let mut num_hdus = 0;
let mut status = 0;

unsafe {
let fitsfile = fptr.as_raw();

/* Use the unsafe fitsio-sys low level library to call a function that is possibly not
implemented in this crate */
fitsio_sys::ffthdu(fitsfile, &mut num_hdus, &mut status);
}
assert_eq!(num_hdus, 2);
# Ok(())
# }
# fn main() { try_main().unwrap(); }
```

This (unsafe) pointer can then be used with the underlying [`fitsio-sys`][fitsio-sys] library directly.

# Threadsafe access

Access to a [`FitsFile`][fits-file] is not threadsafe. Behind the scenes, fetching a
[`FitsHdu`][fits-hdu] changes internal state, and `fitsio` does not provide any concurrent access
gauruntees. Therefore, a [`FitsFile`][fits-file] does not implement `Send` or `Sync`.

In order to allow for threadsafe access, the [`FitsFile`][fits-file] struct has a
[`threadsafe`][fits-file-threadsafe] method, which returns a threadsafe
[`ThreadsafeFitsFile`][threadsafe-fits-file] struct (a tuple-type wrapper around
`Arc<Mutex<FitsFile>>`) which can be shared between threads safely.

The same concerns with `Arc<Mutex<T>>` data should be applied here. Additionally, the library is
subject to OS level limits, such as the maximum number of open files.

## Example

```rust
# use fitsio::FitsFile;
# use std::thread;
# let fptr = FitsFile::open("../testdata/full_example.fits").unwrap();
let f = fptr.threadsafe();

/* Spawn loads of threads... */
# let mut handles = Vec::new();
for i in 0..10_000 {
let mut f1 = f.clone();

/* Send the cloned ThreadsafeFitsFile to another thread */
let handle = thread::spawn(move || {
/* Get the underlyng fits file back */
let mut t = f1.lock().unwrap();

/* Fetch a different HDU per thread */
let hdu_num = i % 2;
let _hdu = t.hdu(hdu_num).unwrap();
});
# handles.push(handle);
}
#
# /* Wait for all of the threads to finish */
# for handle in handles {
#     handle.join().unwrap();
# }
```

[cfitsio]: http://heasarc.gsfc.nasa.gov/fitsio/fitsio.html
[fitsio-sys]: https://crates.io/crates/fitsio-sys
[column-data-description]: tables/struct.ColumnDataDescription.html
[column-data-type]: tables/enum.ColumnDataType.html
[column-description]: tables/struct.ColumnDescription.html
[fits-file-create-image]: fitsfile/struct.FitsFile.html#method.create_image
[fits-file-create-table]: fitsfile/struct.FitsFile.html#method.create_table
[fits-file-create]: fitsfile/struct.FitsFile.html#method.create
[fits-file-edit]: fitsfile/struct.FitsFile.html#method.edit
[fits-file-threadsafe]: fitsfile/struct.FitsFile.html#method.threadsafe
[fits-file]: fitsfile/struct.FitsFile.html
[fits-hdu]: hdu/struct.FitsHdu.html
[fits-hdu-append-column]: hdu/struct.FitsHdu.html#method.append_column
[fits-hdu-columns]: hdu/struct.FitsHdu.html#method.columns
[fits-hdu-delete-column]: hdu/struct.FitsHdu.html#method.delete_column
[fits-hdu-insert-column]: hdu/struct.FitsHdu.html#method.insert_column
[fits-hdu-read-col]: hdu/struct.FitsHdu.html#method.read_col
[fits-hdu-read-key]: hdu/struct.FitsHdu.html#method.read_key
[fits-hdu-read-region]: hdu/struct.FitsHdu.html#method.read_region
[fits-hdu-read-section]: hdu/struct.FitsHdu.html#method.read_section
[fits-hdu-write-key]: hdu/struct.FitsHdu.html#method.write_key
[fits-hdu-write-col]: hdu/struct.FitsHdu.html#method.write_col
[fits-hdu-write-col-range]: hdu/struct.FitsHdu.html#method.write_col_range
[fits-hdu-write-region]: hdu/struct.FitsHdu.html#method.write_region
[fits-hdu-write-image]: hdu/struct.FitsHdu.html#method.write_image
[fits-hdu-write-section]: hdu/struct.FitsHdu.html#method.write_section
[fits-hdu-iter]: hdu/struct.FitsHdu.html#method.iter
[fits-hdu-copy-to]: hdu/struct.FitsHdu.html#method.copy_to
[fits-hdu-delete]: hdu/struct.FitsHdu.html#method.copy_to
[fits-hdu-resize]: hdu/struct.FitsHdu.html#method.resize
[fits-hdu-row]: hdu/struct.FitsHdu.html#method.row
[image-description]: images/struct.ImageDescription.html
[reads-col]: tables/trait.ReadsCol.html
[reads-key]: headers/trait.ReadsKey.html
[writes-key]: headers/trait.ReadsKey.html
[new-fits-file]: fitsfile/struct.NewFitsFile.html
[new-fits-file-open]: fitsfile/struct.NewFitsFile.html#method.open
[new-fits-file-with-custom-primary]: fitsfile/struct.NewFitsFile.html#method.with_custom_primary
[pretty-print]: fitsfile/struct.FitsFile.html#method.pretty_print
[pretty-write]: fitsfile/struct.FitsFile.html#method.pretty_write
[fitsio-derive]: https://crates.io/crates/fitsio-derive
[ndarray]: https://crates.io/crates/ndarray
[arrayd]: https://docs.rs/ndarray/0.11.2/ndarray/type.ArrayD.html
[fitsfile-open]: fitsfile/struct.FitsFile.html#method.open
[`fitssummary`]: ../fitssummary/index.html
[fitsfile-hdu]: fitsfile/struct.FitsFile.html#method.hdu
[threadsafe-fits-file]: threadsafe_fitsfile/struct.ThreadsafeFitsFile.html
*/

#![doc(html_root_url = "https://docs.rs/fitsio/0.15.0")]
#![deny(missing_docs)]
#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]

// If we are using the `bindgen` feature then import `fitsio_sys_bindgen` with a new name
#[cfg(feature = "default")]
use fitsio_sys;
#[cfg(feature = "bindgen")]
use fitsio_sys_bindgen as fitsio_sys;

#[macro_use]
mod macros;
mod fitsfile;
mod longnam;
#[cfg(feature = "array")]
mod ndarray_compat;
mod stringutils;
#[cfg(test)]
mod testhelpers;
mod types;

// Public mods
pub mod hdu;
pub mod headers;
pub mod images;
pub mod tables;
pub mod threadsafe_fitsfile;

pub mod errors;

// Re-exports
pub use crate::fitsfile::FitsFile;

// For custom derive purposes
// pub use tables::FitsRow;
