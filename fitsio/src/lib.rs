#![allow(dead_code, unused_imports)]

extern crate fitsio_sys as sys;
extern crate libc;

use std::ptr;
use std::ffi;

mod stringutils;

/// Error type
#[derive(Debug, PartialEq, Eq)]
pub struct FitsError {
    status: i32,
    message: String,
}

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
        match status {
            0 => Ok(()),
            _ => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
    }
}

impl<'a> DescribesHdu for &'a str {
    fn change_hdu(&self, f: &FitsFile) -> Result<()> {
        let mut _hdu_type = 0;
        let mut status = 0;
        let c_hdu_name = ffi::CString::new(*self).unwrap();

        unsafe {
            sys::ffmnhd(f.fptr,
                        sys::HduType::ANY_HDU as libc::c_int,
                        c_hdu_name.into_raw(),
                        0,
                        &mut status);
        }

        match status {
            0 => Ok(()),
            _ => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
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

                match status {
                    0 => Ok(value),
                    s => {
                        Err(FitsError {
                            status: s,
                            message: stringutils::status_to_string(s).unwrap(),
                        })
                    }
                }
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

        match status {
            0 => {
                let value: Vec<u8> = value.iter()
                    .map(|&x| x as u8)
                    .filter(|&x| x != 0)
                    .collect();
                Ok(String::from_utf8(value).unwrap())
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }

    }
}

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
                match status {
                    0 => Ok(()),
                    _ => Err(FitsError {
                        status: status,
                        message: stringutils::status_to_string(status).unwrap(),
                    }),
                }
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
        match status {
            0 => Ok(()),
            _ => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
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

        match status {
            0 => Ok(()),
            _ => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
    }
}

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

pub struct FitsFile {
    fptr: *mut sys::fitsfile,
    pub filename: String,
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

    match status {
        0 => Ok(hdu_type),
        _ => {
            Err(FitsError {
                status: status,
                message: stringutils::status_to_string(status).unwrap(),
            })
        }
    }
}

impl FitsFile {
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

        match status {
            0 => {
                Ok(FitsFile {
                    fptr: fptr,
                    filename: filename.to_string(),
                })
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }

    }

    pub fn create(path: &str) -> Result<Self> {
        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(path).unwrap();

        unsafe {
            sys::ffinit(&mut fptr as *mut *mut sys::fitsfile,
                        c_filename.as_ptr(),
                        &mut status);
        }

        match status {
            0 => {
                let f = FitsFile {
                    fptr: fptr,
                    filename: path.to_string(),
                };
                f.add_empty_primary().unwrap();
                Ok(f)
            }
            status => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
    }

    fn add_empty_primary(&self) -> Result<()> {
        let mut status = 0;
        unsafe {
            sys::ffphps(self.fptr, 8, 0, ptr::null_mut(), &mut status);
        }
        match status {
            0 => Ok(()),
            _ => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
    }

    pub fn change_hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self)
    }

    pub fn hdu_type(&self) -> Result<sys::HduType> {
        let mut status = 0;
        let mut hdu_type = 0;
        unsafe {
            sys::ffghdt(self.fptr, &mut hdu_type, &mut status);
        }
        match status {
            0 => {
                match hdu_type {
                    0 => Ok(sys::HduType::IMAGE_HDU),
                    2 => Ok(sys::HduType::BINARY_TBL),
                    _ => unimplemented!(),
                }
            }
            _ => {
                Err(FitsError {
                    status: status,
                    message: stringutils::status_to_string(status).unwrap(),
                })
            }
        }
    }

    fn hdu_number(&self) -> usize {
        let mut hdu_num = 0;
        unsafe {
            sys::ffghdn(self.fptr, &mut hdu_num);
        }
        (hdu_num - 1) as usize
    }

    pub fn read_key<T: ReadsKey>(&self, name: &str) -> Result<T> {
        T::read_key(self, name)
    }

    pub fn write_key<T: WritesKey>(&self, name: &str, value: T) -> Result<()> {
        T::write_key(self, name, value)
    }

    fn fetch_hdu_info(&self) -> Result<HduInfo> {
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

    #[test]
    fn fetching_hdu_info() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::ImageInfo { dimensions, shape}) => {
                assert_eq!(dimensions, 2);
                assert_eq!(shape, vec![100, 100]);
            },
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }

        f.change_hdu(1).unwrap();
        match f.fetch_hdu_info() {
            Ok(HduInfo::TableInfo { column_names, column_types, num_rows }) => {
                assert_eq!(num_rows, 50);
                assert_eq!(column_names, vec![
                           "intcol".to_string(),
                           "floatcol".to_string(),
                           "doublecol".to_string(),
                ]);
                assert_eq!(column_types, vec![
                        sys::DataType::TLONG,
                        sys::DataType::TFLOAT,
                        sys::DataType::TDOUBLE,
                ]);
            },
            Err(e) => panic!("Error fetching hdu info {:?}", e),
            _ => panic!("Unknown error"),
        }
    }

}
