extern crate fitsio;
extern crate libc;

use libc::c_int;
use std::ffi;
use std::ptr;
use fitsio::raw::*;

#[test]
fn raw_opening_an_existing_file() {
    let mut fptr: *mut fitsfile = ptr::null_mut();
    let mut status: c_int = -1;
    let c_filename = ffi::CString::new("testdata/full_example.fits").unwrap();

    unsafe {
        ffopen(&mut fptr as *mut *mut fitsfile, c_filename.as_ptr(), 0, &mut status);
        ffclos(fptr, &mut status);
    }

    assert_eq!(status, 0);
}

#[test]
fn getting_current_hdu_number() {
    let mut fptr: *mut fitsfile = ptr::null_mut();
    let mut status: c_int = -1;
    let c_filename = ffi::CString::new("testdata/full_example.fits").unwrap();

    let mut hdu_num: c_int = -1;

    unsafe {
        ffopen(&mut fptr as *mut *mut fitsfile, c_filename.as_ptr(), 0, &mut status);
        ffghdn(fptr, &mut hdu_num);
        ffclos(fptr, &mut status);
    }

    assert_eq!(hdu_num, 1);
}

#[test]
fn changing_hdu_by_absolute_number() {
    let mut fptr: *mut fitsfile = ptr::null_mut();
    let mut status: c_int = -1;
    let c_filename = ffi::CString::new("testdata/full_example.fits").unwrap();

    let mut hdu_type: c_int = 0;
    let mut hdu_num: c_int = 0;

    unsafe {
        ffopen(&mut fptr as *mut *mut fitsfile, c_filename.as_ptr(), 0, &mut status);
        ffmahd(fptr, 2, &mut hdu_type, &mut status);
        ffghdn(fptr, &mut hdu_num);
        ffclos(fptr, &mut status);
    }

    assert_eq!(hdu_num, 2);
}
