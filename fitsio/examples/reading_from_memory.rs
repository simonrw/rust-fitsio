// `fitsio` does not currently support opening files from memory, `cfitsio` _does_. This means we
// can use `Fitsfile::from_raw` to load a `FitsFile` from a file that was opened via
// `fits_open_memfile` in `cfitsio`.
use fitsio::{FileOpenMode, FitsFile};
#[cfg(feature = "default")]
use fitsio_sys as sys;
#[cfg(feature = "bindgen")]
use fitsio_sys_bindgen as sys;
use std::io::Read;

fn main() {
    todo!("not finished yet");
    // read the bytes into memory and return a pointer and length to the file
    let (mut ptr, mut ptr_size) = {
        let filename = "../testdata/full_example.fits";
        let mut f = std::fs::File::open(filename).unwrap();
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).unwrap();

        (bytes.as_ptr() as *mut libc::c_void, bytes.len() as u64)
    };

    // now we have a pointer to the data, let's open this in `fitsio_sys`
    let mut fptr = std::ptr::null_mut();
    let mut status = 0;

    let c_filename = std::ffi::CString::new("full_example.fits").unwrap();
    unsafe {
        sys::ffomem(
            &mut fptr as *mut *mut _,
            c_filename.as_ptr(),
            sys::READONLY as _,
            &mut ptr as *mut *mut _,
            &mut ptr_size as *mut _,
            0,
            None,
            &mut status,
        );
    }

    if status != 0 {
        unsafe { sys::ffrprt(sys::stderr, status) };
        panic!("bad status");
    }

    let _f =
        unsafe { FitsFile::from_raw("full_example.fits", fptr, FileOpenMode::READONLY) }.unwrap();
}
