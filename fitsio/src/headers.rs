//! Header-related code
use crate::errors::{check_status, Result};
use crate::fitsfile::FitsFile;
use crate::longnam::*;
use crate::types::DataType;
use std::ffi;
use std::ptr;

const MAX_VALUE_LENGTH: usize = 71;

/**
Trait applied to types which can be read from a FITS header

This is currently:

* i32
* i64
* f32
* f64
* String
* */
pub trait ReadsKey {
    #[doc(hidden)]
    fn read_key(f: &mut FitsFile, name: &str) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! reads_key_impl {
    ($t:ty, $func:ident) => {
        impl ReadsKey for $t {
            fn read_key(f: &mut FitsFile, name: &str) -> Result<Self> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;
                let mut value: Self = Self::default();

                unsafe {
                    $func(
                        f.fptr.as_mut() as *mut _,
                        c_name.as_ptr(),
                        &mut value,
                        ptr::null_mut(),
                        &mut status,
                    );
                }

                check_status(status).map(|_| value)
            }
        }
    };
}

reads_key_impl!(i32, fits_read_key_log);
#[cfg(all(target_pointer_width = "64", not(target_os = "windows")))]
reads_key_impl!(i64, fits_read_key_lng);
#[cfg(any(target_pointer_width = "32", target_os = "windows"))]
reads_key_impl!(i64, fits_read_key_lnglng);
reads_key_impl!(f32, fits_read_key_flt);
reads_key_impl!(f64, fits_read_key_dbl);

impl ReadsKey for bool {
    fn read_key(f: &mut FitsFile, name: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let int_value = i32::read_key(f, name)?;
        Ok(int_value > 0)
    }
}

impl ReadsKey for String {
    fn read_key(f: &mut FitsFile, name: &str) -> Result<Self> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;
        let mut value: Vec<c_char> = vec![0; MAX_VALUE_LENGTH];

        unsafe {
            fits_read_key_str(
                f.fptr.as_mut() as *mut _,
                c_name.as_ptr(),
                value.as_mut_ptr(),
                ptr::null_mut(),
                &mut status,
            );
        }

        check_status(status).and_then(|_| {
            let value: Vec<u8> = value.iter().map(|&x| x as u8).filter(|&x| x != 0).collect();
            Ok(String::from_utf8(value)?)
        })
    }
}

/// Writing a fits keyword
pub trait WritesKey {
    #[doc(hidden)]
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()>;
    #[doc(hidden)]
    fn write_key_cmt(f: &mut FitsFile, name: &str, value: Self, comment: &str) -> Result<()>;
}

macro_rules! writes_key_impl_int {
    ($t:ty, $datatype:expr) => {
        impl WritesKey for $t {
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;

                let datatype = u8::from($datatype);

                unsafe {
                    fits_write_key(
                        f.fptr.as_mut() as *mut _,
                        datatype as _,
                        c_name.as_ptr(),
                        &value as *const $t as *mut c_void,
                        ptr::null_mut(),
                        &mut status,
                    );
                }
                check_status(status)
            }

            fn write_key_cmt(
                f: &mut FitsFile,
                name: &str,
                value: Self,
                comment: &str,
            ) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let c_cmt = ffi::CString::new(comment)?;
                let mut status = 0;

                let datatype = u8::from($datatype);

                unsafe {
                    fits_write_key(
                        f.fptr.as_mut() as *mut _,
                        datatype as _,
                        c_name.as_ptr(),
                        &value as *const $t as *mut c_void,
                        c_cmt.as_ptr(),
                        &mut status,
                    );
                }
                check_status(status)
            }
        }
    };
}

writes_key_impl_int!(i8, DataType::TSBYTE);
writes_key_impl_int!(i16, DataType::TINT);
writes_key_impl_int!(i32, DataType::TINT);
writes_key_impl_int!(i64, DataType::TLONG);
writes_key_impl_int!(u8, DataType::TBYTE);
writes_key_impl_int!(u16, DataType::TUINT);
writes_key_impl_int!(u32, DataType::TUINT);
writes_key_impl_int!(u64, DataType::TULONG);

