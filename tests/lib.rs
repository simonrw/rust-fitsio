extern crate fitsio;
extern crate libc;
extern crate tempdir;

use std::ffi;
use std::ptr;
use fitsio::raw::*;

#[test]
fn raw_opening_an_existing_file() {
    let mut fptr = ptr::null_mut();
    let mut status = -1;
    let c_filename = ffi::CString::new("testdata/full_example.fits").unwrap();

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
    let c_filename = ffi::CString::new("testdata/full_example.fits").unwrap();
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
    let c_filename = ffi::CString::new("testdata/full_example.fits").unwrap();

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

/*
#[test]
fn api_usage() {
    use fitsio::FitsFile;

    let mut f = FitsFile::open("testdata/full_example.fits");
    let mut primary_hdu = f.primary_hdu();
    let header = primary_hdu.header();
    let exposure_time: f32 = header["exposure"];
}
*/
