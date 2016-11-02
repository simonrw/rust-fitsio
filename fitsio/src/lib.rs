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
//! Information about the current HDU can be fetched with various functions, for example getting
//! the current HDU type:
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
//!
//! match fptr.hdu_type() {
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
//! // image HDU
//! if let Ok(HduInfo::ImageInfo { dimensions, shape }) = fptr.fetch_hdu_info() {
//!    println!("Image is {}-dimensional", dimensions);
//!    println!("Found image with shape {:?}", shape);
//! }
//! # fptr.change_hdu("TESTEXT").unwrap();
//!
//! // tables
//! if let Ok(HduInfo::TableInfo { column_names, num_rows, .. }) = fptr.fetch_hdu_info() {
//!     println!("Table contains {} rows", num_rows);
//!     println!("Table has {} columns", column_names.len());
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
//! let int_value: i64 = fptr.read_key("INTTEST").unwrap();
//! # }
//!
//! // Alternatively
//! # {
//! let int_value = fptr.read_key::<i64>("INTTEST").unwrap();
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
//! fptr.write_key("foo", 1i64).unwrap();
//! assert_eq!(fptr.read_key::<i64>("foo").unwrap(), 1i64);
//! # }
//! ```
//!
//! ## Reading file data
//!
//! ### Images
//!
//! Image data can be read through either
//! [`read_section`](struct.FitsFile.html#method.read_section) which reads contiguous pixels
//! between a start index and end index, or
//! [`read_region`](struct.FitsFile.html#method.read_region) which reads rectangular chunks from
//! the image.
//!
//! ```rust
//! # extern crate fitsio;
//! #
//! # fn main() {
//! # let filename = "../testdata/full_example.fits";
//! # let fptr = fitsio::FitsFile::open(filename).unwrap();
//! // Read the first 100 pixels
//! let first_row: Vec<i32> = fptr.read_section(0, 100).unwrap();
//!
//! // Read a square section of the image
//! use fitsio::positional::Coordinate;
//!
//! let lower_left = Coordinate { x: 0, y: 0 };
//! let upper_right = Coordinate { x: 10, y: 10 };
//! let chunk: Vec<i32> = fptr.read_region(&lower_left, &upper_right).unwrap();
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
//! let integer_data: Vec<i32> = fptr.read_col("intcol").unwrap();
//! # }
//! ```
//!
//! The [`columns`](struct.FitsFile.html#method.columns) method returns an iterator over all of the
//! columns in a table.
//!
//! [1]: http://heasarc.gsfc.nasa.gov/fitsio/fitsio.html
//! [2]: https://crates.io/crates/fitsio-sys

mod stringutils;
pub mod positional;

extern crate fitsio_sys as sys;
extern crate libc;

use std::ptr;
use std::ffi;

use positional::Coordinate;

/// Error type
///
/// `cfitsio` passes errors through integer status codes. This struct wraps this and its associated
/// error message.
#[derive(Debug, PartialEq, Eq)]
pub struct FitsError {
    status: i32,
    message: String,
}

/// Macro for returning a FITS error type
macro_rules! fits_try {
    ($status: ident, $e: expr) => {
        match $status {
            0 => Ok($e),
            _ => {
                Err(FitsError {
                    status: $status,
                    message: stringutils::status_to_string($status).unwrap(),
                })
            }
        }
    }
}

/// FITS specific result type
///
/// This is a shortcut for a result with `FitsError` as the error type
pub type Result<T> = std::result::Result<T, FitsError>;

/// Hdu description type
///
/// Any way of describing a HDU - number or string which either
/// changes the hdu by absolute number, or by name.
pub trait DescribesHdu {
    fn change_hdu(&self, fptr: &FitsFile) -> Result<()>;
}

impl DescribesHdu for usize {
    fn change_hdu(&self, f: &FitsFile) -> Result<()> {
        let mut _hdu_type = 0;
        let mut status = 0;
        unsafe {
            sys::ffmahd(f.fptr, (*self + 1) as i32, &mut _hdu_type, &mut status);
        }

        fits_try!(status, ())
    }
}

