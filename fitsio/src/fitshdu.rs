use super::fitsfile::{FitsFile, HduInfo, DescribesHdu};
use super::sys;
use super::stringutils;
use super::fitserror::{FitsError, Result};
use super::columndescription::ColumnDescription;
use super::conversions::typechar_to_data_type;
use super::libc;
use super::positional::Coordinate;
use super::types::{HduType, DataType};
use std::ffi;
use std::ptr;

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
                        column_descriptions, num_rows, ..
                    }) => {
                        let mut out = vec![$nullval; num_rows];
                        assert_eq!(out.len(), num_rows);
                        let column_number = column_descriptions.iter().position(|ref desc| {
                            desc.name.as_str() == name
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


reads_image_impl!(i8, DataType::TSHORT);
reads_image_impl!(i32, DataType::TINT);
reads_image_impl!(i64, DataType::TLONG);
reads_image_impl!(u8, DataType::TUSHORT);
reads_image_impl!(u32, DataType::TUINT);
reads_image_impl!(u64, DataType::TULONG);
reads_image_impl!(f32, DataType::TFLOAT);
reads_image_impl!(f64, DataType::TDOUBLE);

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
            let current_name = &description.name;
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
            sys::ffghdt(self.fits_file.fptr, &mut hdu_type, &mut status);
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
    pub fn read_section<T: ReadsImage>(&self, start: usize, end: usize) -> Result<Vec<T>> {
        T::read_section(self.fits_file, start, end)
    }

    /// Read a square region into a `Vec`
    pub fn read_region<T: ReadsImage>(&self,
                                      lower_left: &Coordinate,
                                      upper_right: &Coordinate)
                                      -> Result<Vec<T>> {
        T::read_region(self.fits_file, lower_left, upper_right)
    }


    /// Read a binary table column
    pub fn read_col<T: ReadsCol>(&self, name: &str) -> Result<Vec<T>> {
        T::read_col(self.fits_file, name)
    }

    pub fn columns(&self) -> ColumnIterator {
        ColumnIterator::new(self.fits_file)
    }
}


#[cfg(test)]
mod test {
    use super::FitsHdu;
    use super::super::fitsfile::{FitsFile, HduInfo};
    use super::super::types::*;

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
            Ok(value) => assert_eq!(value, 0.09375),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<String>("TEST") {
            Ok(value) => assert_eq!(value, "value"),
            Err(e) => panic!("Error reading key: {:?}", e),
        }
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
        assert_eq!(floatcol_data[0], 17.496801);
        assert_eq!(floatcol_data[15], 19.570272);
        assert_eq!(floatcol_data[49], 10.217053);

        let doublecol_data: Vec<f64> = hdu.read_col("doublecol").unwrap();
        assert_eq!(doublecol_data[0], 16.959972808730814);
        assert_eq!(doublecol_data[15], 19.013522579233065);
        assert_eq!(doublecol_data[49], 16.61153656123406);
    }

    #[test]
    fn column_iterator() {
        use super::Column;

        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(1).unwrap();
        let column_names: Vec<String> = hdu.columns()
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
}
