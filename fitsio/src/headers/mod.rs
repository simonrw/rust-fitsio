//! Header-related code
use crate::errors::{check_status, Result};
use crate::fitsfile::FitsFile;
use crate::longnam::*;
use crate::types::DataType;
use std::ffi;
use std::ptr;

pub mod card;
mod constants;
mod header_value;

use constants::{MAX_COMMENT_LENGTH, MAX_VALUE_LENGTH};
pub use header_value::HeaderValue;

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
                let hv: HeaderValue<$t> = ReadsKey::read_key(f, name)?;
                Ok(hv.value)
            }
        }
        impl ReadsKey for HeaderValue<$t>
        where
            $t: Default,
        {
            fn read_key(f: &mut FitsFile, name: &str) -> Result<Self> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;
                let mut value: Self = Default::default();
                let mut comment: Vec<c_char> = vec![0; MAX_COMMENT_LENGTH];

                unsafe {
                    $func(
                        f.fptr.as_mut() as *mut _,
                        c_name.as_ptr(),
                        &mut value.value,
                        comment.as_mut_ptr(),
                        &mut status,
                    );
                }

                check_status(status).map(|_| {
                    let comment = {
                        let comment: Vec<u8> = comment
                            .iter()
                            .map(|&x| x as u8)
                            .filter(|&x| x != 0)
                            .collect();
                        if comment.is_empty() {
                            None
                        } else {
                            String::from_utf8(comment).ok()
                        }
                    };

                    value.comment = comment;

                    value
                })
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
        i32::read_key(f, name).map(|v| v > 0)
    }
}

impl ReadsKey for HeaderValue<bool> {
    fn read_key(f: &mut FitsFile, name: &str) -> Result<Self>
    where
        Self: Sized,
    {
        let hv: HeaderValue<i32> = ReadsKey::read_key(f, name)?;
        Ok(hv.map(|v| v > 0))
    }
}

impl ReadsKey for String {
    fn read_key(f: &mut FitsFile, name: &str) -> Result<Self> {
        let hv: HeaderValue<String> = ReadsKey::read_key(f, name)?;
        Ok(hv.value)
    }
}

impl ReadsKey for HeaderValue<String> {
    fn read_key(f: &mut FitsFile, name: &str) -> Result<Self> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;
        let mut value: Vec<c_char> = vec![0; MAX_VALUE_LENGTH];
        let mut comment: Vec<c_char> = vec![0; MAX_COMMENT_LENGTH];

        unsafe {
            fits_read_key_str(
                f.fptr.as_mut() as *mut _,
                c_name.as_ptr(),
                value.as_mut_ptr(),
                comment.as_mut_ptr(),
                &mut status,
            );
        }

        check_status(status).and_then(|_| {
            let value: Vec<u8> = value.iter().map(|&x| x as u8).filter(|&x| x != 0).collect();
            String::from_utf8(value)
                .map(|value| {
                    let comment = {
                        let comment: Vec<u8> = comment
                            .iter()
                            .map(|&x| x as u8)
                            .filter(|&x| x != 0)
                            .collect();
                        if comment.is_empty() {
                            None
                        } else {
                            String::from_utf8(comment).ok()
                        }
                    };
                    HeaderValue { value, comment }
                })
                .map_err(From::from)
        })
    }
}

/// Writing a fits keyword
pub trait WritesKey {
    #[doc(hidden)]
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()>;
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
        }

        impl WritesKey for ($t, &str) {
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let (value, comment) = value;
                let c_name = ffi::CString::new(name)?;
                let c_comment = ffi::CString::new(comment)?;
                let mut status = 0;

                let datatype = u8::from($datatype);

                unsafe {
                    fits_write_key(
                        f.fptr.as_mut() as *mut _,
                        datatype as _,
                        c_name.as_ptr(),
                        &value as *const $t as *mut c_void,
                        c_comment.as_ptr(),
                        &mut status,
                    );
                }
                check_status(status)
            }
        }

        impl WritesKey for ($t, String) {
            #[inline(always)]
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let (value, comment) = value;
                WritesKey::write_key(f, name, (value, comment.as_str()))
            }
        }
    };
}

