use std::ffi;
use std::ptr;
use libc;
use fitsfile::FitsFile;
use longnam::*;
use fitserror::check_status;
use errors::Result;

const MAX_VALUE_LENGTH: usize = 71;

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
    #[doc(hidden)]
    fn read_key(f: &FitsFile, name: &str) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! reads_key_impl {
    ($t:ty, $func:ident) => (
        impl ReadsKey for $t {
            fn read_key(f: &FitsFile, name: &str) -> Result<Self> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;
                let mut value: Self = Self::default();

                unsafe {
                    $func(f.fptr as *mut _,
                           c_name.into_raw(),
                           &mut value,
                           ptr::null_mut(),
                           &mut status);
                }

                check_status(status).map(|_| value)
            }
        }
    )
}

reads_key_impl!(i32, fits_read_key_log);
#[cfg(target_pointer_width = "64")]
reads_key_impl!(i64, fits_read_key_lng);
#[cfg(target_pointer_width = "32")]
reads_key_impl!(i64, fits_read_key_lnglng);
reads_key_impl!(f32, fits_read_key_flt);
reads_key_impl!(f64, fits_read_key_dbl);

impl ReadsKey for String {
    fn read_key(f: &FitsFile, name: &str) -> Result<Self> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;
        let mut value: Vec<libc::c_char> = vec![0; MAX_VALUE_LENGTH];

        unsafe {
            fits_read_key_str(
                f.fptr as *mut _,
                c_name.into_raw(),
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
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()>;
}

macro_rules! writes_key_impl_flt {
    ($t:ty, $func:ident) => (
        impl WritesKey for $t {
            fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
                let c_name = ffi::CString::new(name)?;
                let mut status = 0;

                unsafe {
                    $func(f.fptr as *mut _,
                                c_name.into_raw(),
                                value,
                                9,
                                ptr::null_mut(),
                                &mut status);
                }
                check_status(status)
            }
        }
    )
}

impl WritesKey for i64 {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;

        unsafe {
            fits_write_key_lng(
                f.fptr as *mut _,
                c_name.into_raw(),
                value,
                ptr::null_mut(),
                &mut status,
            );
        }
        check_status(status)
    }
}

writes_key_impl_flt!(f32, fits_write_key_flt);
writes_key_impl_flt!(f64, fits_write_key_dbl);

impl WritesKey for String {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        WritesKey::write_key(f, name, value.as_str())
    }
}

impl<'a> WritesKey for &'a str {
    fn write_key(f: &FitsFile, name: &str, value: Self) -> Result<()> {
        let c_name = ffi::CString::new(name)?;
        let mut status = 0;

        unsafe {
            fits_write_key_str(
                f.fptr as *mut _,
                c_name.into_raw(),
                ffi::CString::new(value)?.into_raw(),
                ptr::null_mut(),
                &mut status,
            );
        }

        check_status(status)
    }
}