impl<'a> DescribesHdu for &'a str {
    fn change_hdu(&self, f: &FitsFile) -> Result<()> {
        let mut _hdu_type = 0;
        let mut status = 0;
        let c_hdu_name = ffi::CString::new(*self).unwrap();

        unsafe {
            sys::ffmnhd(f.fptr,
                        sys::HduType::ANY_HDU.into(),
                        c_hdu_name.into_raw(),
                        0,
                        &mut status);
        }

        fits_try!(status, ())
    }
}


/// Trait applied to types which can be read from a FITS header
///
/// This is currently:
///
/// * i32
/// * i64
/// * f32
/// * f64
/// * String
pub trait ReadsKey {
    fn read_key(f: &FitsFile, name: &str) -> Result<Self> where Self: Sized;
}

macro_rules! reads_key_impl {
    ($t:ty, $func:ident) => (
        impl ReadsKey for $t {
            fn read_key(f: &FitsFile, name: &str) -> Result<Self> {
                let c_name = ffi::CString::new(name).unwrap();
                let mut status = 0;
                let mut value: Self = Self::default();

                unsafe {
                    sys::$func(f.fptr,
                           c_name.into_raw(),
                           &mut value,
                           ptr::null_mut(),
                           &mut status);
                }

                fits_try!(status, value)
            }
        }
    )
}

reads_key_impl!(i32, ffgkyl);
reads_key_impl!(i64, ffgkyj);
reads_key_impl!(f32, ffgkye);
reads_key_impl!(f64, ffgkyd);

impl ReadsKey for String {
    fn read_key(f: &FitsFile, name: &str) -> Result<Self> {
        let c_name = ffi::CString::new(name).unwrap();
        let mut status = 0;
        let mut value: Vec<libc::c_char> = vec![0; sys::MAX_VALUE_LENGTH];

        unsafe {
            sys::ffgkys(f.fptr,
                        c_name.into_raw(),
                        value.as_mut_ptr(),
                        ptr::null_mut(),
                        &mut status);
        }

        fits_try!(status, {
            let value: Vec<u8> = value.iter()
                .map(|&x| x as u8)
                .filter(|&x| x != 0)
                .collect();
            String::from_utf8(value).unwrap()
        })
    }
}

/// Writing a fits keyword
pub trait WritesKey {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()>;
}

macro_rules! writes_key_impl_flt {
    ($t:ty, $func:ident) => (
        impl WritesKey for $t {
            fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name).unwrap();
                let mut status = 0;

                unsafe {
                    sys::$func(f.fptr,
                                c_name.into_raw(),
                                value,
                                9,
                                ptr::null_mut(),
                                &mut status);
                }
                fits_try!(status, ())
            }
        }
    )
}

impl WritesKey for i64 {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name).unwrap();
        let mut status = 0;

        unsafe {
            sys::ffpkyj(f.fptr,
                        c_name.into_raw(),
                        value,
                        ptr::null_mut(),
                        &mut status);
        }
        fits_try!(status, ())
    }
}

writes_key_impl_flt!(f32, ffpkye);
writes_key_impl_flt!(f64, ffpkyd);

impl WritesKey for String {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name).unwrap();
        let mut status = 0;

        unsafe {
            sys::ffpkys(f.fptr,
                        c_name.into_raw(),
                        ffi::CString::new(value).unwrap().into_raw(),
                        ptr::null_mut(),
                        &mut status);
        }

        fits_try!(status, ())
    }
}

/// Trait for reading a fits column
pub trait ReadsCol {
    fn read_col(fits_file: &FitsFile, name: &str) -> Result<Vec<Self>> where Self: Sized;
}

