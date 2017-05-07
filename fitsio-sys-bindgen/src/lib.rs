//! This package was automatically generated with [`rust-bindgen`][1] and as such was not
//! user-generated.
//!
//! The functions contained are expected to be used with [`fitsio`][2], a high level API wrapper
//! around the low level direct C-bindings, though the bindings are complete enough to be usable.
//!
//! This code will not be directly documented, and so users should refer to the [`fitsio` C
//! documentation][3] for usage.
//!
//! ## Note about function names
//!
//! Unfortunately we must use fits short names throughout. The C-api exposes long names for
//! functions which are more descriptive, for example `fits_open_file` instead of `ffopen`, but the
//! symbols available in the library have only short names, and the long names are merely
//! preprocessor definitions.
//!
//! ## Examples
//!
//! ```rust
//! # extern crate fitsio_sys_bindgen as fitsio_sys;
//! use std::ptr;
//! use std::ffi;
//! # use fitsio_sys::{ffinit, ffphps, ffclos};
//!
//! # fn main() {
//! let filename = ffi::CString::new("!/tmp/test.fits").unwrap();
//! let mut fptr = ptr::null_mut();
//! let mut status = 0;
//!
//! unsafe {
//!     // Create a new file, clobbering any pre-existing file
//!     ffinit(&mut fptr as *mut *mut _,
//!         filename.as_ptr(),
//!         &mut status);
//!
//!     // Add an empty primary HDU
//!     ffphps(fptr, 8, 0, ptr::null_mut(), &mut status);
//!
//!     // Finally close the file
//!     ffclos(fptr, &mut status);
//! }
//!
//! assert_eq!(status, 0);
//! # }
//! ```
//!
//! [1]: https://github.com/servo/rust-bindgen
//! [2]: https://crates.io/crates/fitsio
//! [3]: http://heasarc.gsfc.nasa.gov/docs/software/fitsio/c/c_user/cfitsio.html

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

extern crate libc;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    extern crate tempdir;

    use std::ffi;
    use std::ptr;
    use libc::c_char;
    use super::*;

    #[test]
    fn raw_opening_an_existing_file() {
        let mut fptr = ptr::null_mut();
        let mut status = -1;
        let c_filename = ffi::CString::new("../testdata/full_example.fits").unwrap();

        unsafe {
            ffopen(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   0,
                   &mut status);
            ffclos(fptr, &mut status);
        }

        assert_eq!(status, 0);
    }

    #[test]
    fn raw_creating_a_new_file() {
        // Set up the test filename
        let tdir = tempdir::TempDir::new("rust-fitsio-").unwrap();
        let filename = tdir.path().join("test.fits");
        assert!(!filename.exists());

        let mut fptr = ptr::null_mut();
        let mut status = 0;
        let c_filename = ffi::CString::new(filename.to_str().unwrap()).unwrap();

        unsafe {
            ffinit(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   &mut status);
        }

        assert!(filename.exists());
    }

    #[test]
    fn getting_current_hdu_number() {
        let mut fptr = ptr::null_mut();
        let mut status = -1;
        let c_filename = ffi::CString::new("../testdata/full_example.fits").unwrap();
        let mut hdu_num = -1;

        unsafe {
            ffopen(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   0,
                   &mut status);
            ffghdn(fptr, &mut hdu_num);
            ffclos(fptr, &mut status);
        }

        assert_eq!(hdu_num, 1);
    }

    #[test]
    fn changing_hdu_by_absolute_number() {
        let mut fptr = ptr::null_mut();
        let mut status = -1;
        let c_filename = ffi::CString::new("../testdata/full_example.fits").unwrap();

        let mut hdu_type = 0;
        let mut hdu_num = 0;

        unsafe {
            ffopen(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   0,
                   &mut status);
            ffmahd(fptr, 2, &mut hdu_type, &mut status);
            ffghdn(fptr, &mut hdu_num);
            ffclos(fptr, &mut status);
        }

        assert_eq!(hdu_num, 2);
    }

    #[test]
    fn reading_header_key_value() {
        let mut fptr = ptr::null_mut();
        let mut status = -1;
        let c_filename = ffi::CString::new("../testdata/full_example.fits").unwrap();

        let mut long_value = 0;
        let mut float_value = 0.0;
        let mut double_value = 0.0;
        let keyname = ffi::CString::new("INTTEST").unwrap();
        let double_keyname = ffi::CString::new("DBLTEST").unwrap();
        let mut comment: Vec<c_char> = vec![0; 73];
        unsafe {
            ffopen(&mut fptr as *mut *mut fitsfile,
                   c_filename.as_ptr(),
                   0,
                   &mut status);
            ffgkyj(fptr,
                   keyname.as_ptr(),
                   &mut long_value,
                   ptr::null_mut(),
                   &mut status);
            ffgkye(fptr,
                   keyname.as_ptr(),
                   &mut float_value,
                   ptr::null_mut(),
                   &mut status);

            // Double version is different
            ffgkyd(fptr,
                   double_keyname.as_ptr(),
                   &mut double_value,
                   comment.as_mut_ptr(),
                   &mut status);
            ffclos(fptr, &mut status);
        }

        assert_eq!(long_value, 42);
        assert_eq!(float_value, 42.0);
        assert_eq!(double_value, 3. / 32.);

        // TODO Hacky way of getting a string out. This should be simplified.
        let comment: Vec<u8> = comment
            .iter()
            .map(|&x| x as u8)
            .filter(|&x| x != 0)
            .collect();
        let comment = String::from_utf8(comment).unwrap();
        assert_eq!(comment, "Double value");
    }

    // #[test]
    // fn api_usage() {
    // use fitsio::FitsFile;
    //
    // let mut f = FitsFile::open("../testdata/full_example.fits");
    // let mut primary_hdu = f.primary_hdu();
    // let header = primary_hdu.header();
    // let exposure_time: f32 = header["exposure"];
    // }
    //
}
