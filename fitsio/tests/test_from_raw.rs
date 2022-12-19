use fitsio::{FileOpenMode, FitsFile};
#[cfg(not(feature = "bindgen"))]
use fitsio_sys;
use fitsio_sys::ffopen;
#[cfg(feature = "bindgen")]
use fitsio_sys_bindgen as fitsio_sys;
use std::ptr;

#[test]
fn from_raw() {
    let filename = "../testdata/full_example.fits";
    let mut fptr = ptr::null_mut();
    let mut status = 0;
    let c_filename = std::ffi::CString::new(filename).expect("filename is not a valid C-string");

    unsafe {
        ffopen(
            &mut fptr as *mut *mut _,
            c_filename.as_ptr(),
            0, // readonly
            &mut status,
        );
    }
    assert_eq!(status, 0);

    let mut f = unsafe { FitsFile::from_raw(fptr, FileOpenMode::READONLY) }.unwrap();

    // the rest of this test is taken from the `images.rs::test_read_image_data` test.
    let hdu = f.hdu(0).unwrap();
    let first_row: Vec<i32> = hdu.read_section(&mut f, 0, 100).unwrap();
    assert_eq!(first_row.len(), 100);
    assert_eq!(first_row[0], 108);
    assert_eq!(first_row[49], 176);

    let second_row: Vec<i32> = hdu.read_section(&mut f, 100, 200).unwrap();
    assert_eq!(second_row.len(), 100);
    assert_eq!(second_row[0], 177);
    assert_eq!(second_row[49], 168);
}