writes_key_impl_int!(i8, DataType::TSBYTE);
writes_key_impl_int!(i16, DataType::TSHORT);
writes_key_impl_int!(i32, DataType::TINT);
writes_key_impl_int!(i64, DataType::TLONG);
writes_key_impl_int!(u8, DataType::TBYTE);
writes_key_impl_int!(u16, DataType::TUSHORT);
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
        }

        impl WritesKey for ($t, &str) {
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let (value, comment) = value;
                let c_name = ffi::CString::new(name)?;
                let c_comment = ffi::CString::new(comment)?;
                let mut status = 0;

                unsafe {
                    $func(
                        f.fptr.as_mut() as *mut _,
                        c_name.as_ptr(),
                        value,
                        9,
                        c_comment.as_ptr(),
                        &mut status,
                    );
                }
                check_status(status)
            }
        }

        impl WritesKey for ($t, String) {
            #[inline(always)]
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let (value, comment) = value;
                WritesKey::write_key(f, name, (value, comment.as_str()))
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
}

impl WritesKey for &'_ str {
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
}

impl WritesKey for (String, &str) {
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
        let (value, comment) = value;
        WritesKey::write_key(f, name, (value.as_str(), comment))
    }
}

impl WritesKey for (String, String) {
    #[inline(always)]
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
        let (value, comment) = value;
        WritesKey::write_key(f, name, (value.as_str(), comment.as_str()))
    }
}

impl<'a> WritesKey for (&'a str, &'a str) {
    fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
        let (value, comment) = value;
        let c_name = ffi::CString::new(name)?;
        let c_value = ffi::CString::new(value)?;
        let c_comment = ffi::CString::new(comment)?;
        let mut status = 0;

        unsafe {
            fits_write_key_str(
                f.fptr.as_mut() as *mut _,
                c_name.as_ptr(),
                c_value.as_ptr(),
                c_comment.as_ptr(),
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
    fn test_writing_with_comments() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                f.hdu(0)
                    .unwrap()
                    .write_key(&mut f, "FOO", (1i64, "Foo value"))
                    .unwrap();
                f.hdu(0)
                    .unwrap()
                    .write_key(&mut f, "BAR", ("baz".to_string(), "baz value"))
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|mut f| {
                    let foo_header_value = f
                        .hdu(0)
                        .unwrap()
                        .read_key::<HeaderValue<i64>>(&mut f, "foo")
                        .unwrap();
                    assert_eq!(foo_header_value.value, 1);
                    assert_eq!(foo_header_value.comment, Some("Foo value".to_string()));
                })
                .unwrap();
        });
    }

    #[test]
    fn test_writing_reading_empty_comment() {
        with_temp_file(|filename| {
            // Scope ensures file is closed properly
            {
                let mut f = FitsFile::create(filename).open().unwrap();
                f.hdu(0)
                    .unwrap()
                    .write_key(&mut f, "FOO", (1i64, ""))
                    .unwrap();
            }

            FitsFile::open(filename)
                .map(|mut f| {
                    let foo_header_value = f
                        .hdu(0)
                        .unwrap()
                        .read_key::<HeaderValue<i64>>(&mut f, "foo")
                        .unwrap();
                    assert_eq!(foo_header_value.value, 1);
                    assert!(foo_header_value.comment.is_none());
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

        let res = hdu.read_key::<bool>(&mut f, "SIMPLE").unwrap();
        assert!(res);
    }
}

#[cfg(test)]
mod headervalue_tests {
    use super::HeaderValue;

    #[test]
    fn equate_different_types() {
        let v = HeaderValue {
            value: 1i64,
            comment: Some("".to_string()),
        };

        assert_eq!(v, 1i64);
    }
}
