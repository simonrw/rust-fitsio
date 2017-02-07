use super::fitsfile::{FitsFile, HduInfo};
use super::sys;
use super::stringutils;
use super::fitserror::{FitsError, Result};
use super::columndescription::ColumnDescription;
use super::conversions::typechar_to_data_type;
use super::libc;
use super::positional::Coordinate;
use super::types::{HduType, DataType, CaseSensitivity};
use std::ffi;
use std::ptr;
use std::ops::Range;

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
            sys::ffmahd(f.fptr as *mut _,
                        (*self + 1) as i32,
                        &mut _hdu_type,
                        &mut status);
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
            sys::ffmnhd(f.fptr as *mut _,
                        HduType::ANY_HDU.into(),
                        c_hdu_name.into_raw(),
                        0,
                        &mut status);
        }

        fits_try!(status, ())
    }
}

/// Trait for reading a fits column
pub trait ReadsCol {
    fn read_col<T: Into<String>>(fits_file: &FitsFile, name: T) -> Result<Vec<Self>>
        where Self: Sized;
    fn read_col_range<T: Into<String>>(fits_file: &FitsFile,
                                       name: T,
                                       range: &Range<usize>)
                                       -> Result<Vec<Self>>
        where Self: Sized;
}

macro_rules! reads_col_impl {
    ($t: ty, $func: ident, $nullval: expr) => (
        impl ReadsCol for $t {
            fn read_col<T: Into<String>>(fits_file: &FitsFile, name: T) -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo {
                        column_descriptions, num_rows, ..
                    }) => {
                        let mut out = vec![$nullval; num_rows];
                        let test_name = name.into();
                        assert_eq!(out.len(), num_rows);
                        let column_number = column_descriptions.iter().position(|ref desc| {
                            desc.name == test_name
                        }).unwrap();
                        let mut status = 0;
                        unsafe {
                            sys::$func(fits_file.fptr as *mut _,
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

            // TODO: should we check the bounds? cfitsio will raise an error, but we
            // could be more friendly and raise our own?
            fn read_col_range<T: Into<String>>(fits_file: &FitsFile, name: T, range: &Range<usize>)
                -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::TableInfo { column_descriptions, .. }) => {
                        let num_output_rows = range.end - range.start + 1;
                        let mut out = vec![$nullval; num_output_rows];
                        let test_name = name.into();
                        let column_number = column_descriptions.iter().position(|ref desc| {
                            desc.name == test_name
                        }).unwrap();
                        let mut status = 0;
                        unsafe {
                            sys::$func(fits_file.fptr as *mut _,
                                       (column_number + 1) as i32,
                                       (range.start + 1) as i64,
                                       1,
                                       num_output_rows as _,
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

pub trait WritesCol {
    fn write_col<T: Into<String>>(fits_file: &FitsFile,
                                  hdu: &FitsHdu,
                                  col_name: T,
                                  col_data: &[Self])
                                  -> Result<()>
        where Self: Sized;
    fn write_col_range<T: Into<String>>(fits_file: &FitsFile,
                                        hdu: &FitsHdu,
                                        col_name: T,
                                        col_data: &[Self],
                                        rows: &Range<usize>)
                                        -> Result<()>
        where Self: Sized;
}

macro_rules! writes_col_impl {
    ($t: ty, $data_type: expr) => (
        impl WritesCol for $t {
            fn write_col<T: Into<String>>(
                fits_file: &FitsFile,
                hdu: &FitsHdu,
                col_name: T,
                col_data: &[Self]) -> Result<()> {
                let colno = hdu.get_column_no(col_name.into())?;
                let mut status = 0;
                unsafe {
                    sys::ffpcl(
                        fits_file.fptr as *mut _,
                        $data_type.into(),
                        (colno + 1) as _,
                        1,
                        1,
                        col_data.len() as _,
                        col_data.as_ptr() as *mut _,
                        &mut status);
                }
                fits_try!(status, ())
            }

            fn write_col_range<T: Into<String>>(fits_file: &FitsFile,
                hdu: &FitsHdu,
                col_name: T,
                col_data: &[Self],
                rows: &Range<usize>)
            -> Result<()> {
                let colno = hdu.get_column_no(col_name.into())?;
                let mut status = 0;
                unsafe {
                    sys::ffpcl(
                        fits_file.fptr as *mut _,
                        $data_type.into(),
                        (colno + 1) as _,
                        (rows.start + 1) as _,
                        1,
                        (rows.end + 1) as _,
                        col_data.as_ptr() as *mut _,
                        &mut status
                    );
                }
                fits_try!(status, ())
            }
        }
    )
}

writes_col_impl!(u32, DataType::TUINT);
writes_col_impl!(u64, DataType::TULONG);
writes_col_impl!(i32, DataType::TINT);
writes_col_impl!(i64, DataType::TLONG);
writes_col_impl!(f32, DataType::TFLOAT);
writes_col_impl!(f64, DataType::TDOUBLE);

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
                    sys::$func(f.fptr as *mut _,
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
            sys::ffgkys(f.fptr as *mut _,
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
                    sys::$func(f.fptr as *mut _,
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
            sys::ffpkyj(f.fptr as *mut _,
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
            sys::ffpkys(f.fptr as *mut _,
                        c_name.into_raw(),
                        ffi::CString::new(value).unwrap().into_raw(),
                        ptr::null_mut(),
                        &mut status);
        }

        fits_try!(status, ())
    }
}

/// Reading fits images
pub trait ReadWriteImage: Sized {
    /// Read pixels from an image between a start index and end index
    ///
    /// Start and end are read inclusively, so start = 0, end = 10 will read 11 pixels
    /// in a row.
    fn read_section(fits_file: &FitsFile, start: usize, end: usize) -> Result<Vec<Self>>;

    /// Read a row of pixels from a fits image
    fn read_rows(fits_file: &FitsFile, start_row: usize, num_rows: usize) -> Result<Vec<Self>>;

    /// Read a single row from the image HDU
    fn read_row(fits_file: &FitsFile, row: usize) -> Result<Vec<Self>>;

    /// Read a square region from the chip.
    ///
    /// Lower left indicates the starting point of the square, and the upper
    /// right defines the pixel _beyond_ the end. The range of pixels included
    /// is inclusive of the lower end, and *exclusive* of the upper end.
    fn read_region(fits_file: &FitsFile,
                   lower_left: &Coordinate,
                   upper_right: &Coordinate)
                   -> Result<Vec<Self>>;

    /// Read a whole image into a new `Vec`
    ///
    /// This reads an entire image into a one-dimensional vector
    fn read_image(fits_file: &FitsFile) -> Result<Vec<Self>> {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { dimensions, shape }) => {
                let mut npixels = 1;
                for dim in 0..dimensions {
                    npixels *= shape[dim];
                }
                Self::read_section(fits_file, 0, npixels)
            }
            Ok(HduInfo::TableInfo { .. }) => {
                Err(FitsError {
                    status: 601,
                    message: "cannot read image data from a table hdu".to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    fn write_section(fits_file: &FitsFile, start: usize, end: usize, data: &[Self]) -> Result<()>;

    fn write_region(fits_file: &FitsFile,
                    lower_left: &Coordinate,
                    upper_right: &Coordinate,
                    data: &[Self])
                    -> Result<()>;
}

macro_rules! read_write_image_impl {
    ($t: ty, $data_type: expr) => (
        impl ReadWriteImage for $t {
            fn read_section(fits_file: &FitsFile, start: usize, end: usize) -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { dimensions: _dimensions, shape: _shape }) => {
                        let nelements = end - start;
                        let mut out = vec![0 as $t; nelements];
                        let mut status = 0;

                        unsafe {
                            sys::ffgpv(fits_file.fptr as *mut _,
                                       $data_type.into(),
                                       (start + 1) as i64,
                                       nelements as i64,
                                       ptr::null_mut(),
                                       out.as_mut_ptr() as *mut libc::c_void,
                                       ptr::null_mut(),
                                       &mut status);
                        }

                        fits_try!(status, out)

                    },
                    Ok(HduInfo::TableInfo { .. }) => Err(FitsError {
                        status: 601,
                        message: "cannot read image data from a table hdu".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }

            fn read_rows(fits_file: &FitsFile, start_row: usize, num_rows: usize)
                -> Result<Vec<Self>> {
                match fits_file.fetch_hdu_info() {
                    Ok(HduInfo::ImageInfo { dimensions, shape }) => {
                        if dimensions != 2 {
                            unimplemented!();
                        }

                        let num_cols = shape[1];
                        let start = start_row * num_cols;
                        let end = (start_row + num_rows) * num_cols;

                        Self::read_section(fits_file, start, end)
                    },
                    Ok(HduInfo::TableInfo { .. }) => Err(FitsError {
                        status: 601,
                        message: "cannot read image data from a table hdu".to_string(),
                    }),
                    Err(e) => Err(e),
                }
            }

            fn read_row(fits_file: &FitsFile, row: usize) -> Result<Vec<Self>> {
                Self::read_rows(fits_file, row, 1)
            }

            fn read_region( fits_file: &FitsFile, lower_left: &Coordinate, upper_right: &Coordinate)
                -> Result<Vec<Self>> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { dimensions, .. }) => {
                            if dimensions != 2 {
                                unimplemented!();
                            }

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
                                    fits_file.fptr as *mut _,
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
                        Ok(HduInfo::TableInfo { .. }) => Err(FitsError {
                            status: 601,
                            message: "cannot read image data from a table hdu".to_string(),
                        }),
                        Err(e) => Err(e),
                    }
                }

            fn write_section(
                fits_file: &FitsFile,
                start: usize,
                end: usize,
                data: &[Self])
                -> Result<()> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let nelements = end - start;
                            assert!(data.len() >= nelements);
                            let mut status = 0;
                            unsafe {
                                sys::ffppr(fits_file.fptr as *mut _,
                                           $data_type.into(),
                                           (start + 1) as i64,
                                           nelements as i64,
                                           data.as_ptr() as *mut _,
                                           &mut status);
                            }

                            fits_try!(status, ())
                        },
                        Ok(HduInfo::TableInfo { .. }) => Err(FitsError {
                            status: 601,
                            message: "cannot write image data to a table hdu".to_string(),
                        }),
                        Err(e) => Err(e),
                    }
                }

            fn write_region(
                fits_file: &FitsFile,
                lower_left: &Coordinate,
                upper_right: &Coordinate,
                data: &[Self])
                -> Result<()> {
                    match fits_file.fetch_hdu_info() {
                        Ok(HduInfo::ImageInfo { .. }) => {
                            let mut fpixel = [ (lower_left.x + 1) as _, (lower_left.y + 1) as _ ];
                            let mut lpixel = [ (upper_right.x + 1) as _, (upper_right.y + 1) as _ ];
                            let mut status = 0;

                            unsafe {
                                sys::ffpss(
                                    fits_file.fptr as *mut _,
                                    $data_type.into(),
                                    fpixel.as_mut_ptr(),
                                    lpixel.as_mut_ptr(),
                                    data.as_ptr() as *mut libc::c_void,
                                    &mut status);
                            }

                            fits_try!(status, ())
                        },
                        Ok(HduInfo::TableInfo { .. }) => Err(FitsError {
                            status: 601,
                            message: "cannot write image data to a table hdu".to_string(),
                        }),
                        Err(e) => Err(e),
                    }
                }
        }
    )
}


read_write_image_impl!(i8, DataType::TSHORT);
read_write_image_impl!(i32, DataType::TINT);
read_write_image_impl!(i64, DataType::TLONG);
read_write_image_impl!(u8, DataType::TUSHORT);
read_write_image_impl!(u32, DataType::TUINT);
read_write_image_impl!(u64, DataType::TULONG);
read_write_image_impl!(f32, DataType::TFLOAT);
read_write_image_impl!(f64, DataType::TDOUBLE);

pub enum Column {
    Int32 { name: String, data: Vec<i32> },
    Int64 { name: String, data: Vec<i64> },
    Float { name: String, data: Vec<f32> },
    Double { name: String, data: Vec<f64> },
}

pub struct ColumnIterator<'a> {
    current: usize,
    column_descriptions: Vec<ColumnDescription>,
    fits_file: &'a FitsFile,
}

impl<'a> ColumnIterator<'a> {
    fn new(fits_file: &'a FitsFile) -> Self {
        match fits_file.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { column_descriptions, num_rows: _num_rows }) => {
                ColumnIterator {
                    current: 0,
                    column_descriptions: column_descriptions,
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
        let ncols = self.column_descriptions.len();

        if self.current < ncols {
            let description = &self.column_descriptions[self.current];
            let current_name = description.name.as_str();
            let current_type = typechar_to_data_type(description.data_type.as_str());

            let retval = match current_type {
                DataType::TSHORT => {
                    i32::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Int32 {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                DataType::TLONG => {
                    i64::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Int64 {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                DataType::TFLOAT => {
                    f32::read_col(self.fits_file, current_name)
                        .map(|data| {
                            Some(Column::Float {
                                name: current_name.to_string(),
                                data: data,
                            })
                        })
                        .unwrap()
                }
                DataType::TDOUBLE => {
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

pub struct FitsHdu<'open> {
    fits_file: &'open FitsFile,
    pub info: HduInfo,
}

impl<'open> FitsHdu<'open> {
    pub fn new<T: DescribesHdu>(fits_file: &'open FitsFile, hdu_description: T) -> Result<Self> {
        try!(fits_file.change_hdu(hdu_description));
        match fits_file.fetch_hdu_info() {
            Ok(hdu_info) => {
                Ok(FitsHdu {
                    fits_file: fits_file,
                    info: hdu_info,
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Get the current HDU type
    pub fn hdu_type(&self) -> Result<HduType> {
        let mut status = 0;
        let mut hdu_type = 0;
        unsafe {
            sys::ffghdt(self.fits_file.fptr as *mut _, &mut hdu_type, &mut status);
        }

        fits_try!(status, {
            match hdu_type {
                0 => HduType::IMAGE_HDU,
                2 => HduType::BINARY_TBL,
                _ => unimplemented!(),
            }
        })
    }

    /// Read header key
    pub fn read_key<T: ReadsKey>(&self, name: &str) -> Result<T> {
        T::read_key(self.fits_file, name)
    }

    /// Write header key
    pub fn write_key<T: WritesKey>(&self, name: &str, value: T) -> Result<()> {
        T::write_key(self.fits_file, name, value)
    }

    /// Read an image between pixel a and pixel b into a `Vec`
    pub fn read_section<T: ReadWriteImage>(&self, start: usize, end: usize) -> Result<Vec<T>> {
        T::read_section(self.fits_file, start, end)
    }

    /// Read multiple rows from a fits image
    pub fn read_rows<T: ReadWriteImage>(&self,
                                        start_row: usize,
                                        num_rows: usize)
                                        -> Result<Vec<T>> {
        T::read_rows(self.fits_file, start_row, num_rows)
    }

    /// Read a single row from a fits image
    pub fn read_row<T: ReadWriteImage>(&self, row: usize) -> Result<Vec<T>> {
        T::read_row(self.fits_file, row)
    }

    /// Read a whole fits image into a vector
    pub fn read_image<T: ReadWriteImage>(&self) -> Result<Vec<T>> {
        T::read_image(self.fits_file)
    }

    /// Write contiguous data to a fits image
    pub fn write_section<T: ReadWriteImage>(&self,
                                            start: usize,
                                            end: usize,
                                            data: &[T])
                                            -> Result<()> {
        T::write_section(self.fits_file, start, end, data)
    }

    /// Write a rectangular region to a fits image
    pub fn write_region<T: ReadWriteImage>(&self,
                                           lower_left: &Coordinate,
                                           upper_right: &Coordinate,
                                           data: &[T])
                                           -> Result<()> {
        T::write_region(self.fits_file, lower_left, upper_right, data)
    }

    /// Read a square region into a `Vec`
    pub fn read_region<T: ReadWriteImage>(&self,
                                          lower_left: &Coordinate,
                                          upper_right: &Coordinate)
                                          -> Result<Vec<T>> {
        T::read_region(self.fits_file, lower_left, upper_right)
    }

    pub fn get_column_no<T: Into<String>>(&self, col_name: T) -> Result<usize> {
        let mut status = 0;
        let mut colno = 0;

        let c_col_name = {
            let col_name = col_name.into();
            ffi::CString::new(col_name.as_str()).unwrap()
        };

        unsafe {
            sys::ffgcno(self.fits_file.fptr as *mut _,
                        CaseSensitivity::CASEINSEN as _,
                        c_col_name.as_ptr() as *mut _,
                        &mut colno,
                        &mut status);
        }
        fits_try!(status, (colno - 1) as usize)
    }

    /// Read a binary table column
    pub fn read_col<T: ReadsCol>(&self, name: &str) -> Result<Vec<T>> {
        T::read_col(self.fits_file, name)
    }

    pub fn read_col_range<T: ReadsCol>(&self, name: &str, range: &Range<usize>) -> Result<Vec<T>> {
        T::read_col_range(self.fits_file, name, range)
    }

    pub fn write_col<T: WritesCol, N: Into<String>>(&self, name: N, col_data: &[T]) -> Result<()> {
        T::write_col(self.fits_file, self, name, col_data)
    }

    pub fn write_col_range<T: WritesCol, N: Into<String>>(&self,
                                                          name: N,
                                                          col_data: &[T],
                                                          rows: &Range<usize>)
                                                          -> Result<()> {
        T::write_col_range(self.fits_file, self, name, col_data, rows)
    }

    pub fn columns(&self) -> ColumnIterator {
        ColumnIterator::new(self.fits_file)
    }
}


#[cfg(test)]
mod test {
    extern crate tempdir;

    use super::FitsHdu;
    use super::super::fitsfile::{FitsFile, HduInfo};
    use super::super::types::*;
    use std::{f32, f64};

    /// Helper function for float comparisons
    fn floats_close_f32(a: f32, b: f32) -> bool {
        (a - b).abs() < f32::EPSILON
    }

    fn floats_close_f64(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn test_manually_creating_a_fits_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = FitsHdu::new(&f, "TESTEXT").unwrap();
        match hdu.info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn getting_hdu_type() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let primary_hdu = f.hdu(0).unwrap();
        assert_eq!(primary_hdu.hdu_type().unwrap(), HduType::IMAGE_HDU);

        let ext_hdu = f.hdu("TESTEXT").unwrap();
        assert_eq!(ext_hdu.hdu_type().unwrap(), HduType::BINARY_TBL);
    }

    #[test]
    fn reading_header_keys() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        match hdu.read_key::<i64>("INTTEST") {
            Ok(value) => assert_eq!(value, 42),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<f64>("DBLTEST") {
            Ok(value) => assert!(floats_close_f64(value, 0.09375)),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<String>("TEST") {
            Ok(value) => assert_eq!(value, "value"),
            Err(e) => panic!("Error reading key: {:?}", e),
        }
    }

    // Writing data
    #[test]
    fn writing_header_keywords() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        // Scope ensures file is closed properly
        {
            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            f.hdu(0).unwrap().write_key("FOO", 1i64).unwrap();
            f.hdu(0).unwrap().write_key("BAR", "baz".to_string()).unwrap();
        }

        FitsFile::open(filename.to_str().unwrap())
            .map(|f| {
                assert_eq!(f.hdu(0).unwrap().read_key::<i64>("foo").unwrap(), 1);
                assert_eq!(f.hdu(0).unwrap().read_key::<String>("bar").unwrap(),
                           "baz".to_string());
            })
            .unwrap();
    }


    #[test]
    fn read_columns() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<i32> = hdu.read_col("intcol").unwrap();
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[15], 10);
        assert_eq!(intcol_data[49], 12);

        let floatcol_data: Vec<f32> = hdu.read_col("floatcol").unwrap();
        assert!(floats_close_f32(floatcol_data[0], 17.496801));
        assert!(floats_close_f32(floatcol_data[15], 19.570272));
        assert!(floats_close_f32(floatcol_data[49], 10.217053));

        let doublecol_data: Vec<f64> = hdu.read_col("doublecol").unwrap();
        assert!(floats_close_f64(doublecol_data[0], 16.959972808730814));
        assert!(floats_close_f64(doublecol_data[15], 19.013522579233065));
        assert!(floats_close_f64(doublecol_data[49], 16.61153656123406));
    }

    #[test]
    fn read_column_regions() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let intcol_data: Vec<i32> = hdu.read_col_range("intcol", &(0..2)).unwrap();
        assert_eq!(intcol_data.len(), 3);
        assert_eq!(intcol_data[0], 18);
        assert_eq!(intcol_data[1], 13);
    }

    #[test]
    fn read_column_region_check_ranges() {
        use super::Result;
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let result_data: Result<Vec<i32>> = hdu.read_col_range("intcol", &(0..2_000_000));
        assert!(result_data.is_err());
    }

    #[test]
    fn column_iterator() {
        use super::Column;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let column_names: Vec<String> = hdu.columns()
            .map(|col| match col {
                Column::Int32 { name, data: _data } => name,
                Column::Int64 { name, data: _data } => name,
                Column::Float { name, data: _data } => name,
                Column::Double { name, data: _data } => name,
            })
            .collect();
        assert_eq!(column_names,
                   vec!["intcol".to_string(), "floatcol".to_string(), "doublecol".to_string()]);
    }

    #[test]
    fn column_number() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("testext").unwrap();
        assert_eq!(hdu.get_column_no("intcol").unwrap(), 0);
        assert_eq!(hdu.get_column_no("floatcol").unwrap(), 1);
        assert_eq!(hdu.get_column_no("doublecol").unwrap(), 2);
    }

    #[test]
    fn write_column_data() {
        use super::super::columndescription::ColumnDescription;

        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        let data_to_write: Vec<i32> = vec![10101; 10];
        {
            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            let table_description = vec![ColumnDescription {
                                             name: "bar".to_string(),
                                             data_type: "1J".to_string(),
                                         }];
            f.create_table("foo".to_string(), &table_description).unwrap();
            let hdu = f.hdu("foo").unwrap();

            hdu.write_col("bar", &data_to_write).unwrap();
        }

        let f = FitsFile::open(filename.to_str().unwrap()).unwrap();
        let hdu = f.hdu("foo").unwrap();
        let data: Vec<i32> = hdu.read_col("bar").unwrap();
        assert_eq!(data, data_to_write);
    }

    #[test]
    fn write_column_subset() {
        use super::super::columndescription::ColumnDescription;

        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        let data_to_write: Vec<i32> = vec![10101; 10];
        {
            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            let table_description = vec![ColumnDescription {
                                             name: "bar".to_string(),
                                             data_type: "1J".to_string(),
                                         }];
            f.create_table("foo".to_string(), &table_description).unwrap();
            let hdu = f.hdu("foo").unwrap();

            hdu.write_col_range("bar", &data_to_write, &(0..5)).unwrap();
        }

        let f = FitsFile::open(filename.to_str().unwrap()).unwrap();
        let hdu = f.hdu("foo").unwrap();
        let data: Vec<i32> = hdu.read_col("bar").unwrap();
        assert_eq!(data.len(), 6);
        assert_eq!(data[..], data_to_write[0..6]);
    }

    #[test]
    fn read_image_data() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let first_row: Vec<i32> = hdu.read_section(0, 100).unwrap();
        assert_eq!(first_row.len(), 100);
        assert_eq!(first_row[0], 108);
        assert_eq!(first_row[49], 176);

        let second_row: Vec<i32> = hdu.read_section(100, 200).unwrap();
        assert_eq!(second_row.len(), 100);
        assert_eq!(second_row[0], 177);
        assert_eq!(second_row[49], 168);
    }

    #[test]
    fn read_whole_image() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let image: Vec<i32> = hdu.read_image().unwrap();
        assert_eq!(image.len(), 10000);
    }

    #[test]
    fn read_image_rows() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let row: Vec<i32> = hdu.read_rows(0, 2).unwrap();
        let ref_row: Vec<i32> = hdu.read_section(0, 200).unwrap();
        assert_eq!(row, ref_row);
    }

    #[test]
    fn read_image_row() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let row: Vec<i32> = hdu.read_row(0).unwrap();
        let ref_row: Vec<i32> = hdu.read_section(0, 100).unwrap();
        assert_eq!(row, ref_row);
    }

    #[test]
    fn read_image_slice() {
        use positional::Coordinate;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        let lower_left = Coordinate { x: 0, y: 0 };
        let upper_right = Coordinate { x: 10, y: 10 };
        let chunk: Vec<i32> = hdu.read_region(&lower_left, &upper_right).unwrap();
        assert_eq!(chunk.len(), 11 * 11);
        assert_eq!(chunk[0], 108);
        assert_eq!(chunk[11], 177);
        assert_eq!(chunk[chunk.len() - 1], 160);
    }

    #[test]
    fn read_image_region_from_table() {
        use positional::Coordinate;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("TESTEXT").unwrap();
        let lower_left = Coordinate { x: 0, y: 0 };
        let upper_right = Coordinate { x: 10, y: 10 };
        if let Err(e) = hdu.read_region::<i32>(&lower_left, &upper_right) {
            assert_eq!(e.status, 601);
            assert_eq!(e.message, "cannot read image data from a table hdu");
        } else {
            panic!("Should have been an error");
        }
    }

    #[test]
    fn read_image_section_from_table() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu("TESTEXT").unwrap();
        if let Err(e) = hdu.read_section::<i32>(0, 100) {
            assert_eq!(e.status, 601);
            assert_eq!(e.message, "cannot read image data from a table hdu");
        } else {
            panic!("Should have been an error");
        }
    }

    #[test]
    fn test_write_image_section() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

        // Scope ensures file is closed properly
        {
            use super::super::fitsfile::ImageDescription;

            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            let image_description = ImageDescription {
                data_type: ImageType::LONG_IMG,
                dimensions: vec![100, 20],
            };
            f.create_image("foo".to_string(), &image_description).unwrap();

            let hdu = f.hdu("foo").unwrap();
            hdu.write_section(0, 100, &data_to_write).unwrap();
        }

        let f = FitsFile::open(filename.to_str().unwrap()).unwrap();
        let hdu = f.hdu("foo").unwrap();
        let first_row: Vec<i64> = hdu.read_section(0, 100).unwrap();
        assert_eq!(first_row, data_to_write);

    }

    #[test]
    fn test_write_image_region() {
        use positional::Coordinate;

        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");

        // Scope ensures file is closed properly
        {
            use super::super::fitsfile::ImageDescription;

            let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
            let image_description = ImageDescription {
                data_type: ImageType::LONG_IMG,
                dimensions: vec![100, 20],
            };
            f.create_image("foo".to_string(), &image_description).unwrap();

            let hdu = f.hdu("foo").unwrap();

            let lower_left = Coordinate { x: 0, y: 0 };
            let upper_right = Coordinate { x: 10, y: 10 };
            let data: Vec<i64> = (0..121).map(|v| v + 50).collect();
            hdu.write_region(&lower_left, &upper_right, &data).unwrap();
        }

        let f = FitsFile::open(filename.to_str().unwrap()).unwrap();
        let hdu = f.hdu("foo").unwrap();
        let lower_left = Coordinate { x: 0, y: 0 };
        let upper_right = Coordinate { x: 10, y: 10 };
        let chunk: Vec<i64> = hdu.read_region(&lower_left, &upper_right).unwrap();
        assert_eq!(chunk.len(), 11 * 11);
        assert_eq!(chunk[0], 50);
        assert_eq!(chunk[25], 75);
    }

    #[test]
    fn write_image_section_to_table() {
        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

        use columndescription::ColumnDescription;

        let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
        let table_description = vec![ColumnDescription {
                                         name: "bar".to_string(),
                                         data_type: "1J".to_string(),
                                     }];
        f.create_table("foo".to_string(), &table_description).unwrap();

        let hdu = f.hdu("foo").unwrap();
        if let Err(e) = hdu.write_section(0, 100, &data_to_write) {
            assert_eq!(e.status, 601);
            assert_eq!(e.message, "cannot write image data to a table hdu");
        } else {
            panic!("Should have thrown an error");
        }
    }

    #[test]
    fn write_image_region_to_table() {
        use columndescription::ColumnDescription;
        use positional::Coordinate;

        let tdir = tempdir::TempDir::new("fitsio-").unwrap();
        let tdir_path = tdir.path();
        let filename = tdir_path.join("test.fits");
        let data_to_write: Vec<i64> = (0..100).map(|v| v + 50).collect();

        let f = FitsFile::create(filename.to_str().unwrap()).unwrap();
        let table_description = vec![ColumnDescription {
                                         name: "bar".to_string(),
                                         data_type: "1J".to_string(),
                                     }];
        f.create_table("foo".to_string(), &table_description).unwrap();

        let hdu = f.hdu("foo").unwrap();

        let lower_left = Coordinate { x: 0, y: 0 };
        let upper_right = Coordinate { x: 10, y: 10 };

        if let Err(e) = hdu.write_region(&lower_left, &upper_right, &data_to_write) {
            assert_eq!(e.status, 601);
            assert_eq!(e.message, "cannot write image data to a table hdu");
        } else {
            panic!("Should have thrown an error");
        }
    }
}