macro_rules! reads_col_impl {
    ($t: ty, $func: ident, $nullval: expr) => (
        impl ReadsCol for $t {
            fn read_col(fits_file: &FitsFile, name: &str) -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo {
                        column_names, num_rows, ..
                    }) => {
                        let mut out = vec![$nullval; num_rows];
                        assert_eq!(out.len(), num_rows);
                        let column_number = column_names.iter().position(|ref colname| {
                            colname.as_str() == name
                        }).unwrap();
                        let mut status = 0;
                        unsafe {
                            sys::$func(fits_file.fptr,
                                       (column_number + 1) as i32,
                                       1,
                                       1,
                                       num_rows as i64,
                                       $nullval,
                                       out.as_mut_ptr(),
                                       ptr::null_mut(),
                                       &mut status);

                        }
                        fits_try!(status, out)
                    },
                    Err(e) => Err(e),
                    _ => panic!("Unknown error occurred"),
                }
            }
        }
    )
}

reads_col_impl!(i32, ffgcvk, 0);
reads_col_impl!(u32, ffgcvuk, 0);
reads_col_impl!(i64, ffgcvj, 0);
reads_col_impl!(u64, ffgcvuj, 0);
reads_col_impl!(f32, ffgcve, 0.0);
reads_col_impl!(f64, ffgcvd, 0.0);

// TODO: impl for string

/// Reading fits images
pub trait ReadsImage {
    fn read_section(fits_file: &FitsFile, start: usize, end: usize) -> Result<Vec<Self>>
        where Self: Sized;

    /// Read a square region from the chip.
    ///
    /// Lower left indicates the starting point of the square, and the upper
    /// right defines the pixel _beyond_ the end. The range of pixels included
    /// is inclusive of the lower end, and *exclusive* of the upper end.
    fn read_region(fits_file: &FitsFile,
                   lower_left: &Coordinate,
                   upper_right: &Coordinate)
                   -> Result<Vec<Self>>
        where Self: Sized;
}

macro_rules! reads_image_impl {
    ($t: ty, $data_type: expr) => (
        impl ReadsImage for $t {
            fn read_section(fits_file: &FitsFile, start: usize, end: usize) -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { dimensions: _dimensions, shape: _shape }) => {
                        let nelements = end - start;
                        let mut out = vec![0 as $t; nelements];
                        let mut status = 0;

                        unsafe {
                            sys::ffgpv(fits_file.fptr,
                                        $data_type.into(),
                                        (start + 1) as i64,
                                        nelements as i64,
                                        ptr::null_mut(),
                                        out.as_mut_ptr() as *mut libc::c_void,
                                        ptr::null_mut(),
                                        &mut status);
                        }

                        fits_try!(status, out)

                    }
                    Err(e) => Err(e),
                    _ => panic!("Unknown error occurred"),
                }
            }

            fn read_region( fits_file: &FitsFile, lower_left: &Coordinate, upper_right: &Coordinate)
                -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { dimensions: _dimensions, shape: _shape }) => {
                        // TODO: check dimensions

                        // These have to be mutable because of the C-api
                        let mut fpixel = [ (lower_left.x + 1) as _, (lower_left.y + 1) as _ ];
                        let mut lpixel = [ (upper_right.x + 1) as _, (upper_right.y + 1) as _ ];
                        let mut inc = [ 1, 1 ];
                        let nelements =
                            ((upper_right.y - lower_left.y) + 1) *
                            ((upper_right.x - lower_left.x) + 1);
                        let mut out = vec![0 as $t; nelements as usize];
                        let mut status = 0;

                        unsafe {
                            sys::ffgsv(
                                fits_file.fptr,
                                $data_type.into(),
                                fpixel.as_mut_ptr(),
                                lpixel.as_mut_ptr(),
                                inc.as_mut_ptr(),
                                ptr::null_mut(),
                                out.as_mut_ptr() as *mut libc::c_void,
                                ptr::null_mut(),
                                &mut status);

                        }

                        fits_try!(status, out)
                    }
                    Err(e) => Err(e),
                    _ => panic!("Unknown error occurred"),
                }
            }
        }
    )
}


