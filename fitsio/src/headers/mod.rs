//! Header-related code
use crate::errors::{check_status, Result};
use crate::fitsfile::FitsFile;
use crate::longnam::*;
use crate::types::HasFitsDataType;
use std::ffi;
use std::ptr;

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
    ($t:ty) => {
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
                    fits_read_key(
                        f.fptr.as_mut() as *mut _,
                        <$t as HasFitsDataType>::FITS_DATA_TYPE.into(),
                        c_name.as_ptr(),
                        &mut value.value as *mut $t as *mut c_void,
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

reads_key_impl!(i32);
reads_key_impl!(i64);
reads_key_impl!(f32);
reads_key_impl!(f64);

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

macro_rules! writes_key_impl {
    ($t:ty) => {
        impl WritesKey for $t {
            fn write_key(f: &mut FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;

                unsafe {
                    fits_write_key(
                        f.fptr.as_mut() as *mut _,
                        <$t as HasFitsDataType>::FITS_DATA_TYPE.into(),
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

                unsafe {
                    fits_write_key(
                        f.fptr.as_mut() as *mut _,
                        <$t as HasFitsDataType>::FITS_DATA_TYPE.into(),
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

writes_key_impl!(i8);
writes_key_impl!(i16);
writes_key_impl!(i32);
writes_key_impl!(i64);
writes_key_impl!(u8);
writes_key_impl!(u16);
writes_key_impl!(u32);
writes_key_impl!(u64);
writes_key_impl!(f32);
writes_key_impl!(f64);

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
            hdu.write_key(&mut f, "ONE", -1i8).unwrap();
            hdu.write_key(&mut f, "TWO", -500i16).unwrap();
            hdu.write_key(&mut f, "THREE", -1_000_000i32).unwrap();
            hdu.write_key(&mut f, "FOUR", -99_000_000_000i64).unwrap();
            hdu.write_key(&mut f, "UONE", 1u8).unwrap();
            hdu.write_key(&mut f, "UTWO", 500u16).unwrap();
            hdu.write_key(&mut f, "UTHREE", 1_000_000u32).unwrap();
            hdu.write_key(&mut f, "UFOUR", 3_000_000_000u32).unwrap();
            hdu.write_key(&mut f, "UFIVE", 99_000_000_000u64).unwrap();

            // make sure we round-trip:

            assert_matches1!(hdu.read_key(&mut f, "ONE"), Ok(-1i32));
            assert_matches1!(hdu.read_key(&mut f, "ONE"), Ok(-1i64));

            assert_matches1!(hdu.read_key(&mut f, "TWO"), Ok(-500i32));
            assert_matches1!(hdu.read_key(&mut f, "TWO"), Ok(-500i64));

            assert_matches1!(hdu.read_key(&mut f, "THREE"), Ok(-1_000_000i32));
            assert_matches1!(hdu.read_key(&mut f, "THREE"), Ok(-1_000_000i64));

            assert_matches1!(hdu.read_key::<i32>(&mut f, "FOUR"), Err(_));
            assert_matches1!(hdu.read_key(&mut f, "FOUR"), Ok(-99_000_000_000i64));

            assert_matches1!(hdu.read_key(&mut f, "UONE"), Ok(1i32));
            assert_matches1!(hdu.read_key(&mut f, "UONE"), Ok(1i64));

            assert_matches1!(hdu.read_key(&mut f, "UTWO"), Ok(500i32));
            assert_matches1!(hdu.read_key(&mut f, "UTWO"), Ok(500i64));

            assert_matches1!(hdu.read_key(&mut f, "UTHREE"), Ok(1_000_000i32));
            assert_matches1!(hdu.read_key(&mut f, "UTHREE"), Ok(1_000_000i64));

            assert_matches1!(hdu.read_key::<i32>(&mut f, "UFOUR"), Err(_));
            assert_matches1!(hdu.read_key(&mut f, "UFOUR"), Ok(3_000_000_000i64));

            assert_matches1!(hdu.read_key::<i32>(&mut f, "UFIVE"), Err(_));
            assert_matches1!(hdu.read_key(&mut f, "UFIVE"), Ok(99_000_000_000i64));
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