macro_rules! writes_key_impl_flt {
    ($t:ty, $func:ident) => {
        impl WritesKey for $t {
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;

                unsafe {
                    $func(
                        f.fptr.as_mut() as *mut _,
                        c_name.as_ptr(),
                        value,
                        9,
                        ptr::null_mut(),
                        &mut status,
                    );
                }
                check_status(status)
            }

            fn write_key_cmt(
                f: &mut FitsFile,
                name: &str,
                value: Self,
                comment: &str,
            ) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let c_cmt = ffi::CString::new(comment)?;
                let mut status = 0;

                unsafe {
                    $func(
                        f.fptr.as_mut() as *mut _,
                        c_name.as_ptr(),
                        value,
                        9,
                        c_cmt.as_ptr(),
                        &mut status,
                    );
                }
                check_status(status)
            }
        }
    };
}

writes_key_impl_flt!(f32, fits_write_key_flt);
writes_key_impl_flt!(f64, fits_write_key_dbl);

impl WritesKey for String {
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
        WritesKey::write_key(f, name, value.as_str())
    }
    fn write_key_cmt(f: &mut FitsFile, name: &str, value: Self, comment: &str) -> Result<()> {
        WritesKey::write_key_cmt(f, name, value.as_str(), comment)
    }
}

impl<'a> WritesKey for &'a str {
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let c_value = ffi::CString::new(value)?;
        let mut status = 0;

        unsafe {
            fits_write_key_str(
                f.fptr.as_mut() as *mut _,
                c_name.as_ptr(),
                c_value.as_ptr(),
                ptr::null_mut(),
                &mut status,
            );
        }

        check_status(status)
    }

    fn write_key_cmt(f: &mut FitsFile, name: &str, value: Self, comment: &str) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let c_value = ffi::CString::new(value)?;
        let c_cmt = ffi::CString::new(comment)?;
        let mut status = 0;

        unsafe {
            fits_write_key_str(
                f.fptr.as_mut() as *mut _,
                c_name.as_ptr(),
                c_value.as_ptr(),
                c_cmt.as_ptr(),
                &mut status,
            );
        }

        check_status(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testhelpers::{duplicate_test_file, floats_close_f64, with_temp_file};

    #[test]
    fn test_reading_header_keys() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.hdu(0).unwrap();
        match hdu.read_key::<i64>(&mut f, "INTTEST") {
            Ok(value) => assert_eq!(value, 42),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<f64>(&mut f, "DBLTEST") {
            Ok(value) => assert!(
                floats_close_f64(value, 0.09375),
                "{:?} != {:?}",
                value,
                0.09375
            ),
            Err(e) => panic!("Error reading key: {:?}", e),
        }

        match hdu.read_key::<String>(&mut f, "TEST") {
            Ok(value) => assert_eq!(value, "value"),
            Err(e) => panic!("Error reading key: {:?}", e),
        }
    }

    #[test]
    fn test_writing_header_keywords() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                f.hdu(0).unwrap().write_key(&mut f, "FOO", 1i64).unwrap();
                f.hdu(0)
                    .unwrap()
                    .write_key(&mut f, "BAR", "baz".to_string())
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|mut f| {
                    assert_eq!(f.hdu(0).unwrap().read_key::<i64>(&mut f, "foo").unwrap(), 1);
                    assert_eq!(
                        f.hdu(0).unwrap().read_key::<String>(&mut f, "bar").unwrap(),
                        "baz".to_string()
                    );
                })
                .unwrap();
        });
    }

    #[test]
    fn test_writing_integers() {
        duplicate_test_file(|filename| {
            let mut f = FitsFile::edit(filename).unwrap();
            let hdu = f.hdu(0).unwrap();
            hdu.write_key(&mut f, "ONE", 1i8).unwrap();
            hdu.write_key(&mut f, "TWO", 1i16).unwrap();
            hdu.write_key(&mut f, "THREE", 1i32).unwrap();
            hdu.write_key(&mut f, "FOUR", 1i64).unwrap();
            hdu.write_key(&mut f, "UONE", 1u8).unwrap();
            hdu.write_key(&mut f, "UTWO", 1u16).unwrap();
            hdu.write_key(&mut f, "UTHREE", 1u32).unwrap();
            hdu.write_key(&mut f, "UFOUR", 1u64).unwrap();
        });
    }

    #[test]
    fn boolean_header_values() {
        let mut f = FitsFile::open("../testdata/full_example.fits").unwrap();
        let hdu = f.primary_hdu().unwrap();

        let res = dbg!(hdu.read_key::<bool>(&mut f, "SIMPLE").unwrap());

        assert!(res);
    }
}