reads_image_impl!(i8, sys::DataType::TSHORT);
reads_image_impl!(i32, sys::DataType::TINT);
reads_image_impl!(i64, sys::DataType::TLONG);
reads_image_impl!(u8, sys::DataType::TUSHORT);
reads_image_impl!(u32, sys::DataType::TUINT);
reads_image_impl!(u64, sys::DataType::TULONG);
reads_image_impl!(f32, sys::DataType::TFLOAT);
reads_image_impl!(f64, sys::DataType::TDOUBLE);

/// Description of the current HDU
///
/// If the current HDU is an image, then
/// [`fetch_hdu_info`](struct.FitsFile.html#method.fetch_hdu_info) returns `HduInfo::ImageInfo`.
/// Otherwise the variant is `HduInfo::TableInfo`.
#[derive(PartialEq, Eq, Debug)]
pub enum HduInfo {
    ImageInfo {
        dimensions: usize,
        shape: Vec<usize>,
    },
    TableInfo {
        column_names: Vec<String>,
        column_types: Vec<sys::DataType>,
        num_rows: usize,
    },
}

/// Main entry point to the FITS file format
///
///
pub struct FitsFile {
    fptr: *mut sys::fitsfile,
    pub filename: String,
}

impl Clone for FitsFile {
    fn clone(&self) -> Self {
        FitsFile::open(&self.filename).unwrap()
    }
}

fn typechar_to_data_type<T: Into<String>>(typechar: T) -> sys::DataType {
    match typechar.into().as_str() {
        "X" => sys::DataType::TBIT,
        "B" => sys::DataType::TBYTE,
        "L" => sys::DataType::TLOGICAL,
        "A" => sys::DataType::TSTRING,
        "I" => sys::DataType::TSHORT,
        "J" => sys::DataType::TLONG,
        "E" => sys::DataType::TFLOAT,
        "D" => sys::DataType::TDOUBLE,
        "C" => sys::DataType::TCOMPLEX,
        "M" => sys::DataType::TDBLCOMPLEX,
        "K" => sys::DataType::TLONGLONG,
        other => panic!("Unhandled case: {}", other),
    }
}

unsafe fn fetch_hdu_info(fptr: *mut sys::fitsfile) -> Result<HduInfo> {
    let mut status = 0;
    let mut hdu_type = 0;

    sys::ffghdt(fptr, &mut hdu_type, &mut status);
    let hdu_type = match hdu_type {
        0 => {
            let mut dimensions = 0;
            sys::ffgidm(fptr, &mut dimensions, &mut status);

            let mut shape = vec![0; dimensions as usize];
            sys::ffgisz(fptr, dimensions, shape.as_mut_ptr(), &mut status);

            HduInfo::ImageInfo {
                dimensions: dimensions as usize,
                shape: shape.iter().map(|v| *v as usize).collect(),
            }
        }
        1 | 2 => {
            let mut num_rows = 0;
            sys::ffgnrw(fptr, &mut num_rows, &mut status);

            let mut num_cols = 0;
            sys::ffgncl(fptr, &mut num_cols, &mut status);
            let mut column_names = Vec::with_capacity(num_cols as usize);
            let mut column_types = Vec::with_capacity(num_cols as usize);

            for i in 0..num_cols {
                let mut name_buffer: Vec<libc::c_char> = vec![0; 71];
                let mut type_buffer: Vec<libc::c_char> = vec![0; 71];
                sys::ffgbcl(fptr,
                            (i + 1) as i32,
                            name_buffer.as_mut_ptr(),
                            ptr::null_mut(),
                            type_buffer.as_mut_ptr(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            ptr::null_mut(),
                            &mut status);
                column_names.push(stringutils::buf_to_string(&name_buffer).unwrap());
                column_types.push(typechar_to_data_type(stringutils::buf_to_string(&type_buffer)
                            .unwrap()));
            }

            HduInfo::TableInfo {
                column_names: column_names,
                column_types: column_types,
                num_rows: num_rows as usize,
            }
        }
        _ => panic!("Invalid hdu type found"),
    };

    fits_try!(status, hdu_type)
}

pub enum Column {
    Int32 {
        name: String,
        data: Vec<i32>,
    },
    Int64 {
        name: String,
        data: Vec<i64>,
    },
    Float {
        name: String,
        data: Vec<f32>,
    },
    Double {
        name: String,
        data: Vec<f64>,
    },
}

pub struct ColumnIterator<'a> {
    current: usize,
    column_names: Vec<String>,
    column_types: Vec<sys::DataType>,
    fits_file: &'a FitsFile,
}

