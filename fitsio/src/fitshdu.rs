use super::fitsfile::{FitsFile, HduInfo, DescribesHdu};
use super::sys;
use super::stringutils;
use super::fitserror::{FitsError, Result};
use super::libc;
use std::ffi;
use std::ptr;


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


pub struct FitsHdu<'open> {
    fits_file: &'open FitsFile,
    pub hdu_info: HduInfo,
}

impl<'open> FitsHdu<'open> {
    pub fn new<T: DescribesHdu>(fits_file: &'open FitsFile, hdu_description: T) -> Result<Self> {
        try!(fits_file.change_hdu(hdu_description));
        match fits_file.fetch_hdu_info() {
            Ok(hdu_info) => {
                Ok(FitsHdu {
                    fits_file: fits_file,
                    hdu_info: hdu_info,
                })
            }
            Err(e) => Err(e),
        }
    }

    fn change_hdu<T: DescribesHdu>(&self, hdu_description: T) -> Result<()> {
        hdu_description.change_hdu(self.fits_file)
    }

    /// Get the current HDU type
    pub fn hdu_type(&self) -> Result<sys::HduType> {
        let mut status = 0;
        let mut hdu_type = 0;
        unsafe {
            sys::ffghdt(self.fits_file.fptr, &mut hdu_type, &mut status);
        }

        fits_try!(status, {
            match hdu_type {
                0 => sys::HduType::IMAGE_HDU,
                2 => sys::HduType::BINARY_TBL,
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
}


#[cfg(test)]
mod test {
    use super::FitsHdu;
    use super::super::fitsfile::{FitsFile, HduInfo};

    #[test]
    fn test_manually_creating_a_fits_hdu() {
        let f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = FitsHdu::new(&f, "TESTEXT").unwrap();
        match hdu.hdu_info {
            HduInfo::TableInfo { num_rows, .. } => {
                assert_eq!(num_rows, 50);
            }
            _ => panic!("Incorrect HDU type found"),
        }
    }

    #[test]
    fn getting_hdu_type() {
        use ::sys::HduType;

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
}