impl<'a> ColumnIterator<'a> {
    fn new(fits_file: &'a FitsFile) -> Self {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { column_names, column_types, num_rows: _num_rows }) => {
                ColumnIterator {
                    current: 0,
                    column_names: column_names,
                    column_types: column_types,
                    fits_file: fits_file,
                }
            }
            Err(e) => panic!("{:?}", e),
            _ => panic!("Unknown error occurred"),
        }
    }
}

impl<'a> Iterator for ColumnIterator<'a> {
    type Item = Column;

    fn next(&mut self) -> Option<Self::Item> {
        let ncols = self.column_names.len();

        if self.current < ncols {
            let current_name = &self.column_names[self.current];
            let current_type = &self.column_types[self.current];

            let retval = match *current_type {
                sys::DataType::TSHORT => {
                    i32::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Int32 {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                sys::DataType::TLONG => {
                    i64::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Int64 {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                sys::DataType::TFLOAT => {
                    f32::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Float {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                sys::DataType::TDOUBLE => {
                    f64::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Double {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                _ => unimplemented!(),
            };

            self.current += 1;

            retval

        } else {
            None
        }
    }
}

impl FitsFile {
    /// Open a fits file from disk
    ///
    /// # Examples
    ///
    /// ```
    /// use fitsio::FitsFile;
    ///
    /// let f = FitsFile::open("../testdata/full_example.fits").unwrap();
    ///
    /// // Continue to use `f` afterwards
    /// ```
    pub fn open(filename: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(filename).unwrap();

        unsafe {
            sys::ffopen(&mut fptr as *mut *mut sys::fitsfile,
                        c_filename.as_ptr(),
                        sys::FileOpenMode::READONLY as libc::c_int,
                        &mut status);
        }

        fits_try!(status,
                  FitsFile {
                      fptr: fptr,
                      filename: filename.to_string(),
                  })
    }

    /// Create a new fits file on disk
    pub fn create(path: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(path).unwrap();

        unsafe {
            sys::ffinit(&mut fptr as *mut *mut sys::fitsfile,
                        c_filename.as_ptr(),
                        &mut status);
        }

        fits_try!(status, {
            let f = FitsFile {
                fptr: fptr,
                filename: path.to_string(),
            };
            try!(f.add_empty_primary());
            f
        })
    }

    fn add_empty_primary(&self) -> Result<()> {
        let mut status = 0;
        unsafe {
            sys::ffphps(self.fptr, 8, 0, ptr::null_mut(), &mut status);
        }

        fits_try!(status, ())
    }

    /// Change the current HDU
    pub fn change_hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self)
    }

    /// Get the current HDU type
    pub fn hdu_type(&self) -> Result<sys::HduType> {
        let mut status = 0;
        let mut hdu_type = 0;
        unsafe {
            sys::ffghdt(self.fptr, &mut hdu_type, &mut status);
        }

        fits_try!(status, {
            match hdu_type {
                0 => sys::HduType::IMAGE_HDU,
                2 => sys::HduType::BINARY_TBL,
                _ => unimplemented!(),
            }
        })
    }

    pub fn hdu_number(&self) -> usize {
        let mut hdu_num = 0;
        unsafe {
            sys::ffghdn(self.fptr, &mut hdu_num);
        }
        (hdu_num - 1) as usize
    }

    /// Read header key
    pub fn read_key<T: ReadsKey>(&self, name: &str) -> Result<T> {
        T::read_key(self, name)
    }

    /// Write header key
    pub fn write_key<T: WritesKey>(&self, name: &str, value: T) -> Result<()> {
        T::write_key(self, name, value)
    }

    /// Read a binary table column
    pub fn read_col<T: ReadsCol>(&self, name: &str) -> Result<Vec<T>> {
        T::read_col(self, name)
    }

    /// Read an image between pixel a and pixel b into a `Vec`
    pub fn read_section<T: ReadsImage>(&self, start: usize, end: usize) -> Result<Vec<T>> {
        T::read_section(self, start, end)
    }

    /// Read a square region into a `Vec`
    pub fn read_region<T: ReadsImage>(&self,
                                      lower_left: &Coordinate,
                                      upper_right: &Coordinate)
                                      -> Result<Vec<T>> {
        T::read_region(self, lower_left, upper_right)
    }

    pub fn columns(&self) -> ColumnIterator {
        ColumnIterator::new(self)
    }

    /// Get the current hdu info
    pub fn fetch_hdu_info(&self) -> Result<HduInfo> {
        unsafe { fetch_hdu_info(self.fptr) }
    }
}

impl Drop for FitsFile {
    fn drop(&mut self) {
        let mut status = 0;
        unsafe {
            sys::ffclos(self.fptr, &mut status);
        }
    }
}

#[cfg(test)]
mod test {
    extern crate tempdir;
    use super::*;
    use sys;
    use super::typechar_to_data_type;

    #[test]
    fn typechar_conversions() {
        let input = vec![
            "X",
            "B",
            "L",
            "A",
            "I",
            "J",
            "E",
            "D",
            "C",
            "M",
        ];
        let expected = vec![
            sys::DataType::TBIT,
            sys::DataType::TBYTE,
            sys::DataType::TLOGICAL,
            sys::DataType::TSTRING,
            sys::DataType::TSHORT,
            sys::DataType::TLONG,
            sys::DataType::TFLOAT,
            sys::DataType::TDOUBLE,
            sys::DataType::TCOMPLEX,
            sys::DataType::TDBLCOMPLEX,
        ];

        input.iter()
            .zip(expected)
            .map(|(&i, e)| {
                assert_eq!(typechar_to_data_type(i), e);
            })
            .collect::<Vec<_>>();
    }

    #[test]
    fn opening_an_existing_file() {
        match FitsFile::open("../testdata/full_example.fits") {
            Ok(_) => {}
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn creating_a_new_file() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        assert!(!filename.exists());

        FitsFile::create(filename.to_str().unwrap())
            .map(|f| {
                assert!(filename.exists());

                // Ensure the empty primary has been written
                let naxis: i64 = f.read_key("NAXIS").unwrap();
                assert_eq!(naxis, 0);
            })
            .unwrap();
    }

    #[test]
    fn fetching_a_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        for i in 0..2 {
            f.change_hdu(i).unwrap();
            assert_eq!(f.hdu_number(), i);
        }

        match f.change_hdu(2) {
            Err(e) => assert_eq!(e.status, 107),
            _ => panic!("Error checking for failure"),
        }

        f.change_hdu("TESTEXT").unwrap();
        assert_eq!(f.hdu_number(), 1);
    }

    #[test]
    fn reading_header_keys() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        match f.read_key::<i64>("INTTEST") {
            Ok(value) => assert_eq!(value, 42),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match f.read_key::<f64>("DBLTEST") {
            Ok(value) => assert_eq!(value, 0.09375),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match f.read_key::<String>("TEST") {
            Ok(value) => assert_eq!(value, "value"),
            Err(e) => panic!("Error reading key: {:?}", e),
        }
    }

    #[test]
    fn getting_hdu_type() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        assert_eq!(f.hdu_type().unwrap(), sys::HduType::IMAGE_HDU);

        f.change_hdu("TESTEXT").unwrap();
        assert_eq!(f.hdu_type().unwrap(), sys::HduType::BINARY_TBL);
    }

    #[test]
    fn fetching_hdu_info() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { dimensions, shape }) => {
                assert_eq!(dimensions, 2);
                assert_eq!(shape, vec![100, 100]);
            }
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }

        f.change_hdu(1).unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { column_names, column_types, num_rows }) => {
                assert_eq!(num_rows, 50);
                assert_eq!(column_names,
                           vec![
                           "intcol".to_string(),
                           "floatcol".to_string(),
                           "doublecol".to_string(),
                ]);
                assert_eq!(column_types,
                           vec![
                        sys::DataType::TLONG,
                        sys::DataType::TFLOAT,
                        sys::DataType::TDOUBLE,
                ]);
            }
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }
    }

    #[test]
    fn read_columns() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu(1).unwrap();
        let intcol_data: Vec<i32> = f.read_col("intcol").unwrap();
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[15], 10);
        assert_eq!(intcol_data[49], 12);

        let floatcol_data: Vec<f32> = f.read_col("floatcol").unwrap();
        assert_eq!(floatcol_data[0], 17.496801);
        assert_eq!(floatcol_data[15], 19.570272);
        assert_eq!(floatcol_data[49], 10.217053);

        let doublecol_data: Vec<f64> = f.read_col("doublecol").unwrap();
        assert_eq!(doublecol_data[0], 16.959972808730814);
        assert_eq!(doublecol_data[15], 19.013522579233065);
        assert_eq!(doublecol_data[49], 16.61153656123406);
    }

    #[test]
    fn read_image_data() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let first_row: Vec<i32> = f.read_section(0, 100).unwrap();
        assert_eq!(first_row.len(), 100);
        assert_eq!(first_row[0], 108);
        assert_eq!(first_row[49], 176);

        let second_row: Vec<i32> = f.read_section(100, 200).unwrap();
        assert_eq!(second_row.len(), 100);
        assert_eq!(second_row[0], 177);
        assert_eq!(second_row[49], 168);
    }

    #[test]
    fn read_image_slice() {
        use super::positional::Coordinate;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let lower_left = Coordinate { x: 0, y: 0 };
        let upper_right = Coordinate { x: 10, y: 10 };
        let chunk: Vec<i32> = f.read_region(&lower_left, &upper_right).unwrap();
        assert_eq!(chunk.len(), 11 * 11);
        assert_eq!(chunk[0], 108);
        assert_eq!(chunk[11], 177);
        assert_eq!(chunk[chunk.len() - 1], 160);
    }

    #[test]
    fn cloning() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let f2 = f.clone();

        assert!(f.fptr != f2.fptr);

        f.change_hdu(1).unwrap();
        assert!(f.hdu_number() != f2.hdu_number());
    }

    #[test]
    fn test_fits_try() {
        use super::stringutils;

        let status = 0;
        assert_eq!(fits_try!(status, 10), Ok(10));

        let status = 105;
        assert_eq!(fits_try!(status, 10),
                   Err(FitsError {
                       status: status,
                       message: stringutils::status_to_string(status).unwrap(),
                   }));
    }

    #[test]
    fn column_iterator() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        f.change_hdu(1).unwrap();
        let column_names: Vec<String> = f.columns()
            .map(|col| {
                match col {
                    Column::Int32 { name, data: _data } => name,
                    Column::Int64 { name, data: _data } => name,
                    Column::Float { name, data: _data } => name,
                    Column::Double { name, data: _data } => name,
                }
            })
            .collect();
        assert_eq!(column_names,
                   vec!["intcol".to_string(), "floatcol".to_string(), "doublecol".to_string()]);
    }

    // Writing data
    #[test]
    fn writing_header_keywords() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        // Closure ensures file is closed properly
        {
            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            f.write_key("FOO", 1i64).unwrap();
            f.write_key("BAR", "baz".to_string()).unwrap();
        }

        FitsFile::open(filename.to_str().unwrap())
            .map(|f| {
                assert_eq!(f.read_key::<i64>("FOO").unwrap(), 1);
                assert_eq!(f.read_key::<String>("BAR").unwrap(), "baz".to_string());
            })
            .unwrap();
    }

}
